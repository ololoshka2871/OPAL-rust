use stm32l4xx_hal::gpio::{Alternate, Floating, Input, AF10, PA11, PA12};
use stm32l4xx_hal::pac::{Peripherals, PWR, RCC, USB};
use stm32l4xx_hal::{prelude::*, stm32};

use stm32_usbd::UsbBus;

use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};


pub struct Peripheral {
    pub usb: USB,
    pub pin_dm: PA11<Alternate<AF10, Input<Floating>>>,
    pub pin_dp: PA12<Alternate<AF10, Input<Floating>>>,
}

unsafe impl Sync for Peripheral {}

unsafe impl UsbPeripheral for Peripheral {
    const REGISTERS: *const () = USB::ptr() as *const ();

    const DP_PULL_UP_FEATURE: bool = true; // internal pull-up supported by stm32l*
                                           // stm32l433.pdf: p.69
    const EP_MEMORY: *const () = 0x4000_6C00 as _;
    const EP_MEMORY_SIZE: usize = 1024;
    const EP_MEMORY_ACCESS_2X16: bool = true; // FIXME

    fn enable() {
        let rcc = unsafe { &*RCC::ptr() };
        let pwr = unsafe { &*PWR::ptr() };

        cortex_m::interrupt::free(|_| {
            // Enable USB peripheral
            rcc.apb1enr1.modify(|_, w| w.usbfsen().set_bit());

            // NVIC - TODO

            {
                // Enable PWR peripheral
                rcc.apb1enr1.modify(|_, w| w.pwren().set_bit());

                // enable montoring 1.2v
                pwr.cr2.modify(|_, w| w.pvme1().set_bit());

                // wait bit clear
                while pwr.sr2.read().pvmo1().bits() {
                    cortex_m::asm::delay(1);
                }

                // disable monitoring
                pwr.cr2.modify(|_, w| w.pvme1().clear_bit());

                // Enable VddUSB
                pwr.cr2.modify(|_, w| w.usv().set_bit());
            }

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
        //CurrentTask::delay(Duration::ms(100));
        cortex_m::asm::delay(72);
    }
}


fn enable_crs(rcc: &mut stm32::RCC, crs: &mut stm32::CRS) {
    rcc.apb1enr1.modify(|_, w| w.crsen().set_bit());

    // Initialize clock recovery
    // Set autotrim enabled.
    crs.cr.modify(|_, w| w.autotrimen().set_bit());
    // Enable CR
    crs.cr.modify(|_, w| w.cen().set_bit());
}

fn enable_usb_pwr(rcc: &mut stm32::RCC, pwr: &mut stm32::PWR) {
    // Enable PWR peripheral
    rcc.apb1enr1.modify(|_, w| w.pwren().set_bit());

    // enable montoring 1.2v
    pwr.cr2.modify(|_, w| w.pvme1().set_bit());

    // wait bit clear
    while pwr.sr2.read().pvmo1().bits() {
        CurrentTask::delay(Duration::ms(1));
    }

    // disable monitoring
    pwr.cr2.modify(|_, w| w.pvme1().clear_bit());

    // Enable VddUSB
    pwr.cr2.modify(|_, w| w.usv().set_bit());
}

pub fn usbd(mut dp: Peripherals) -> ! {
    // disable Vddusb power isolation
    enable_crs(&mut dp.RCC, &mut dp.CRS);
    //enable_usb_pwr(&mut dp.RCC, &mut dp.PWR);

    let mut rcc = dp.RCC.constrain();
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);

    let usb_dm = gpioa.pa11.into_af10(&mut gpioa.moder, &mut gpioa.afrh);
    let usb_dp = gpioa.pa12.into_af10(&mut gpioa.moder, &mut gpioa.afrh);

    let usb_bus = UsbBus::new(dp.USB, (usb_dm, usb_dp));

    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(USB_CLASS_CDC)
        .build();

    //usb_dev.force_reset();

    loop {
        if !usb_dev.poll(&mut [&mut serial]) {
            //CurrentTask::delay(Duration::ms(1));
            continue;
        }

        let mut buf = [0u8; 64];

        match serial.read(&mut buf) {
            Ok(count) if count > 0 => {
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
