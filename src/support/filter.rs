#![allow(dead_code)]

pub struct Filter<T: Default + Copy, const SIZE: usize> {
    data: [T; SIZE],
    writen: usize,
    wp: usize,
}

impl<T: Default + Copy, const SIZE: usize> Filter<T, SIZE> {
    pub fn new() -> Self {
        Self {
            data: [T::default(); SIZE],
            writen: 0,
            wp: 0,
        }
    }

    pub fn add(&mut self, val: T) {
        self.data[self.wp] = val;
        self.wp = if self.wp == SIZE - 1 { 0 } else { self.wp + 1 };
        if self.writen < SIZE {
            self.writen += 1;
        }
    }
}

impl<const SIZE: usize> Filter<u32, SIZE> {
    pub fn avarage(&self) -> u32 {
        let summ = self.data[..self.writen]
            .iter()
            .fold(0u64, |acc, v| acc + *v as u64);
        (summ / (self.writen as u64)) as u32
    }
}

impl<const SIZE: usize> Filter<f32, SIZE> {
    pub fn avarage(&self) -> f32 {
        self.data[..self.writen - 1].iter().sum::<f32>() / self.writen as f32
    }
}
