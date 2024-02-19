use crate::{frame::Frame, parse::Parse};

use get::Get;
use ping::Ping;
use publish::Publish;
use set::Set;
use subscribe::{Subscribe, Unsubscribe};
use unknown::Unknown;

mod get;
mod ping;
mod publish;
mod set;
mod subscribe;
mod unknown;

#[derive(Debug)]
pub enum Command {
    Get(Get),
    Publish(Publish),
    Set(Set),
    Subscribe(Subscribe),
    Unsubscribe(Unsubscribe),
    Ping(Ping),
    Unknown(Unknown),
}

impl Command {
    fn from_frame(frame: Frame) -> crate::Result<Command> {
        let mut parse = Parse::new(frame)?;

        let command_name = parse.next_string()?.to_lowercase();

        let command = match &command_name[..] {
            "get" => Command::Get(Get::parse_frame(&mut parse)?),
            "publish" => Command::Publish(Publish::parse_frame(&mut parse)?),
            "set" => Command::Set(Set::parse_frame(&mut parse)?),
            "subscribe" => Command::Subscribe(Subscribe::parse_frame(&mut parse)?),
            "unsubscribe" => Command::Unsubscribe(Unsubscribe::parse_frame(&mut parse)?),
            "ping" => Command::Ping(Ping::parse_frame(&mut parse)?),
            _ => {
                return Ok(Command::Unknown(Unknown::new(command_name)));
            }
        };

        parse.finish()?;

        Ok(command)
    }
}
