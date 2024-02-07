use std::io::Cursor;

use bytes::{Buf, Bytes};

use std::string::FromUtf8Error;

#[derive(Debug, Clone)]
pub enum Frame {
    // Simple Strings（简单字符串）：以加号（+）开头，后跟字符串内容，以回车换行（\r\n）结束。
    // 单行字符串（Simple Strings）： 响应的首字节是 "+"
    // +OK\r\n
    Simple(String),
    // Errors（错误类型）：以减号（-）开头，后跟错误消息字符串，以回车换行（\r\n）结束。
    // 错误（Errors）： 响应的首字节是 "-"
    // -Error message\r\n
    Error(String),
    // Integers（整数）：以冒号（:）开头，后跟整数值的字符串表示，以回车换行（\r\n）结束。
    // 整型（Integers）： 响应的首字节是 ":"
    // :1000\r\n
    Integer(u64),
    // Bulk Strings（块字符串）：以美元符号（$）开头，后跟字符串长度的字符串表示，然后是实际字符串内容，以回车换行（\r\n）结束。
    // 多行字符串（Bulk Strings）： 响应的首字节是"$"
    // 美元符 "$" 后面跟着组成字符串的字节数(前缀长度)，并以 CRLF 结尾。
    // 实际的字符串数据。
    // 结尾是 CRLF。
    // $6\r\nfoobar\r\n
    Bulk(Bytes),
    Null,
    // Arrays（数组）：以星号（*）开头，后跟数组的长度的字符串表示，然后是数组中的元素，每个元素都遵循RESP协议的其他数据类型格式，以回车换行（\r\n）结束。
    // 以星号* 为首字符，接着是表示数组中元素个数的十进制数，最后以 CRLF 结尾。
    // 外加数组中每个 RESP 类型的元素
    // *2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n
    Array(Vec<Frame>),
}

#[derive(Debug)]
pub enum Error {
    Incomplete,
    Other(crate::Error),
}

impl Frame {
    // 检查src能否被完整的解码
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
        match get_u8(src)? {
            b'+' => {
                get_line(src)?;
                Ok(())
            }
            b'-' => {
                get_line(src)?;
                Ok(())
            }
            b':' => {
                get_decimal(src)?;
                Ok(())
            }
            // $5\r\nhello\r\n
            b'$' => {
                // redis协议中规定如果后面是‘-1\r\n’的话，认为是空的字符串，因此需要跳过这个情况
                if b'-' == peek_u8(src)? {
                    // skip -1\r\n
                    skip(src, 4)
                // 如果下一个字节不是'-'的话，说明是一个正常的二进制字符串，此时需要读取一个十进制的数据，该数值表示字符串的长度
                // 然后跳过相应的长度，以便读取正常的字符串数据
                } else {
                    // 获取二进制字符串的长度
                    let len = get_decimal(src)? as usize;
                    // 需要加上\r\n
                    skip(src, len + 2)
                }
            }
            // *3\r\n$3\r\nfoo\r\n$3\r\nbar\r\n$3\r\nbaz\r\n
            b'*' => {
                let len = get_decimal(src)?;
                for _ in 0..len {
                    Frame::check(src)?;
                }
                Ok(())
            }
            actual => Err(format!("protocol error;invalid frame byte type,{}", actual).into()),
        }
    }

    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Frame, Error> {
        match get_u8(src)? {
            // 简单字符串 直接读取字符串内容并返回
            b'+' => {
                let line = get_line(src)?.to_vec();
                let string = String::from_utf8(line)?;
                Ok(Frame::Simple(string))
            }
            // Error
            b'-' => {
                let line = get_line(src)?.to_vec();
                let string = String::from_utf8(line)?;
                Ok(Frame::Error(string))
            }
            // 数值
            b':' => {
                let value = get_decimal(src)?;
                Ok(Frame::Integer(value))
            }
            b'$' => {
                // redis协议中规定如果是-1/r/n的话 说明是空字符串
                if b'-' == peek_u8(src)? {
                    // todo
                    let line = get_line(src)?;
                    if line != b"-1" {
                        return Err("protocol error;Invalid frame format".into());
                    }

                    Ok(Frame::Null)
                } else {
                    // $6\r\nfoobar\r\n
                    let len = get_decimal(src)? as usize;
                    let start = len + 2;
                    if src.remaining() < start {
                        return Err(Error::Incomplete);
                    }
                    let data = Bytes::copy_from_slice(&src.chunk()[start..]);
                    skip(src, start)?;
                    Ok(Frame::Bulk(data))
                }
            }
            b'*' => {
                // *2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n
                let len = get_decimal(src)? as usize;
                let mut res = Vec::with_capacity(len);

                for _ in 0..len {
                    res.push(Frame::parse(src)?);
                }

                Ok(Frame::Array(res))
            }
            _ => {
                unimplemented!()
            }
        }
    }
}

fn get_decimal(src: &mut Cursor<&[u8]>) -> Result<u64, Error> {
    // 将字符串的转换成十进制的数据
    // 1234556hello => 1234556
    use atoi::atoi;

    let line = get_line(src)?;

    atoi::<u64>(line).ok_or_else(|| "protocol error:invalid frame format".into())
}

fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    let start = src.position() as usize;
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            src.set_position((i + 2) as u64);
            let line: &[u8] = &src.get_ref()[start..i];
            return Ok(line);
        }
    }

    Err(Error::Incomplete)
}

fn skip(src: &mut Cursor<&[u8]>, n: usize) -> Result<(), Error> {
    if src.remaining() < n {
        return Err(Error::Incomplete);
    }
    src.advance(n);
    Ok(())
}

fn peek_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }
    Ok(src.chunk()[0])
}

fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }
    Ok(src.get_u8())
}

impl From<String> for Error {
    fn from(value: String) -> Error {
        Error::Other(value.into())
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Error {
        value.to_string().into()
    }
}

impl From<FromUtf8Error> for Error {
    fn from(_src: FromUtf8Error) -> Error {
        "protocol error; invalid frame format".into()
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Incomplete => "stream ended early".fmt(f),
            Error::Other(err) => err.fmt(f),
        }
    }
}
