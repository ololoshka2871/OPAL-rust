use stm32l4xx_hal::device::{pwr, rcc};

use crate::support::usb_connection_checker::UsbConnectionChecker;

pub struct VUsbMonitor<'a> {
    was_pwr_disabled: bool,
    rcc: &'a rcc::RegisterBlock,
    pwr: &'a pwr::RegisterBlock,
}

impl<'a> VUsbMonitor<'a> {
    pub fn new(rcc: &'a rcc::RegisterBlock, pwr: &'a pwr::RegisterBlock) -> VUsbMonitor<'a> {
        defmt::trace!("Create USB power monitor");
        let was_pwr_disabled = rcc.apb1enr1.read().pwren().bit_is_clear();

        if was_pwr_disabled {
            defmt::trace!("PWR register block was disabled, enabling...");
            rcc.apb1enr1.modify(|_, w| w.pwren().set_bit());
        } else {
            defmt::trace!("PWR register block was enabled");
        }

        // enable montoring 1.2v
        pwr.cr2.modify(|_, w| w.pvme1().set_bit());

        // без этогй задержки в релизном билде видимо не успевает сработать и всегда детектится что
        // USB питание присутствует
        cortex_m::asm::delay(10);

        VUsbMonitor {
            was_pwr_disabled,
            rcc,
            pwr,
        }
    }
}

impl<'a> Drop for VUsbMonitor<'a> {
    // Деструктор
    fn drop(&mut self) {
        defmt::trace!("Destroing USB power monitor");

        //disable monitoring
        self.pwr.cr2.modify(|_, w| w.pvme1().clear_bit());

        if self.was_pwr_disabled {
            defmt::trace!("PWR register block was disabled, restore..");
            self.rcc.apb1enr1.modify(|_, w| w.pwren().clear_bit());
        }
    }
}

impl<'a> UsbConnectionChecker for VUsbMonitor<'a> {
    fn is_usb_connected(&self) -> bool {
        let mut present = false;

        // wait bit clear
        for _ in 0..3 {
            if self.pwr.sr2.read().pvmo1().bit_is_clear() {
                present = true;
                break;
            }
            cortex_m::asm::delay(1);
        }

        defmt::debug!("USB power monitor PVMO1: {}", present);

        present
    }
}
