use alloc::{string::String, vec::Vec};

pub trait Stream<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<(), T>;
    fn read_all(&mut self) -> Result<Vec<u8>, T>;
    fn read_line(&mut self, max_len: Option<usize>) -> Result<String, T>;
}
