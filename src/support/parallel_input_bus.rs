use embedded_hal::digital::v2::InputPin;

pub trait ParallelInputBus<T: InputPin> {
    type Output;
    fn get(&self) -> Result<Self::Output, T::Error>;
}

pub struct SimpleParallelInputBus<T: InputPin, const count: usize>(pub [T; count]);

impl<T: InputPin, const count: usize> ParallelInputBus<T> for SimpleParallelInputBus<T, count> {
    type Output = u8;

    fn get(&self) -> Result<Self::Output, <T as InputPin>::Error> {
        let mut res = 0;

        for (i, p) in self.0.iter().enumerate() {
            if p.is_high()? {
                res |= 1 << i;
            }
        }

        Ok(res)
    }
}
