pub struct FormatableCStr(pub *const u8);

impl defmt::Format for FormatableCStr {
    fn format(&self, fmt: defmt::Formatter) {
        if fmt.inner.needs_tag() {
            let t = defmt_macros::internp!("{=str}");
            fmt.inner.u8(&t);
        }

        unsafe {
            let slice = core::slice::from_raw_parts(
                self.0,
                strlenn(self.0, crate::config::MAX_TASK_NAME_LEN),
            );

            fmt.inner.leb64(slice.len());
            fmt.inner.write(slice);

            core::mem::forget(slice);
        }
    }
}

unsafe fn strlenn(str: *const u8, max: usize) -> usize {
    for i in 0..max {
        if *str.add(i) == b'\0' {
            return i;
        }
    }
    max
}
