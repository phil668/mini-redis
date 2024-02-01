use std::{
    collections::{BTreeSet, HashMap},
    sync::{Arc, Mutex},
    time::Instant,
};

use bytes::Bytes;
use tokio::sync::{broadcast, Notify};

struct DbDropGuard {
    db: Db,
}

// db包含一个 hashmap，存储key-value数据
#[derive(Debug, Clone)]
struct Db {
    shared: Arc<Shared>,
}

#[derive(Debug)]
struct Shared {
    state: Mutex<State>,
    bacground_task: Notify,
}

#[derive(Debug)]
struct State {
    entries: HashMap<String, Entry>,
    pub_sub: HashMap<String, broadcast::Sender<Bytes>>,
    expirations: BTreeSet<(Instant, String)>,
    shutdowm: bool,
}

#[derive(Debug)]
struct Entry {
    data: Bytes,
    expires_at: Option<Instant>,
}
