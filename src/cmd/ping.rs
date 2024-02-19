use bytes::Bytes;

use crate::parse::{Parse, ParseError};

#[derive(Debug, Default)]
pub struct Ping {
    msg: Option<Bytes>,
}

impl Ping {
    fn new(msg: Option<Bytes>) -> Ping {
        Ping { msg }
    }

    pub fn parse_frame(parse: &mut Parse) -> crate::Result<Ping> {
        match parse.next_bytes() {
            Ok(msg) => Ok(Ping::new(Some(msg))),
            Err(ParseError::EndOfStream) => Ok(Ping::default()),
            Err(err) => Err(err.into()),
        }
    }
}
