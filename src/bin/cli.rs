use clap::Parser;

#[derive(Parser, Debug)]
#[command(name="mini-redis-server",author, version, about="A Redis Server", long_about = None)]
struct Args {
    /// Name of the persion to greet
    #[arg(short, long)]
    name: String,

    /// Count of the greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {
    let args = Args::parse();

    for _ in 0..args.count {
        println!("hello,{}", args.name)
    }
}
