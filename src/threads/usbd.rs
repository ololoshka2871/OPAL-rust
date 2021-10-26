use freertos_rust::{Duration, InterruptContext};
use stm32_usbd::UsbBus;

use stm32l4xx_hal::interrupt;

use stm32l4xx_hal::stm32l4::stm32l4x2::Interrupt;

use usb_device::prelude::*;
use usbd_scsi::Scsi;
use usbd_serial::SerialPort;

use crate::threads::{storage::EMfatStorage, usb_periph::UsbPeriph};

static mut USBD_THREAD: Option<freertos_rust::Task> = None;

static POOL_FORCED: u8 = 3;

pub struct UsbdPeriph {
    pub usb: stm32l4xx_hal::device::USB,
    pub gpioa: stm32l4xx_hal::gpio::gpioa::Parts,
}

pub fn usbd(mut usbd_periph: UsbdPeriph) -> ! {
    defmt::info!("Usb thread started!");

    unsafe {
        USBD_THREAD = Some(freertos_rust::Task::current().unwrap());
    }

    defmt::info!("Creating usb low-level driver: PA11, PA12, AF10");
    let usb_bus = UsbBus::new(UsbPeriph {
        usb: usbd_periph.usb,
        pin_dm: usbd_periph
            .gpioa
            .pa11
            .into_af10(&mut usbd_periph.gpioa.moder, &mut usbd_periph.gpioa.afrh),
        pin_dp: usbd_periph
            .gpioa
            .pa12
            .into_af10(&mut usbd_periph.gpioa.moder, &mut usbd_periph.gpioa.afrh),
    });

    defmt::info!("Allocating SCSI device");
    let mut scsi = Scsi::new(
        &usb_bus, //&usb_bus,
        64,       // для устройств full speed: max_packet_size 8, 16, 32 or 64
        EMfatStorage::new("LOGGER\0"),
        "SCTB", // <= max 8 больших букв
        "SelfWriter",
        "L442",
    );

    defmt::info!("Allocating ACM device");
    let mut serial = SerialPort::new(&usb_bus);

    let vid_pid = UsbVidPid(0x0483, 0x5720);
    defmt::info!("Building usb device: vid={} pid={}", &vid_pid.0, &vid_pid.1);
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, vid_pid)
        .manufacturer("SCTB ELPA")
        .product("Pressure self-registrator")
        .serial_number("0123456789")
        //.device_class(0) // Это не нужно для композита
        .composite_with_iads()
        .build();

    defmt::info!("USB ready");
    let mut pool_failed = 0u8;
    loop {
        // Важно! Список передаваемый сюда в том же порядке,
        // что были инициализированы интерфейсы
        if !usb_dev.poll(&mut [&mut scsi, &mut serial]) {
            pool_failed += 1;
            if pool_failed > POOL_FORCED {
                // block until usb interrupt
                unsafe {
                    cortex_m::peripheral::NVIC::unmask(Interrupt::USB);
                }
                core::mem::forget(
                    freertos_rust::Task::current()
                        .unwrap()
                        .wait_for_notification(0, 0, Duration::ms(5)),
                );
                pool_failed = 0;
            }
            continue;
        }

        process_serial(&mut serial);
    }
}

fn process_serial<B: usb_device::bus::UsbBus>(serial: &mut SerialPort<B>) {
    let mut buf = [0u8; 64];

    match serial.read(&mut buf) {
        Ok(count) if count > 0 => {
            defmt::info!("Serial> Ressived {} bytes", count);
            // Echo back in upper case
            for c in buf[0..count].iter_mut() {
                if 0x61 <= *c && *c <= 0x7a {
                    *c &= !0x20;
                }
            }

            let mut write_offset = 0;
            while write_offset < count {
                match serial.write(&buf[write_offset..count]) {
                    Ok(len) if len > 0 => {
                        write_offset += len;
                    }
                    _ => {}
                }
            }
        }
        _ => {}
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
    cortex_m::peripheral::NVIC::mask(Interrupt::USB);
    cortex_m::peripheral::NVIC::unpend(Interrupt::USB);
}
