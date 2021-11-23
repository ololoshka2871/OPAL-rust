use my_proc_macro::c_str;
use nanopb_rs::{pb_decode::TIStream, Error};

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
