use log::debug;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::persist::{Persist, PERSIST_N};

pub struct DataProcessor {
    sem: Arc<Semaphore>,
    id: AtomicU64,
    persist: Persist,
}
impl DataProcessor {
    pub fn new() -> Self {
        Self {
            sem: Arc::new(Semaphore::new(PERSIST_N)),
            id: AtomicU64::new(1024),
            persist: Persist::new(),
        }
    }
    pub async fn request(&self, id: u64) {
        // Wait until we can be sure we have a slot available in the persistent
        // store i.e. that the enqueue will succeed. We eliminate the
        // reservation and restore it when the transaction is complete.
        let sem = self.sem.clone();

        // We first try to acquire the semaphore in a non-blocking fashion
        // simply so that we can determine whether this is a blocking or
        // non-blocking request for the semaphore.
        match sem.try_acquire() {
            Ok(s) => {
                debug!("request is non-blocking, id = {}", id);
                s
            }
            Err(_) => {
                debug!("request is blocking, id = {}", id);
                sem.acquire().await.unwrap()
            }
        }
        .forget();

        // Create a new ID for the enqueued work.
        let new_id = self.id.fetch_add(1, Ordering::SeqCst);

        // Enqueue the transaction with the persistent store.
        let sem = self.sem.clone();
        self.persist.enqueue(new_id, async move {
            sem.add_permits(1);
        });

        // Return to the caller without waiting for the transaction to
        // complete.
    }
}
