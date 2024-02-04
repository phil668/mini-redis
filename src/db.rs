use std::{
    collections::{BTreeSet, HashMap},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use bytes::Bytes;
use tokio::sync::{broadcast, Notify};

struct DbDropGuard {
    db: Db,
}

// db包含一个 hashmap，存储key-value数据
// 当db被new创建时，会通过tokio的spawn开启一个线程：backgro_task,这个线程是用来
// 将用户设置了过期时间的key-value清楚的，这个线程会持续运行到Db被drop之前
#[derive(Debug, Clone)]
struct Db {
    shared: Arc<Shared>,
}

#[derive(Debug)]
struct Shared {
    // state用于存储key-value，加Mutex的原因是因为tokio的异步可能会在多个线程中同时处理state
    // 因此需要通过Mutex锁处理竞态的情况
    state: Mutex<State>,
    // 通知后台任务处理过期的redis条目，background_task会一直等待直到被通知，检查是否过期或者关闭信号
    // Notify是tokio中用于实现异步通信机制
    // 我们可以使用notify.notified在一个子线程中等待著线程的通知，主线程可以使用notify2.notify_one()来发送一个通知
    bacground_task: Notify,
}

#[derive(Debug)]
struct State {
    // key-value的数据结构，hashmap，管理用户设置的redis数据
    entries: HashMap<String, Entry>,
    // 管理通知者和订阅者
    pub_sub: HashMap<String, broadcast::Sender<Bytes>>,
    // 用于存储每个key值的time to live
    // background_task 会去遍历BTreeSet，找到过期的值
    // 有可能同一个时间会被创建多个过期时间，因此还需要通过一个唯一的key值来处理
    expirations: BTreeSet<(Instant, String)>,
    // 当Db被drop的时候，shutdown的值会变成true，用于通知background_task退出
    shutdowm: bool,
}

#[derive(Debug)]
struct Entry {
    // 存储的值
    data: Bytes,
    // 设置的过期时间
    expires_at: Option<Instant>,
}

impl DbDropGuard {
    fn new() -> Self {
        DbDropGuard { db: Db::new() }
    }

    fn db(&self) -> Db {
        self.db.clone()
    }
}

impl Db {
    fn new() -> Db {
        let shared: Arc<Shared> = Arc::new(Shared {
            state: Mutex::new(State {
                entries: HashMap::new(),
                pub_sub: HashMap::new(),
                expirations: BTreeSet::new(),
                shutdowm: false,
            }),
            bacground_task: Notify::new(),
        });

        // 开启后台清楚过期的background_task
        tokio::spawn(purge_expired_tasks(Arc::clone(&shared)));

        Db { shared }
    }

    fn get(&self, key: &str) -> Option<Bytes> {
        let state = self.shared.state.lock().unwrap();
        state.entries.get(key).map(|item| item.data.clone())
    }

    fn set(&self, key: String, value: Bytes, expire: Option<Duration>) {
        //通过Mutex获取state
        let mut state = self.shared.state.lock().unwrap();

        // 任务是否需要被通知 是在set的过程中计算出来的

        let mut notify: bool = false;

        // 过期时间为调用设置时的时间 加上用户设置的duration
        let expires_at: Option<Instant> = expire.map(|duration| {
            let when = Instant::now() + duration;

            // 只有当下一个过期时间大于当前时间，才需要通知background_task去清楚
            notify = state
                .next_expiration()
                .map(|expiration| expiration > when)
                .unwrap_or(true);

            when
        });

        // HaspMap如果insert的key之前有值，会更新这个key对应的value，并将之前的value返回
        let prev = state.entries.insert(
            key.clone(),
            Entry {
                data: value,
                expires_at,
            },
        );
    }
}

impl Shared {
    fn purge_expired_keys(&self) -> Option<Instant> {
        let mut state = self.state.lock().unwrap();

        // 如果shutdown为true，证明db已经关闭
        if state.shutdowm {
            return None;
        }

        let now = Instant::now();

        // lock（）返回的是MutexGuard并不是对state的可变引用，但我们后续的while中需要使用state的可变引用
        // 使用*操作符，解引用出互斥锁内的数据，然后通过&mut创建其内部数据的可变引用
        let state = &mut *state;

        while let Some(&(when, ref key)) = state.expirations.iter().next() {
            // 比较b tree树中的时间和当前时间
            if when > now {
                return Some(when);
            }

            // key过期，remove1
            state.entries.remove(key);
            state.expirations.remove(&(when, key.clone()));
        }

        None
    }

    fn is_shutdown(&self) -> bool {
        self.state.lock().unwrap().shutdowm
    }
}

impl State {
    fn next_expiration(&self) -> Option<Instant> {
        self.expirations
            .iter()
            .next()
            .map(|expiration| expiration.0)
    }
}

// 清除过期的任务
async fn purge_expired_tasks(shared: Arc<Shared>) {
    while !shared.is_shutdown() {}
}
