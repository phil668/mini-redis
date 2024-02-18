use bytes::Bytes;
use std::time::Duration;

use crate::parse::{Parse, ParseError};

#[derive(Debug)]
pub struct Set {
    key: String,
    value: Bytes,
    expire: Option<Duration>,
}

impl Set {
    fn new(key: impl ToString, value: Bytes, expire: Option<Duration>) -> Set {
        Set {
            key: key.to_string(),
            value,
            expire,
        }
    }

    fn key(&self) -> &str {
        &self.key
    }

    fn value(&self) -> &Bytes {
        &self.value
    }

    fn expire(&self) -> Option<Duration> {
        self.expire
    }

    // SET key value [EX seconds|PX milliseconds]
    pub fn parse_frame(parse: &mut Parse) -> crate::Result<Set> {
        let key = parse.next_string()?;
        let value = parse.next_bytes()?;
        let mut expire: Option<Duration> = None;
        match parse.next_string() {
            Ok(s) if s.to_uppercase() == "EX" => {
                let secs = parse.next_int()?;
                expire = Some(Duration::from_secs(secs));
            }
            Ok(s) if s.to_uppercase() == "PX" => {
                let ms = parse.next_int()?;
                expire = Some(Duration::from_millis(ms));
            }
            Ok(_) => return Err("currently set only supports expire".into()),
            Err(ParseError::EndOfStream) => {}
            Err(err) => return Err(err.into()),
        }
        Ok(Set { key, value, expire })
    }
}
