use freertos_rust::{CurrentTask, Duration};
use stm32l4xx_hal::gpio::{Alternate, Floating, Input, AF10, PA11, PA12};
use stm32l4xx_hal::pac::{Peripherals, RCC, USB};
use stm32l4xx_hal::{prelude::*, stm32};

use stm32_usbd::{UsbBus, UsbPeripheral};

use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

pub struct Peripheral {
    pub usb: stm32::USB,
    pub pin_dm: PA11<Alternate<AF10, Input<Floating>>>,
    pub pin_dp: PA12<Alternate<AF10, Input<Floating>>>,
}

unsafe impl Sync for Peripheral {}

unsafe impl UsbPeripheral for Peripheral {
    const REGISTERS: *const () = stm32::USB::ptr() as *const ();

    // internal pull-up supported by stm32l*
    const DP_PULL_UP_FEATURE: bool = true;

    // USB memory region stm32l433.pdf: p.69
    const EP_MEMORY: *const () = 0x4000_6C00 as _;

    // 0x4000_6C00 - 0x4000_6FFF
    const EP_MEMORY_SIZE: usize = 1024;
    const EP_MEMORY_ACCESS_2X16: bool = true;

    fn enable() {
        let crs = unsafe { &(*stm32::CRS::ptr()) };
        let rcc = unsafe { &*RCC::ptr() };
        let pwr = unsafe { &*stm32::PWR::ptr() };

        cortex_m::interrupt::free(|_| {
            // enable crs
            rcc.apb1enr1.modify(|_, w| w.crsen().set_bit());

            // Initialize clock recovery
            // Set autotrim enabled.
            crs.cr.modify(|_, w| w.autotrimen().set_bit());
            // Enable CR
            crs.cr.modify(|_, w| w.cen().set_bit());

            //-------------------------------------------------
            // Disable USB power isolation

            // Enable PWR peripheral
            rcc.apb1enr1.modify(|_, w| w.pwren().set_bit());

            // enable montoring 1.2v
            pwr.cr2.modify(|_, w| w.pvme1().set_bit());

            // wait bit clear
            while !pwr.sr2.read().pvmo1().bit_is_clear() {
                cortex_m::asm::delay(1);
            }

            // disable monitoring
            pwr.cr2.modify(|_, w| w.pvme1().clear_bit());

            // Enable VddUSB
            pwr.cr2.modify(|_, w| w.usv().set_bit());

            //-------------------------------------------------

            // Enable USB peripheral
            rcc.apb1enr1.modify(|_, w| w.usbfsen().set_bit());

            // Reset USB peripheral
            rcc.apb1rstr1
                .modify(|r, w| unsafe { w.bits(r.bits() | (1u32 << 26)) });
            rcc.apb1rstr1
                .modify(|r, w| unsafe { w.bits(r.bits() & !(1u32 << 26)) });
        });
    }

    fn startup_delay() {
        // There is a chip specific startup delay. For STM32F103xx it's 1µs and this should wait for
        // at least that long.
        cortex_m::asm::delay(72);
    }
}

pub fn usbd(dp: Peripherals) -> ! {
    defmt::info!("Usb thread started!");

    let mut rcc = dp.RCC.constrain();
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);

    defmt::info!("Creating usb low-level driver: PA11, PA12, AF10");
    let usb_bus = UsbBus::new(Peripheral {
        usb: dp.USB,
        pin_dm: gpioa.pa11.into_af10(&mut gpioa.moder, &mut gpioa.afrh),
        pin_dp: gpioa.pa12.into_af10(&mut gpioa.moder, &mut gpioa.afrh),
    });

    defmt::info!("Allocating ACM device");
    let mut serial = SerialPort::new(&usb_bus);


    let vid_pid = UsbVidPid(0x16c0, 0x27dd);
    defmt::info!("Building usb device: vid={} pid={}", &vid_pid.0, &vid_pid.1);
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, vid_pid)
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(USB_CLASS_CDC)
        .build();

    loop {
        if !usb_dev.poll(&mut [&mut serial]) {
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
