use std::{
    collections::{BTreeSet, HashMap},
    sync::{Arc, Mutex},
    time::Duration,
};

use bytes::Bytes;
use tokio::sync::{broadcast, Notify};
use tokio::time::{sleep_until, Instant};

pub struct DbDropGuard {
    db: Db,
}

// db包含一个 hashmap，存储key-value数据
// 当db被new创建时，会通过tokio的spawn开启一个线程：backgro_task,这个线程是用来
// 将用户设置了过期时间的key-value清楚的，这个线程会持续运行到Db被drop之前
#[derive(Debug, Clone)]
pub struct Db {
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
    pub fn new() -> Self {
        DbDropGuard { db: Db::new() }
    }

    fn db(&self) -> Db {
        self.db.clone()
    }
}

impl Drop for DbDropGuard {
    fn drop(&mut self) {
        self.db.shut_down_purge()
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

        if let Some(prev) = prev {
            // 如果之前hashMap存储的值有expirea_at的话，需要将expirations内对应的值也清除掉
            if let Some(when) = prev.expires_at {
                state.expirations.remove(&(when, key.clone()));
            }
        }

        // 如果set的时候传递了过期时间的话，需要在expireation的BTreeSet中设置相应的key和过期时间
        if let Some(when) = expires_at {
            state.expirations.insert((when, key.clone()));
        }

        // 在设置完haspMap以及Btreeset之后将互斥锁释放掉
        drop(state);

        if notify {
            // 如果需要notify，即过期时间已经大于现在的时间，就通知后台线程去清理过期的key
            self.shared.bacground_task.notify_one()
        }
    }

    // 订阅一个值，返回一个boardcast的receiver，通过这个recivier可以获取到这个值变化
    fn subscribe(&self, key: String) -> broadcast::Receiver<Bytes> {
        use std::collections::hash_map::Entry;
        // todo
        let mut state = self.shared.state.lock().unwrap();
        match state.pub_sub.entry(key) {
            Entry::Occupied(v) => v.get().subscribe(),
            Entry::Vacant(v) => {
                // 如果没有需要新建一个boardcast
                let (tx, rx) = broadcast::channel(1024);
                v.insert(tx);
                rx
            }
        }
    }

    // 发布消息 ，让所有订阅者进行接收，哪些值改动了
    // 返回订阅者的数量
    fn publish(&self, key: &str, value: Bytes) -> usize {
        let state = self.shared.state.lock().unwrap();
        state
            .pub_sub
            .get(key)
            .map(|tx| tx.send(value).unwrap_or(0))
            .unwrap_or(0)
    }

    // 当db被drop时，需要通知后台清除所有的key
    fn shut_down_purge(&self) {
        let mut state = self.shared.state.lock().unwrap();

        state.shutdowm = true;

        drop(state);

        self.shared.bacground_task.notify_one();
    }
}

impl Shared {
    // 清除所有过期的key
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

        // 迭代循环expireations，将所有过期的key全部清除掉
        while let Some(&(when, ref key)) = state.expirations.iter().next() {
            // 比较b tree树中的时间和当前时间
            if when > now {
                return Some(when);
            }

            // key过期，remove
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

// 后台持续运行的任务，用于清除过期的key
async fn purge_expired_tasks(shared: Arc<Shared>) {
    while !shared.is_shutdown() {
        // 如果purge_expired_keys返回了时间，说明暂时还没有过期的值
        if let Some(when) = shared.purge_expired_keys() {
            tokio::select! {
                _ = sleep_until(when) => {}
                _ = shared.bacground_task.notified() => {}
            }
            // 如果没有返回时间，说明没有即将过期的key了
        } else {
            // 等待其他线程的通知
            shared.bacground_task.notified().await;
        }
    }

    dbg!("Purege background task shut down");
}
