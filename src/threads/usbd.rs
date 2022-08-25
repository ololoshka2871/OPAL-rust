use core::ops::DerefMut;

use alloc::sync::Arc;
use alloc::vec::Vec;
use freertos_rust::{
    CurrentTask, Duration, FreeRtosError, InterruptContext, Mutex, Task, TaskPriority,
};
use stm32_usbd::UsbBus;

use stm32l4xx_hal::gpio::{Alternate, PushPull};
use stm32l4xx_hal::interrupt;

use stm32l4xx_hal::stm32l4::stm32l4x3::Interrupt;

use usb_device::{class_prelude::UsbBusAllocator, prelude::*};
use usbd_serial::SerialPort;

use crate::{
    support::{self},
    threads::usb_periph::UsbPeriph,
};

static mut USBD_THREAD: Option<freertos_rust::Task> = None;

static mut USBD: Option<Usbd> = None;

pub struct UsbdPeriph {
    pub usb: stm32l4xx_hal::device::USB,
    pub pin_dm: stm32l4xx_hal::gpio::PA11<Alternate<PushPull, 10>>,
    pub pin_dp: stm32l4xx_hal::gpio::PA12<Alternate<PushPull, 10>>,
}

pub struct Usbd {
    usb_bus: UsbBusAllocator<UsbBus<UsbPeriph>>,
    interrupt_controller: Arc<dyn support::interrupt_controller::IInterruptController>,
    interrupt_prio: u8,

    serial: Option<SerialPort<'static, UsbBus<UsbPeriph>>>,
    serial_port: Option<Arc<Mutex<&'static mut SerialPort<'static, UsbBus<UsbPeriph>>>>>,

    subscribers: Vec<Task>,
}

impl Usbd {
    pub fn init(
        usbd_periph: UsbdPeriph,
        interrupt_controller: Arc<dyn support::interrupt_controller::IInterruptController>,
        interrupt_prio: u8,
    ) {
        if unsafe { USBD.is_some() } {
            return;
        }

        defmt::info!("Creating usb low-level driver");

        let res = Self {
            usb_bus: UsbBus::new(UsbPeriph {
                usb: usbd_periph.usb,
                pin_dm: usbd_periph.pin_dm,
                pin_dp: usbd_periph.pin_dp,
            }),
            interrupt_controller,
            interrupt_prio,

            serial: None,
            serial_port: None,

            subscribers: Vec::new(),
        };

        unsafe {
            // Должен быть статик, так как заимствуется сущностью, которая будет статик.
            USBD = Some(res);
        }
    }

    fn get_static_self() -> &'static mut Usbd {
        unsafe { USBD.as_mut().expect("Call Usbd::init() first!") }
    }

    pub fn serial_port() -> Arc<Mutex<&'static mut SerialPort<'static, UsbBus<UsbPeriph>>>> {
        let mut _self = Self::get_static_self();

        if _self.serial_port.is_none() {
            defmt::info!("Allocating ACM device");
            _self.serial = Some(SerialPort::new(&_self.usb_bus));

            _self.serial_port = Some(Arc::new(
                Mutex::new(_self.serial.as_mut().unwrap())
                    .expect("Failed to create serial guard mutex"),
            ));
        }
        _self.serial_port.as_ref().unwrap().clone()
    }

    pub fn subsbrbe(task: Task) {
        let mut _self = Self::get_static_self();

        _self.subscribers.push(task);
    }

    pub fn strat(
        vid_pid: UsbVidPid,
        stack_size: u16,
        priority: TaskPriority,
    ) -> Result<(), FreeRtosError> {
        let mut _self = Self::get_static_self();

        let thread = Task::new()
            .name("Usbd")
            .stack_size(stack_size)
            .priority(priority)
            .start(move |_| {
                defmt::info!("Usb thread started!");
                defmt::info!("Building usb device: vid={} pid={}", &vid_pid.0, &vid_pid.1);

                let mut usb_dev = UsbDeviceBuilder::new(&_self.usb_bus, vid_pid)
                    .manufacturer("SCTB ELPA")
                    .product("OPAL-rust")
                    .serial_number("0123456789")
                    .composite_with_iads()
                    .build();

                defmt::trace!("Set usb interrupt prio = {}", _self.interrupt_prio);
                _self
                    .interrupt_controller
                    .set_priority(Interrupt::USB_FS.into(), _self.interrupt_prio);

                defmt::info!("USB ready!");

                let serial_port = _self
                    .serial_port
                    .as_ref()
                    .expect("call Usbd::serial_port() before!");

                loop {
                    // Важно! Список передаваемый сюда в том же порядке,
                    // что были инициализированы интерфейсы
                    let res = match serial_port.lock(Duration::ms(1)) {
                        Ok(mut serial) => usb_dev.poll(&mut [*serial.deref_mut()]),
                        Err(_) => true,
                    };

                    if !res {
                        //crate::support::led::led_set(0);
                        // block until usb interrupt
                        // interrupt_controller.unpend(Interrupt::USB_FS.into()); // без этого скорость в 1,5 раза выше
                        _self.interrupt_controller.unmask(Interrupt::USB_FS.into());

                        unsafe {
                            let _ = freertos_rust::Task::current()
                                .unwrap_unchecked()
                                // ожидаем, что нотификационное значение будет > 0
                                .take_notification(true, Duration::infinite());
                        }

                        _self.interrupt_controller.mask(Interrupt::USB_FS.into());
                    } else {
                        //crate::support::led::led_set(1);
                        _self
                            .subscribers
                            .iter()
                            .for_each(|s| s.notify(freertos_rust::TaskNotification::Increment));

                        //support::mast_yield();
                        CurrentTask::delay(Duration::ms(1));
                    }
                }
            })?;

        unsafe {
            USBD_THREAD = Some(thread);
        }

        Ok(())
    }
}

// USB exception
// ucCurrentPriority >= ucMaxSysCallPriority (80)
#[interrupt]
unsafe fn USB_FS() {
    let interrupt_ctx = InterruptContext::new();
    if let Some(usbd) = USBD_THREAD.as_ref() {
        // Результат не особо важен
        // инкремент нотификационного значения
        let _ = usbd.notify_from_isr(&interrupt_ctx, freertos_rust::TaskNotification::Increment);
    }

    // Как только прерывание случилось, мы посылаем сигнал потоку
    // НО ивент вызвавший прерыывание пока не снялся, поэтому мы будем
    // бесконечно в него заходить по кругу, нужно запретить пока что это
    // прерывание
    // TODO: device independent layer
    cortex_m::peripheral::NVIC::mask(Interrupt::USB_FS);
    cortex_m::peripheral::NVIC::unpend(Interrupt::USB_FS);
}
