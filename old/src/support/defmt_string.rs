use alloc::string::String;

pub struct DefmtString<'a>(pub &'a String);

impl<'a> defmt::Format for DefmtString<'a> {
    fn format(&self, fmt: defmt::Formatter) {
        if fmt.inner.needs_tag() {
            let t = defmt_macros::internp!("{=str}");
            fmt.inner.u8(&t);
        }

        fmt.inner.leb64(self.0.len());
        fmt.inner.write(self.0.as_bytes());
    }
}
