use stm32l4xx_hal::pac::Peripherals;
use stm32l4xx_hal::prelude::*;

use stm32_usbd::UsbBus;

use usb_device::prelude::*;
use usbd_scsi::Scsi;
use usbd_serial::SerialPort;

use crate::threads::{storage::Storage, usb_periph::UsbPeriph};

pub fn usbd(dp: Peripherals) -> ! {
    defmt::info!("Usb thread started!");

    let mut rcc = dp.RCC.constrain();
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);

    defmt::info!("Creating usb low-level driver: PA11, PA12, AF10");
    let usb_bus = UsbBus::new(UsbPeriph {
        usb: dp.USB,
        pin_dm: gpioa.pa11.into_af10(&mut gpioa.moder, &mut gpioa.afrh),
        pin_dp: gpioa.pa12.into_af10(&mut gpioa.moder, &mut gpioa.afrh),
    });

    defmt::info!("Allocating ACM device");
    let mut serial = SerialPort::new(&usb_bus);

    defmt::info!("Allocating SCSI device");
    let mut scsi = Scsi::new(
        &usb_bus,
        64,
        Storage {},
        "SCTB", // <= 8 больших букв
        "SelfWriter",
        "L442",
    );

    let vid_pid = UsbVidPid(0x16c0, 0x27dd);
    defmt::info!("Building usb device: vid={} pid={}", &vid_pid.0, &vid_pid.1);
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, vid_pid)
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        //.device_class(USB_CLASS_CDC)
        .device_class(usbd_mass_storage::USB_CLASS_MSC)
        .build();

    loop {
        if !usb_dev.poll(&mut [&mut serial, &mut scsi]) {
            //CurrentTask::delay(Duration::ms(1));
            continue;
        }
        defmt::trace!("USB device polled succesfuly!");

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
}
