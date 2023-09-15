use std::sync::Arc;
use tokio::sync::Semaphore;
use usdt::UniqueId;

use crate::persist::{Persist, PERSIST_N};
use crate::probes;

pub struct DataProcessor {
    sem: Arc<Semaphore>,
    persist: Persist,
}
impl DataProcessor {
    pub fn new() -> Self {
        Self {
            sem: Arc::new(Semaphore::new(PERSIST_N)),
            persist: Persist::new(),
        }
    }
    pub async fn request(&self, id: &UniqueId) {
        // Wait until we can be sure we have a slot available in the persistent
        // store i.e. that the enqueue will succeed. We eliminate the
        // reservation and restore it when the transaction is complete.
        let sem = self.sem.clone();

        // We first try to acquire the semaphore in a non-blocking fashion
        // simply so that we can determine whether this is a blocking or
        // non-blocking request for the semaphore.
        match sem.try_acquire() {
            Ok(s) => {
                probes::request__nonblock!(|| id);
                s
            }
            Err(_) => {
                probes::request__block!(|| id);
                sem.acquire().await.unwrap()
            }
        }
        .forget();

        // Create a new ID for the enqueued work.
        let new_id = UniqueId::new();

        // Enqueue the transaction with the persistent store.
        let sem = self.sem.clone();
        self.persist.enqueue(new_id, async move {
            sem.add_permits(1);
        });

        // Return to the caller without waiting for the transaction to
        // complete.
    }
}
