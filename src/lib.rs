// * ‘server’: 开启TcpListner监听，处理过来的请求
// * ‘client’ 向server发起请求，set，get等命令，可以拿到结果
// * 'command' 抽象出redis操作的各种命令
// * 'frame' 一个完整的redis请求抽象出来的结构体，类似于http中的header，body等
pub mod connection;
pub mod db;
pub mod frame;
pub mod server;

/// 默认端口
pub const DEFAULT_PORT: u16 = 6379;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;
