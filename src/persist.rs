use crate::probes;
use futures::Future;
use rand_distr::{Distribution, Normal};
use std::{sync::Arc, time::Duration};
use tokio::sync::Semaphore;
use usdt::UniqueId;

// The persistent store can handle at most `PERSIST_N` concurrent operations.
// More than that will cause a panic.
pub struct Persist {
    rng: Normal<f64>,
    sem: Arc<Semaphore>,
}

pub const PERSIST_N: usize = 8;

impl Persist {
    pub fn new() -> Self {
        Self {
            rng: Normal::new(10_000.0, 5_000.0).unwrap(),
            sem: Arc::new(Semaphore::new(PERSIST_N)),
        }
    }

    pub fn enqueue(&self, id: UniqueId, handler: impl Future + Send + 'static) {
        probes::persist__start!(|| &id);
        // This will panic if there are no leases available and this is by
        // design.
        self.sem.try_acquire().unwrap().forget();
        let d = self.rng.sample(&mut rand::thread_rng()) as u64;
        let delta = Duration::from_micros(50_000 + d);
        let sem = self.sem.clone();
        // Asynchronously delay for the time required to complete a transaction
        // and then call the completion handler.
        tokio::task::spawn(async move {
            tokio::time::sleep(delta).await;
            probes::persist__done!(|| &id);
            sem.add_permits(1);
            handler.await;
        });
    }
}
