use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:3000").await.unwrap();
    let request = "GET /bbb HTTP/1.1\r\n\
                   Host: example.com\r\n\
                   \r\n";

    stream.write_all(request.as_bytes()).await;

    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await.unwrap();
    let response = String::from_utf8_lossy(&buffer[..n]);
    println!("response,{}", response);
}
