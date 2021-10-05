use stm32_usbd::UsbPeripheral;
use stm32l4xx_hal::device::RCC;
use stm32l4xx_hal::gpio::{AF10, Alternate, Floating, Input, PA11, PA12};
use stm32l4xx_hal::stm32;

pub struct UsbPeriph {
    pub usb: stm32::USB,
    pub pin_dm: PA11<Alternate<AF10, Input<Floating>>>,
    pub pin_dp: PA12<Alternate<AF10, Input<Floating>>>,
}

unsafe impl Sync for UsbPeriph {}

unsafe impl UsbPeripheral for UsbPeriph {
    const REGISTERS: *const () = stm32::USB::ptr() as *const ();

    // internal pull-up supported by stm32l*
    const DP_PULL_UP_FEATURE: bool = true;

    // USB memory region stm32l433.pdf: p.69
    const EP_MEMORY: *const () = 0x4000_6C00 as _;

    // 0x4000_6C00 - 0x4000_6FFF
    const EP_MEMORY_SIZE: usize = 1024;
    const EP_MEMORY_ACCESS_2X16: bool = true;

    fn enable() {
        let crs = unsafe { &*stm32::CRS::ptr() };
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
