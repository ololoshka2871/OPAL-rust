use alloc::vec::Vec;

use prost::bytes::BufMut;
use prost::{EncodeError, Message};

pub fn encode_md_message(response: super::messages::Response) -> Result<Vec<u8>, EncodeError> {
    let size = response.encoded_len();

    let mut result = Vec::with_capacity(size + 1 + core::mem::size_of::<u64>());
    result.put_u8(super::messages::Info::Magick as u8);

    match response.encode_length_delimited(&mut result) {
        Ok(_) => Ok(result),
        Err(e) => Err(e),
    }
}
