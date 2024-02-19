use bytes::Bytes;

use crate::parse::Parse;

#[derive(Debug)]
pub struct Publish {
    channel: String,
    message: Bytes,
}

impl Publish {
    fn new(channel: impl ToString, message: Bytes) -> Publish {
        Publish {
            channel: channel.to_string(),
            message,
        }
    }

    // PUBLISH channel message
    pub fn parse_frame(parse: &mut Parse) -> crate::Result<Publish> {
        let channel = parse.next_string()?;
        let message = parse.next_bytes()?;
        Ok(Publish {
            channel: channel,
            message: message,
        })
    }
}
