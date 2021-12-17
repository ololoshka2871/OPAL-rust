use alloc::vec::Vec;

use nanopb_rs::Error;
use prost::bytes::BufMut;
use prost::Message;

pub fn encode_md_message1(response: super::messages::Response) -> Result<Vec<u8>, Error> {
    let size = response.encoded_len();

    let mut result = Vec::with_capacity(size + 1 + core::mem::size_of::<u64>());
    result.put_u8(super::messages::Info::Magick as u8);

    match response.encode_length_delimited(&mut result) {
        Ok(_) => Ok(result),
        Err(_) => Err(Error::from_str("Encode error")),
    }
}
