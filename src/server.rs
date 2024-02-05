use std::{future::Future, sync::Arc};

use tokio::{
    net::{TcpListener, TcpStream},
    sync::{broadcast, mpsc, Semaphore},
    time::{self, Duration},
};
use tracing::info;

use crate::{
    connection::Connection,
    db::{Db, DbDropGuard},
};
// 实现一个server.run方法，可以传入一个TcpListener，能够获取到请求，对请求进行处理

struct Listener {
    db_holder: DbDropGuard,
    listener: TcpListener,
    limit_connections: Arc<Semaphore>,
    notify_shutdown: broadcast::Sender<()>,
    shutdowm_complete_tx: mpsc::Sender<()>,
}

struct Handler {
    db: Db,
    connection: Connection,
}

// 最大连接数
const MAX_CONNECTIONS: usize = 250;

pub async fn run(listener: TcpListener, shutdown: impl Future) {
    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdowm_complete_tx, _) = mpsc::channel(1);

    let server: Listener = Listener {
        db_holder: DbDropGuard::new(),
        listener,
        limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
        notify_shutdown: notify_shutdown,
        shutdowm_complete_tx: shutdowm_complete_tx,
    };

    tokio::select! {
        res = server.run() => {
            todo!()
        }
        _ = shutdown => {
            info!("shuting down")
        }
    }
}

impl Listener {
    async fn run(&self) -> crate::Result<()> {
        // todo
        info!("accepting inbound connections");
        loop {
            let permit = self
                .limit_connections
                .clone()
                .acquire_owned()
                .await
                .unwrap();
            let socket = self.accept().await?;
        }

        Ok(())
    }
    // 开始接受tcpStream
    async fn accept(&self) -> crate::Result<TcpStream> {
        let mut backoff = 1;
        loop {
            match self.listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
                Err(e) => {
                    // 如果重试次数大于64的话，返回错误
                    if backoff > 64 {
                        return Err(e.into());
                    }
                }
            }

            time::sleep(Duration::from_secs(backoff)).await;

            backoff *= 2;
        }
    }
}
