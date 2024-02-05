use tokio::net::TcpListener;

use clap::Parser;
use mini_redis::{server, DEFAULT_PORT};

#[derive(Parser, Debug)]
#[command(name = "mini-redis-server", author, version, about = "A Redis Server")]
struct Args {
    /// Name of the persion to greet
    #[clap(long)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() -> mini_redis::Result<()> {
    let args = Args::parse();

    let port = args.port.unwrap_or(DEFAULT_PORT);

    let listen_url = format!("127.0.0.1:{}", port);

    // 监听listen_url
    let listner = TcpListener::bind(listen_url).await?;

    server::run(listner, async {});

    Ok(())
}
