use tokio::sync::broadcast;

// 监听服务器的关闭信号
struct Shutdown {
    is_shutdown: bool,
    notify: broadcast::Receiver<()>,
}

impl Shutdown {
    fn run(notify: broadcast::Receiver<()>) -> Self {
        Shutdown {
            is_shutdown: false,
            notify,
        }
    }

    fn is_shutdown(&self) -> bool {
        self.is_shutdown
    }

    async fn recv(&mut self) {
        if self.is_shutdown() {
            return;
        }

        let _ = self.notify.recv().await;

        self.is_shutdown = true;
    }
}
