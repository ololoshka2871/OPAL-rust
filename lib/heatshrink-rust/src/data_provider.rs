pub trait DataProvider {
    fn next(&mut self) -> Option<u8>;
}
