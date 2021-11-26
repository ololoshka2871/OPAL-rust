use core::ops::DerefMut;

use alloc::sync::Arc;
use freertos_rust::{Duration, InterruptContext, Mutex, Task, TaskPriority};
use my_proc_macro::c_str;
use stm32_usbd::UsbBus;

use stm32l4xx_hal::interrupt;

use stm32l4xx_hal::stm32l4::stm32l4x2::Interrupt;

use usb_device::{class_prelude::UsbBusAllocator, prelude::*};
use usbd_scsi::Scsi;
use usbd_serial::SerialPort;

use crate::{
    support::{self},
    threads::{protobuf_server, usb_periph::UsbPeriph, vfs::EMfatStorage},
};

static mut USBD_THREAD: Option<freertos_rust::Task> = None;
static mut USB_BUS: Option<UsbBusAllocator<UsbBus<UsbPeriph>>> = None;

pub struct UsbdPeriph {
    pub usb: stm32l4xx_hal::device::USB,
    pub gpioa: stm32l4xx_hal::gpio::gpioa::Parts,
}

pub fn usbd(
    mut usbd_periph: UsbdPeriph,
    interrupt_controller: Arc<dyn support::interrupt_controller::IInterruptController>,
    interrupt_prio: u8,
) -> ! {
    defmt::info!("Usb thread started!");

    unsafe {
        USBD_THREAD = Some(freertos_rust::Task::current().unwrap());
    }

    defmt::info!("Creating usb low-level driver: PA11, PA12, AF10");
    unsafe {
        // Должен быть статик, так как заимствуется сущностью, которая будет статик.
        USB_BUS = Some(UsbBus::new(UsbPeriph {
            usb: usbd_periph.usb,
            pin_dm: usbd_periph
                .gpioa
                .pa11
                .into_af10(&mut usbd_periph.gpioa.moder, &mut usbd_periph.gpioa.afrh),
            pin_dp: usbd_periph
                .gpioa
                .pa12
                .into_af10(&mut usbd_periph.gpioa.moder, &mut usbd_periph.gpioa.afrh),
        }))
    }

    defmt::info!("Allocating SCSI device");
    let mut scsi = Scsi::new(
        unsafe { USB_BUS.as_ref().unwrap() }, //&usb_bus,
        64, // для устройств full speed: max_packet_size 8, 16, 32 or 64
        EMfatStorage::new(c_str!("LOGGER")),
        "SCTB", // <= max 8 больших букв
        "SelfWriter",
        "L442",
    );

    defmt::info!("Allocating ACM device");
    let serial = SerialPort::new(unsafe { USB_BUS.as_ref().unwrap() });

    let serial_container =
        Arc::new(Mutex::new(serial).expect("Failed to create serial guard mutex"));

    let vid_pid = UsbVidPid(0x0483, 0x5720);
    defmt::info!("Building usb device: vid={} pid={}", &vid_pid.0, &vid_pid.1);
    let mut usb_dev = UsbDeviceBuilder::new(unsafe { USB_BUS.as_ref().unwrap() }, vid_pid)
        .manufacturer("SCTB ELPA")
        .product("Pressure self-registrator")
        .serial_number("0123456789")
        //.device_class(0) // Это не нужно для композита
        .composite_with_iads()
        .build();

    defmt::trace!("Set usb interrupt prio = {}", interrupt_prio);
    interrupt_controller.set_priority(Interrupt::USB.into(), interrupt_prio);

    defmt::info!("USB ready!");

    {
        let sn = serial_container.clone();
        defmt::trace!("Creating protobuf server thread...");
        Task::new()
            .name("Protobuf")
            .stack_size(2048)
            .priority(TaskPriority(1))
            .start(move |_| protobuf_server::protobuf_server(sn))
            .expect("Failed to create protobuf server");
    }

    loop {
        // Важно! Список передаваемый сюда в том же порядке,
        // что были инициализированы интерфейсы
        let res = match serial_container.lock(Duration::ms(1)) {
            Ok(mut serial) => usb_dev.poll(&mut [&mut scsi, serial.deref_mut()]),
            Err(_) => true,
        };

        if !res {
            // block until usb interrupt
            interrupt_controller.unpend(Interrupt::USB.into());
            interrupt_controller.unmask(Interrupt::USB.into());

            let _ = freertos_rust::Task::current()
                .unwrap()
                .wait_for_notification(0, 0, Duration::infinite());

            interrupt_controller.mask(Interrupt::USB.into());
        }
    }
}

// USB exception

// ucCurrentPriority >= ucMaxSysCallPriority (80)
#[interrupt]
unsafe fn USB() {
    let interrupt_ctx = InterruptContext::new();
    if let Some(usbd) = USBD_THREAD.as_ref() {
        // Результат не особо важен
        core::mem::forget(
            usbd.notify_from_isr(&interrupt_ctx, freertos_rust::TaskNotification::NoAction),
        );
    }

    // Как только прерывание случилось, мы посылаем сигнал потоку
    // НО ивент вызвавший прерыывание пока не снялся, поэтому мы будем
    // бесконечно в него заходить по кругу, нужно запретить пока что это
    // прерывание
    // TODO: device independent layer
    cortex_m::peripheral::NVIC::mask(Interrupt::USB);
    cortex_m::peripheral::NVIC::unpend(Interrupt::USB);
}
