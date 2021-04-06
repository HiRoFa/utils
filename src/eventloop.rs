use futures::executor::{block_on, LocalPool, LocalSpawner};
use futures::task::{SpawnExt, LocalSpawnExt};
use std::future::Future;
use std::sync::Arc;

/// the EventLoop struct is a single thread event queue which also features scheduling
pub struct EventLoop {
    pool: LocalPool,
}

impl EventLoop {
    pub fn new() -> Self {
        let pool = LocalPool::new();
       Self { pool }
    }

    /// add a task to the pool from a task
    pub fn add_local_future_void<F: Future<Output = ()> + 'static>(&self, fut: F) {
        self.pool.spawner().spawn_local(fut).ok().expect("start fut failed");
    }

    pub fn add_local_void<T: FnOnce() + 'static>(&self, task: T) {
        self.add_local_future_void(async move {task()});
    }

    /// add a task to the pool
    pub fn add_future<R: Send + 'static, F: Future<Output = R> + Send + 'static>(&mut self, fut: F) -> impl Future<Output = R> {
        let ret = self.pool.spawner().spawn_with_handle(fut).ok().expect("start fut failed");
        self.pool.run_until_stalled();
        ret
    }



    pub fn add<R: Send + 'static, T: FnOnce() -> R + Send + 'static>(&mut self, task: T) -> impl Future<Output = R> {
        self.add_future(async move {task()})
    }



    /// this executes a task and blocks until the task is done
    /// this should never be called from the worker thread
    pub fn exe<R: Send + 'static, T: FnOnce() -> R + Send + 'static>(&mut self, task: T) -> R {
        block_on(self.add(task))
    }
}

#[cfg(test)]
pub mod tests {
    use crate::eventloop::EventLoop;
    use futures::task::LocalSpawnExt;

    #[test]
    fn test() {
        //

        let mut test_loop  = EventLoop::new();

        let res = test_loop.exe(|| {123});

        let loop_handle = test_loop.pool.spawner();
        test_loop.add(move || {
            println!("a");
            loop_handle.spawn_local(async {println!("b")});
            println!("c");
        });

        assert_eq!(res, 123);

    }
}
