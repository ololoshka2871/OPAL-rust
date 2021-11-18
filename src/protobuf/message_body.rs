use core::mem;

use nanopb_rs::{pb_decode::TIStream, Error};

use super::ru_sktbelpa_pressure_self_writer_Request;

pub fn recive_message_body<T: TIStream>(
    mut is: T,
) -> Result<ru_sktbelpa_pressure_self_writer_Request, Error> {
    let mut res: ru_sktbelpa_pressure_self_writer_Request =
        unsafe { mem::MaybeUninit::zeroed().assume_init() };
    match is
        .stream()
        .decode(&mut res, ru_sktbelpa_pressure_self_writer_Request::fields())
    {
        Ok(_) => Ok(res),
        Err(e) => {
            is.stream().flush();
            Err(e)
        }
    }
}
