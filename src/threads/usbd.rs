use core::ops::DerefMut;

use alloc::sync::Arc;
use freertos_rust::{Duration, InterruptContext, Mutex, Task, TaskPriority};
use stm32_usbd::UsbBus;

use stm32l4xx_hal::gpio::{Alternate, PushPull};
use stm32l4xx_hal::interrupt;

use stm32l4xx_hal::stm32l4::stm32l4x3::Interrupt;

use usb_device::{class_prelude::UsbBusAllocator, prelude::*};
use usbd_serial::SerialPort;

use crate::threads::gcode_server;
use crate::{
    support::{self},
    threads::usb_periph::UsbPeriph,
};

static mut USBD_THREAD: Option<freertos_rust::Task> = None;
static mut USB_BUS: Option<UsbBusAllocator<UsbBus<UsbPeriph>>> = None;

pub struct UsbdPeriph {
    pub usb: stm32l4xx_hal::device::USB,
    pub pin_dm: stm32l4xx_hal::gpio::PA11<Alternate<PushPull, 10>>,
    pub pin_dp: stm32l4xx_hal::gpio::PA12<Alternate<PushPull, 10>>,
}

pub fn usbd(
    usbd_periph: UsbdPeriph,
    interrupt_controller: Arc<dyn support::interrupt_controller::IInterruptController>,
    interrupt_prio: u8,
) -> ! {
    defmt::info!("Usb thread started!");

    unsafe {
        USBD_THREAD = Some(freertos_rust::Task::current().unwrap_unchecked());
    }

    defmt::info!("Creating usb low-level driver: PA11, PA12, AF10");

    unsafe {
        // Должен быть статик, так как заимствуется сущностью, которая будет статик.
        USB_BUS = Some(UsbBus::new(UsbPeriph {
            usb: usbd_periph.usb,
            pin_dm: usbd_periph.pin_dm,
            pin_dp: usbd_periph.pin_dp,
        }))
    }

    defmt::info!("Allocating ACM device");
    let serial = SerialPort::new(unsafe { USB_BUS.as_ref().unwrap_unchecked() });

    let serial_container =
        Arc::new(Mutex::new(serial).expect("Failed to create serial guard mutex"));

    let vid_pid = UsbVidPid(0x0483, 0x5720);
    defmt::info!("Building usb device: vid={} pid={}", &vid_pid.0, &vid_pid.1);
    let mut usb_dev =
        UsbDeviceBuilder::new(unsafe { USB_BUS.as_ref().unwrap_unchecked() }, vid_pid)
            .manufacturer("SCTB ELPA")
            .product("OPAL-rust")
            .serial_number("0123456789")
            //.device_class(0) // Это не нужно для композита
            .composite_with_iads()
            .build();

    defmt::trace!("Set usb interrupt prio = {}", interrupt_prio);
    interrupt_controller.set_priority(Interrupt::USB_FS.into(), interrupt_prio);

    defmt::info!("USB ready!");

    let gcode_srv = {
        let sn = serial_container.clone();
        defmt::trace!("Creating G-Code server thread...");
        Task::new()
            .name("G-CODE")
            .stack_size(2048)
            .priority(TaskPriority(crate::config::GCODE_TASK_PRIO))
            .start(move |_| gcode_server::gcode_server(sn))
            .expect("Failed to create G-CODE server")
    };

    loop {
        // Важно! Список передаваемый сюда в том же порядке,
        // что были инициализированы интерфейсы
        let res = match serial_container.lock(Duration::ms(1)) {
            Ok(mut serial) => usb_dev.poll(&mut [serial.deref_mut()]),
            Err(_) => true,
        };

        if !res {
            crate::support::led::led_set(0);
            // block until usb interrupt
            // interrupt_controller.unpend(Interrupt::USB_FS.into()); // без этого скорость в 1,5 раза выше
            interrupt_controller.unmask(Interrupt::USB_FS.into());

            unsafe {
                let _ = freertos_rust::Task::current()
                    .unwrap_unchecked()
                    // ожидаем, что нотификационное значение будет > 0
                    .take_notification(true, Duration::infinite());
            }

            interrupt_controller.mask(Interrupt::USB_FS.into());
        } else {
            crate::support::led::led_set(1);
            gcode_srv.notify(freertos_rust::TaskNotification::Increment);
        }
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
