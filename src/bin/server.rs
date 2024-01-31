use clap::Parser;
use mini_redis::DEFAULT_PORT;

#[derive(Parser, Debug)]
#[command(name = "mini-redis-server", author, version, about = "A Redis Server")]
struct Args {
    /// Name of the persion to greet
    #[clap(long)]
    port: Option<u16>,
}

fn main() {
    let args = Args::parse();

    let port = args.port.unwrap_or(DEFAULT_PORT);

    let listen_url = format!("127.0.0.1:{}", port);

    println!("{}", listen_url);
}
