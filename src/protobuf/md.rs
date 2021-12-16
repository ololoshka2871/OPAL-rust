use alloc::vec;
use my_proc_macro::c_str;
use nanopb_rs::{pb_decode::TIStream, Error};

/*
pub fn recive_md_header<T: TIStream>(is: &mut T) -> Result<usize, Error> {
    match decode_magick(is) {
        Ok(_) => {}
        Err(e) => {
            is.stream().flush();
            return Err(e);
        }
    }

    match decode_msg_size(is) {
        Ok(s) => Ok(s),
        Err(e) => {
            is.stream().flush();
            Err(e)
        }
    }
}

pub fn decode_magick<T: TIStream>(is: &mut T) -> Result<(), Error> {
    match is.stream().decode_variant() {
        Ok(v) => {
            if v != super::messages::ru_sktbelpa_pressure_self_writer_INFO_MAGICK as u64 {
                Err(Error::from_str(c_str!("Invalid message magick!")))
            } else {
                Ok(())
            }
        }
        Err(e) => Err(e),
    }
}

pub fn decode_msg_size<T: TIStream>(is: &mut T) -> Result<usize, Error> {
    match is.stream().decode_variant() {
        Ok(v) => {
            if v == 0 || v > 1500 {
                Err(Error::from_str(c_str!("Invalid message length")))
            } else {
                Ok(v as usize)
            }
        }
        Err(e) => Err(e),
    }
}
*/
//------------------------------------------------------------------------------------

pub fn recive_md_header1<T: TIStream>(is: &mut T) -> Result<usize, Error> {
    decode_magick1(is).map_err(|e| {
        is.stream().flush();
        e
    })?;

    decode_msg_size1(is).map_err(|e| {
        is.stream().flush();
        e
    })
}

pub fn decode_magick1<T: TIStream>(is: &mut T) -> Result<(), Error> {
    let v = is.stream().read(1)?;
    if v[0] != super::messages::Info::Magick as u8 {
        Err(Error::from_str(c_str!("Invalid message magick!")))
    } else {
        Ok(())
    }
}

pub fn decode_msg_size1<T: TIStream>(is: &mut T) -> Result<usize, Error> {
    let mut data = vec![];
    for _ in 0..4 {
        let b = is.stream().read(1)?;
        data.push(b[0]);
        if b[0] < 0x80 {
            break;
        }
    }

    match prost::decode_length_delimiter(data.as_slice()) {
        Ok(v) => {
            if v == 0 || v > 1500 {
                Err(Error::from_str(c_str!("Invalid message length")))
            } else {
                Ok(v as usize)
            }
        }
        Err(_) => Err(Error::from_str(c_str!("Failed to decode len"))),
    }
}
