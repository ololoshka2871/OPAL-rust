use nanopb_rs::{pb_decode::TIStream, Error};
use prost::Message;

pub fn recive_message_body1<T: TIStream>(mut is: T) -> Result<super::messages::Request, Error> {
    let size = is.stream().bytes_left;
    let data = is.stream().read(size)?;

    super::messages::Request::decode(data.as_slice()).map_err(|_| Error::from_str("Decode failed"))
}
