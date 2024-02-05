use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

use crate::frame::Frame;

// Connetction用于将Tcp流中的数据按照redis的协议，读取和写入为完整的Frame
pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    fn new(stream: TcpStream) -> Self {
        Connection {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(1024 * 4),
        }
    }

    fn parse_frame(&self) -> crate::Result<Option<Frame>> {
        Ok(None)
    }
}
