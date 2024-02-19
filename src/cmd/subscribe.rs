use crate::parse::{Parse, ParseError};

#[derive(Debug)]
pub struct Subscribe {
    channels: Vec<String>,
}

#[derive(Debug)]
pub struct Unsubscribe {
    channels: Vec<String>,
}

impl Subscribe {
    fn new(channels: Vec<String>) -> Subscribe {
        Subscribe { channels }
    }

    // SUBSCRIBE channel1 channel2
    pub fn parse_frame(parse: &mut Parse) -> crate::Result<Subscribe> {
        let mut channels = vec![parse.next_string()?];

        loop {
            match parse.next_string() {
                Ok(v) => channels.push(v),
                Err(ParseError::EndOfStream) => break,
                Err(err) => return Err(err.into()),
            }
        }

        Ok(Subscribe { channels })
    }
}

impl Unsubscribe {
    fn new(channels: Vec<String>) -> Unsubscribe {
        Unsubscribe { channels }
    }

    pub fn parse_frame(parse: &mut Parse) -> crate::Result<Unsubscribe> {
        let mut channels = vec![parse.next_string()?];

        loop {
            match parse.next_string() {
                Ok(v) => channels.push(v),
                Err(ParseError::EndOfStream) => break,
                Err(err) => return Err(err.into()),
            }
        }

        Ok(Unsubscribe { channels })
    }
}
