use core::ops::DerefMut;

use alloc::sync::Arc;
use alloc::vec::Vec;
use embedded_hal::digital::v2::OutputPin;
use freertos_rust::{
    Duration, FreeRtosError, InterruptContext, Mutex, Task, TaskNotification, TaskPriority,
};
use stm32_usbd::UsbBus;

use stm32f1xx_hal::gpio::{self, Floating, Input};

use stm32f1xx_hal::pac::interrupt;
use stm32f1xx_hal::stm32::Interrupt;

use stm32f1xx_hal::usb::Peripheral;
use usb_device::{class_prelude::UsbBusAllocator, prelude::*};
use usbd_serial::SerialPort;

use crate::support::{self};

static mut USBD_THREAD: Option<freertos_rust::Task> = None;

static mut USBD: Option<Usbd> = None;

pub struct UsbdPeriph<PIN: OutputPin> {
    pub usb: stm32f1xx_hal::device::USB,
    pub pin_dm: gpio::PA11<Input<Floating>>,
    pub pin_dp: gpio::PA12<Input<Floating>>,
    pub usb_pull_up: PIN,
}

pub struct Usbd {
    usb_bus: UsbBusAllocator<UsbBus<Peripheral>>,
    interrupt_controller: Arc<dyn support::interrupt_controller::IInterruptController>,
    interrupt_prio: u8,

    serial: Option<SerialPort<'static, UsbBus<Peripheral>>>,
    serial_port: Option<Arc<Mutex<&'static mut SerialPort<'static, UsbBus<Peripheral>>>>>,

    subscribers: Vec<Task>,
}

impl Usbd {
    pub fn init<PIN: OutputPin>(
        mut usbd_periph: UsbdPeriph<PIN>,
        interrupt_controller: Arc<dyn support::interrupt_controller::IInterruptController>,
        interrupt_prio: u8,
        usb_pull_up_cative_state: embedded_hal::digital::v2::PinState,
    ) {
        if unsafe { USBD.is_some() } {
            return;
        }

        defmt::info!("Creating usb low-level driver");

        let res = Self {
            usb_bus: UsbBus::new(Peripheral {
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
        let _ = usbd_periph.usb_pull_up.set_state(usb_pull_up_cative_state);
    }

    fn get_static_self() -> &'static mut Usbd {
        unsafe { USBD.as_mut().expect("Call Usbd::init() first!") }
    }

    pub fn serial_port() -> Arc<Mutex<&'static mut SerialPort<'static, UsbBus<Peripheral>>>> {
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

    pub fn subscribe(task: Task) {
        let mut _self = Self::get_static_self();

        _self.subscribers.push(task);
    }

    pub fn strat(
        vid_pid: UsbVidPid,
        name: &'static str,
        manufacturer: &'static str,
        serial: &'static str,
        stack_size: usize,
        priority: TaskPriority,
    ) -> Result<(), FreeRtosError> {
        let mut _self = Self::get_static_self();

        let thread = Task::new()
            .name("Usbd")
            .stack_size((stack_size / core::mem::size_of::<u32>()) as u16)
            .priority(priority)
            .start(move |_| {
                defmt::info!("Usb thread started!");
                defmt::info!("Building usb device: vid={} pid={}", &vid_pid.0, &vid_pid.1);

                let mut usb_dev = UsbDeviceBuilder::new(&_self.usb_bus, vid_pid)
                    .manufacturer(manufacturer)
                    .product(name)
                    .serial_number(serial)
                    .composite_with_iads()
                    .build();

                {
                    defmt::trace!("Set usb interrupt prio = {}", _self.interrupt_prio);
                    _self
                        .interrupt_controller
                        .set_priority(Interrupt::USB_HP_CAN_TX.into(), _self.interrupt_prio);
                    _self
                        .interrupt_controller
                        .set_priority(Interrupt::USB_LP_CAN_RX0.into(), _self.interrupt_prio);
                }

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

                    if res {
                        // crate::support::led::led_set(1);
                        _self
                            .subscribers
                            .iter()
                            .for_each(|s| s.notify(TaskNotification::Increment));

                        support::mast_yield();
                    } else {
                        // crate::support::led::led_set(0);

                        // block until usb interrupt
                        cortex_m::interrupt::free(|_| {
                            _self
                                .interrupt_controller
                                .unmask(Interrupt::USB_HP_CAN_TX.into());
                            _self
                                .interrupt_controller
                                .unmask(Interrupt::USB_LP_CAN_RX0.into());
                        });

                        unsafe {
                            let _ = freertos_rust::Task::current()
                                .unwrap_unchecked()
                                // ожидаем, что нотификационное значение будет > 0
                                .take_notification(true, Duration::infinite());
                        }

                        cortex_m::interrupt::free(|_| {
                            _self
                                .interrupt_controller
                                .mask(Interrupt::USB_HP_CAN_TX.into());
                            _self
                                .interrupt_controller
                                .mask(Interrupt::USB_LP_CAN_RX0.into());
                        });
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
unsafe fn USB_HP_CAN_TX() {
    use cortex_m::peripheral::NVIC;

    usb_interrupt();

    NVIC::mask(Interrupt::USB_HP_CAN_TX);
    NVIC::unpend(Interrupt::USB_HP_CAN_TX);
}

#[interrupt]
unsafe fn USB_LP_CAN_RX0() {
    use cortex_m::peripheral::NVIC;

    usb_interrupt();

    NVIC::mask(Interrupt::USB_LP_CAN_RX0);
    NVIC::unpend(Interrupt::USB_LP_CAN_RX0);
}

unsafe fn usb_interrupt() {
    let interrupt_ctx = InterruptContext::new();
    if let Some(usbd) = USBD_THREAD.as_ref() {
        // Результат не особо важен
        // инкремент нотификационного значения
        let _ = usbd.notify_from_isr(&interrupt_ctx, TaskNotification::Increment);
    }

    // Как только прерывание случилось, мы посылаем сигнал потоку
    // НО ивент вызвавший прерыывание пока не снялся, поэтому мы будем
    // бесконечно в него заходить по кругу, нужно запретить пока что это
    // прерывание
    // TODO: device independent layer
    // cortex_m::peripheral::NVIC::mask(Interrupt::USB...);
    // cortex_m::peripheral::NVIC::unpend(Interrupt::USB...);
}
