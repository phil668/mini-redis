use std::{fmt, vec};

use bytes::Bytes;

use crate::frame::Frame;

use std::str;

pub struct Parse {
    parts: vec::IntoIter<Frame>,
}

#[derive(Debug)]
pub enum ParseError {
    EndOfStream,
    Other(crate::Error),
}

impl Parse {
    pub fn new(frame: Frame) -> Result<Parse, ParseError> {
        let array = match frame {
            Frame::Array(v) => v,
            _ => {
                return Err(format!("protocol error:expected array,got {:?}", frame).into());
            }
        };

        Ok(Parse {
            parts: array.into_iter(),
        })
    }

    fn next(&mut self) -> Result<Frame, ParseError> {
        self.parts.next().ok_or(ParseError::EndOfStream)
    }

    pub fn next_string(&mut self) -> Result<String, ParseError> {
        match self.next()? {
            Frame::Simple(s) => Ok(s),
            Frame::Bulk(data) => str::from_utf8(&data[..])
                .map(|v| v.to_string())
                .map_err(|_| "protocol error;invalid string".into()),
            frame => {
                return Err(format!("protocol error,expected string,got {:?}", frame).into());
            }
        }
    }

    fn next_bytes(&mut self) -> Result<Bytes, ParseError> {
        match self.next()? {
            Frame::Simple(s) => Ok(Bytes::from(s.into_bytes())),
            Frame::Bulk(data) => Ok(data),
            frame => {
                return Err(format!("protocol error,expected bytes,got {:?}", frame).into());
            }
        }
    }

    fn next_int(&mut self) -> Result<u64, ParseError> {
        use atoi::atoi;

        const MSG: &str = "protocol error;invalid number";

        match self.next()? {
            Frame::Integer(v) => Ok(v),
            Frame::Simple(s) => atoi::<u64>(s.as_bytes()).ok_or_else(|| MSG.into()),
            Frame::Bulk(data) => atoi::<u64>(&data).ok_or_else(|| MSG.into()),
            frame => {
                return Err(format!("protocol error,expected int frame,got {:?}", frame).into());
            }
        }
    }

    pub fn finish(&mut self) -> Result<(), ParseError> {
        if self.parts.next().is_none() {
            Result::Ok(())
        } else {
            Result::Err("protocol error; expected end of frame, but there was more".into())
        }
    }
}

// 将String类型转换为ParseError
impl From<String> for ParseError {
    fn from(value: String) -> Self {
        ParseError::Other(value.into())
    }
}

impl From<&str> for ParseError {
    fn from(value: &str) -> Self {
        ParseError::Other(value.into())
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::EndOfStream => "protocol error;unexpected of end of stream".fmt(f),
            ParseError::Other(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for ParseError {}
