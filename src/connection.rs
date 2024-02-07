use std::io::{self, Cursor, Write};

use bytes::{Buf, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

use crate::frame::{self, Frame};

// Connetction用于将Tcp流中的数据按照redis的协议，读取和写入为完整的Frame
pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Connection {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(1024 * 4),
        }
    }

    async fn read_frame(&mut self) -> crate::Result<Option<Frame>> {
        //
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            // 如果没有读取出frame，说明可能是buffer缓冲区数据不足 尝试从stream中读更多的数据到缓冲区内
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection per by reset".into());
                }
            }
        }
    }

    fn parse_frame(&mut self) -> crate::Result<Option<Frame>> {
        use frame::Error::Incomplete;
        // Cursor 用于记录现在读取到buffer中的哪个位置
        let mut buf = Cursor::new(&self.buffer[..]);

        match Frame::check(&mut buf) {
            Ok(_) => {
                let len = buf.position() as usize;

                buf.set_position(0);

                let frame = Frame::parse(&mut buf)?;

                self.buffer.advance(len);

                return Ok(Some(frame));
            }
            Err(Incomplete) => {
                return Ok(None);
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    async fn write_frame(&mut self, frame: &Frame) -> io::Result<()> {
        match frame {
            Frame::Array(v) => {
                let len = v.len();

                self.stream.write_u8(b'*').await?;
                self.write_decimal(len as u64).await?;

                for item in v {
                    self.write_value(item).await?;
                }
            }
            _ => {
                self.write_value(frame).await?;
            }
        }

        self.stream.flush().await
    }

    async fn write_value(&mut self, frame: &Frame) -> io::Result<()> {
        match frame {
            Frame::Simple(v) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(v.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Error(v) => {
                self.stream.write_u8(b'-').await?;
                self.stream.write_all(v.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Integer(v) => {
                self.stream.write_u8(b':').await?;
                self.write_decimal(*v).await?;
            }
            Frame::Bulk(v) => {
                self.stream.write_u8(b'$').await?;
                self.write_decimal(v.len() as u64).await?;
                self.stream.write_all(v).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Array(_) => unreachable!(),
            _ => {
                unimplemented!()
            }
        }
        Ok(())
    }

    async fn write_decimal(&mut self, v: u64) -> io::Result<()> {
        // 写入一个数字到stream中
        let mut buf = [0u8; 20];
        let mut buf = Cursor::new(&mut buf[..]);

        write!(&mut buf, "{}", v)?;

        self.stream.write_all(&buf.get_ref()[..]).await?;
        self.stream.write_all(b"\r\n").await?;
        Ok(())
    }
}
