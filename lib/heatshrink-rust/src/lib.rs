#![no_std]

pub mod data_provider;
pub mod decoder;
pub mod encoder;

#[cfg(all(unix))]
#[macro_use]
extern crate std;

#[cfg(all(unix))]
#[cfg(test)]
mod tests {
    use crate::data_provider::DataProvider;
    use crate::decoder::HeatshrinkDecoder;
    use crate::encoder::HeatshrinkEncoder;

    use std::vec::Vec;

    struct ArrayReader {
        src: *const u8,
        offset: usize,
        size: usize,
    }

    impl crate::data_provider::DataProvider for ArrayReader {
        fn next(&mut self) -> Option<u8> {
            if self.offset < self.size {
                let res = unsafe { Some(*self.src.offset(self.offset as isize)) };
                self.offset += 1;
                res
            } else {
                None
            }
        }
    }

    impl ArrayReader {
        fn from<T>(src: &T) -> ArrayReader {
            ArrayReader {
                src: (src as *const T) as *const u8,
                offset: 0,
                size: std::mem::size_of::<T>(),
            }
        }
    }

    impl crate::data_provider::DataProvider for core::slice::Iter<'_, u8> {
        fn next(&mut self) -> Option<u8> {
            if let Some(_v) = Iterator::next(self) {
                Some(*_v)
            } else {
                None
            }
        }
    }

    #[test]
    fn buf_read() {
        let buff = [1u8, 2, 3];
        let mut provider = ArrayReader::from(&buff);

        assert_eq!(Some(1u8), provider.next());
        assert_eq!(Some(2u8), provider.next());
        assert_eq!(Some(3u8), provider.next());
        assert_eq!(None, provider.next());
    }

    #[test]
    fn encode_zeros() {
        let zeros = [0u8; 8];

        //data provider
        let mut provider = zeros.iter().map(|a| *a);
        let mut enc = HeatshrinkEncoder::from_source(&mut provider);

        //result
        assert_eq!(Some(0x0), enc.next());
        assert_eq!(Some(0x38), enc.next());
        assert_eq!(None, enc.next());
    }

    #[test]
    fn decode_zeros() {
        let input = [0u8, 0x38];

        //data provider
        let mut provider = input.iter().map(|a| *a);
        let mut dec = HeatshrinkDecoder::from_source(&mut provider);

        for _ in 0..8 {
            assert_eq!(Some(0u8), dec.next());
        }
        assert_eq!(None, dec.next());
    }

    #[test]
    fn enc_dec() {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        let src = (0..100)
            .map(|_| rng.gen_range(0u8..0xff))
            .collect::<Vec<u8>>();

        println!("=src: {}", src.len());

        let mut it_src = src.iter().map(|a| *a);

        let enc = HeatshrinkEncoder::from_source(&mut it_src);
        let encoded = enc.collect::<Vec<_>>();

        println!("=compressed: {}", encoded.len());

        let mut it_decoded = encoded.iter().map(|a| *a);

        let dec = HeatshrinkDecoder::from_source(&mut it_decoded);
        let decoded = dec.collect::<Vec<_>>();

        println!("=unpacked: {}", decoded.len());

        assert_eq!(src, decoded);
    }

    #[test]
    fn enc_dec_direct() {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        let src = (0..100)
            .map(|_| rng.gen_range(0u8..0xff))
            .collect::<Vec<u8>>();

        println!("=src: {}", src.len());

        let mut it_src = src.iter().map(|a| *a);

        let mut enc = HeatshrinkEncoder::from_source(&mut it_src);
        let dec = HeatshrinkDecoder::from_source(&mut enc);

        let decoded = dec.collect::<Vec<_>>();

        println!("=unpacked: {}", decoded.len());

        assert_eq!(src, decoded);
    }
}
