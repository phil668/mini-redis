use std::io::Cursor;

use bytes::{Buf, BytesMut};
use tokio::{
    io::{AsyncReadExt, BufWriter},
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
}
