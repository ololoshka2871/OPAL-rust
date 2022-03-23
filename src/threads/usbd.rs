use core::ops::DerefMut;

use alloc::sync::Arc;
use freertos_rust::{Duration, InterruptContext, Mutex, Task, TaskPriority};
use my_proc_macro::c_str;
use stm32_usbd::UsbBus;

use stm32l4xx_hal::gpio::{Alternate, PushPull};
use stm32l4xx_hal::interrupt;

use stm32l4xx_hal::stm32l4::stm32l4x3::Interrupt;

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
    pub pin_dm: stm32l4xx_hal::gpio::PA11<Alternate<PushPull, 10>>,
    pub pin_dp: stm32l4xx_hal::gpio::PA12<Alternate<PushPull, 10>>,
}

pub fn usbd(
    usbd_periph: UsbdPeriph,
    interrupt_controller: Arc<dyn support::interrupt_controller::IInterruptController>,
    interrupt_prio: u8,
    output: Arc<Mutex<crate::workmodes::output_storage::OutputStorage>>,
    cq: Arc<freertos_rust::Queue<super::sensor_processor::Command>>,
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

    defmt::info!("Allocating SCSI device");
    let mut scsi = Scsi::new(
        unsafe { USB_BUS.as_ref().unwrap_unchecked() }, //&usb_bus,
        64, // для устройств full speed: max_packet_size 8, 16, 32 or 64
        EMfatStorage::new(c_str!("LOGGER")),
        "SCTB", // <= max 8 больших букв
        "SelfWriter",
        "L442",
    );

    /*
    defmt::info!("Allocating ACM device");
    let serial = SerialPort::new(unsafe { USB_BUS.as_ref().unwrap_unchecked() });

    let serial_container =
        Arc::new(Mutex::new(serial).expect("Failed to create serial guard mutex"));
        */

    let vid_pid = UsbVidPid(0x0483, 0x5720);
    defmt::info!("Building usb device: vid={} pid={}", &vid_pid.0, &vid_pid.1);
    let mut usb_dev =
        UsbDeviceBuilder::new(unsafe { USB_BUS.as_ref().unwrap_unchecked() }, vid_pid)
            .manufacturer("SCTB ELPA")
            .product("Pressure self-registrator")
            .serial_number("0123456789")
            //.device_class(0) // Это не нужно для композита
            .composite_with_iads()
            .build();

    defmt::trace!("Set usb interrupt prio = {}", interrupt_prio);
    interrupt_controller.set_priority(Interrupt::USB_FS.into(), interrupt_prio);

    defmt::info!("USB ready!");

    /*
    let protobuf_srv = {
        let sn = serial_container.clone();
        defmt::trace!("Creating protobuf server thread...");
        Task::new()
            .name("Protobuf")
            .stack_size(2048)
            .priority(TaskPriority(crate::config::PROTOBUF_TASK_PRIO))
            .start(move |_| protobuf_server::protobuf_server(sn, output, cq))
            .expect("Failed to create protobuf server")
    };
    */

    loop {
        // Важно! Список передаваемый сюда в том же порядке,
        // что были инициализированы интерфейсы
        let res = /*match serial_container.lock(Duration::ms(1)) {
            Ok(mut serial) => usb_dev.poll(&mut [&mut scsi, serial.deref_mut()]),
            Err(_) => true,
        };*/
        usb_dev.poll(&mut [&mut scsi]);

        if !res {
            // block until usb interrupt
            interrupt_controller.unpend(Interrupt::USB_FS.into());
            interrupt_controller.unmask(Interrupt::USB_FS.into());

            unsafe {
                let _ = freertos_rust::Task::current()
                    .unwrap_unchecked()
                    // ожидаем, что нотификационное значение будет > 0
                    .take_notification(true, Duration::infinite());
            }

            interrupt_controller.mask(Interrupt::USB_FS.into());
        } else {
            //protobuf_srv.notify(freertos_rust::TaskNotification::Increment);
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
        let _ = usbd.notify_from_isr(&interrupt_ctx, freertos_rust::TaskNotification::SetValue(1));
    }

    // Как только прерывание случилось, мы посылаем сигнал потоку
    // НО ивент вызвавший прерыывание пока не снялся, поэтому мы будем
    // бесконечно в него заходить по кругу, нужно запретить пока что это
    // прерывание
    // TODO: device independent layer
    cortex_m::peripheral::NVIC::mask(Interrupt::USB_FS);
    cortex_m::peripheral::NVIC::unpend(Interrupt::USB_FS);
}
