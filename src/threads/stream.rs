use alloc::vec::Vec;

pub trait Stream<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<(), T>;
    fn read_all(&mut self) -> Result<Vec<u8>, T>;
}
