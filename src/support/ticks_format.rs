#[derive(Default)]
pub struct Ticks(pub u32);

impl defmt::Format for Ticks {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "{:09}", self.0);
    }
}
