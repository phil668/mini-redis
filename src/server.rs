use std::sync::Arc;

use tokio::{net::TcpListener, sync::Semaphore};
// 实现一个server.run方法，可以传入一个TcpListener，能够获取到请求，对请求进行处理

struct Listener {
    listener: TcpListener,
    limit_connections: Arc<Semaphore>,
}

// 最大连接数
const MAX_CONNECTIONS: usize = 250;

pub fn run(listener: TcpListener) {
    let server = Listener {
        listener,
        limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
    };
}
