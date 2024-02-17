use crate::{frame::Frame, parse::Parse};

use get::Get;

mod get;

#[derive(Debug)]
pub enum Command {
    Get(Get),
    Publish,
    Set,
    Subscribe,
    Unsubscribe,
    Ping,
    Unknown,
}

impl Command {
    fn from_frame(frame: Frame) -> crate::Result<Command> {
        let mut parse = Parse::new(frame)?;

        let command_name = parse.next_string()?.to_lowercase();

        let command = match &command_name[..] {
            "get" => Command::Get(Get::parse_frame(&mut parse)?),
            "publish" => Command::Publish,
            "set" => Command::Set,
            "subscribe" => Command::Subscribe,
            "unsubscribe" => Command::Unsubscribe,
            "ping" => Command::Ping,
            _ => {
                return Ok(Command::Unknown);
            }
        };

        parse.finish()?;

        Ok(command)
    }
}
