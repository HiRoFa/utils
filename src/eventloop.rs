use futures::executor::{block_on, LocalPool};
use futures::task::{LocalSpawnExt, SpawnExt};
use std::future::Future;

/// the EventLoop struct is a single threade event queue which also features scheduling
pub struct EventLoop {
    pool: LocalPool,
}

impl EventLoop {
    pub fn new() -> Self {
        let pool = LocalPool::new();
        Self { pool }
    }

    /// add a task to the pool
    pub fn add<T: FnOnce() -> R + Send>(&self, task: T) -> impl Future<Output = R> {
        self.pool.spawner().spawn_with_handle(task);
    }

    /// this executes a task and blocks untill the task is done
    /// this should never be called from the worker thread
    pub fn exe<T: FnOnce() -> R + Send>(&self, task: T) -> R {
        block_on(self.pool.spawner().spawn_with_handle())
    }
}

#[cfg(test)]
pub mod tests {
    #[test]
    fn test() {
        //
    }
}
