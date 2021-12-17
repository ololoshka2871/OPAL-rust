use core::fmt::Debug;

use prost::{DecodeError, Message};

use super::Stream;

pub fn recive_message_body<T: Debug, S: Stream<T>>(
    stream: &mut S,
) -> Result<super::messages::Request, DecodeError> {
    let data = stream
        .read_all()
        .map_err(|_| DecodeError::new("Failed to read message body"))?;

    super::messages::Request::decode(data.as_slice())
}
