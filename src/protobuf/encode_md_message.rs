use alloc::vec;
use alloc::vec::Vec;
use my_proc_macro::c_str;
use nanopb_rs::{Error, OStream};

use crate::protobuf::messages::ru_sktbelpa_pressure_self_writer_INFO_MAGICK;

use super::ru_sktbelpa_pressure_self_writer_Response;

use super::sizable::Sizable;

pub fn encode_md_message(
    response: ru_sktbelpa_pressure_self_writer_Response,
) -> Result<Vec<u8>, Error> {
    let size = ru_sktbelpa_pressure_self_writer_Response::get_size(&response);

    let mut result = vec![0_u8; size + 1 + core::mem::size_of::<u64>()];
    let buf = result.as_mut_slice();
    let mut os = OStream::from_buffer(buf);

    if let Err(_) = os
        .stream()
        .write(&[ru_sktbelpa_pressure_self_writer_INFO_MAGICK])
    {
        return Err(Error::from_str(c_str!("Failed to write magick")));
    }

    if let Err(_) = os.stream().encode_varint(size as u64) {
        return Err(Error::from_str(c_str!("Failed to encode size\0")));
    }

    if let Err(e) = os
        .stream()
        .encode::<ru_sktbelpa_pressure_self_writer_Response>(
            ru_sktbelpa_pressure_self_writer_Response::fields(),
            &response,
        )
    {
        return Err(e);
    }
    // Умеьшить размер вектора в соответствии стем сколько буйт действительно было записано
    result.resize(os.stram_size(), 0);

    Ok(result)
}
