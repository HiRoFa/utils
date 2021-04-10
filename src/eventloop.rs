use futures::executor::{block_on, LocalPool, LocalSpawner};
use futures::task::{LocalSpawnExt, SpawnExt};
use std::cell::RefCell;
use std::future::Future;
use std::sync::mpsc::{channel, Sender};
use std::thread::JoinHandle;

/// the EventLoop struct is a single thread event queue
pub struct EventLoop {
    tx: Sender<Box<dyn FnOnce() + Send + 'static>>,
    join_handle: Option<JoinHandle<()>>,
}

thread_local! {
    static POOL: RefCell<LocalPool> = RefCell::new(LocalPool::new());
    static SPAWNER: RefCell<Option<LocalSpawner>> = RefCell::new(None);
}

impl EventLoop {
    /// init a new EventLoop
    pub fn new() -> Self {
        let (tx, rx) = channel();

        let join_handle = std::thread::spawn(move || {
            POOL.with(|rc| {
                let pool = &mut *rc.borrow_mut();

                SPAWNER.with(|rc| {
                    let opt = &mut *rc.borrow_mut();
                    let _ = opt.replace(pool.spawner());
                });
            });

            POOL.with(|rc| {
                let pool = &mut *rc.borrow_mut();
                let spawner = pool.spawner();
                loop {
                    let fut: Box<dyn FnOnce() + Send + 'static> =
                        rx.recv().ok().expect("recv failed");
                    // this seems redundant.. i could just run the task closure

                    spawner
                        .spawn(async move { fut() })
                        .ok()
                        .expect("spawn failed");
                    pool.run_until_stalled();

                    // shutdown que
                    if SPAWNER.with(|rc| rc.borrow().is_none()) {
                        log::debug!("EventLoop worker loop break");
                        break;
                    }
                }
                log::debug!("EventLoop worker loop done");
            })
        });

        Self {
            tx,
            join_handle: Some(join_handle),
        }
    }

    /// internal method to ensure a member is called from the worker thread
    fn assert_is_pool_thread() {
        debug_assert!(SPAWNER.with(|rc| { rc.borrow().is_some() }));
    }

    /// add a future to the EventLoop from within a running task
    pub fn add_local_future_void<F: Future<Output = ()> + 'static>(fut: F) {
        EventLoop::assert_is_pool_thread();
        SPAWNER.with(move |rc| {
            let spawner = &*rc.borrow();
            spawner
                .as_ref()
                .unwrap()
                .spawn_local(fut)
                .ok()
                .expect("start fut failed");
        });
    }

    /// add a future to the EventLoop from within a running task
    pub fn add_local_future<R: Send + 'static, F: Future<Output = R> + 'static>(
        fut: F,
    ) -> impl Future<Output = R> {
        EventLoop::assert_is_pool_thread();
        SPAWNER.with(move |rc| {
            let spawner = &*rc.borrow();
            spawner
                .as_ref()
                .unwrap()
                .spawn_local_with_handle(fut)
                .ok()
                .expect("start fut failed")
        })
    }

    /// add a task to the EventLoop from within a running task
    pub fn add_local_void<T: FnOnce() + 'static>(&self, task: T) {
        EventLoop::assert_is_pool_thread();
        Self::add_local_future_void(async move { task() });
    }

    /// add a task to the EventLoop
    pub fn add<T: FnOnce() -> R + Send + 'static, R: Send + 'static>(
        &self,
        task: T,
    ) -> impl Future<Output = R> {
        self.add_future(async move { task() })
    }

    /// execute a task in the EventLoop and block until it completes
    pub fn exe<R: Send + 'static, T: FnOnce() -> R + Send + 'static>(&self, task: T) -> R {
        block_on(self.add(task))
    }

    /// add an async block to the EventLoop
    /// #Example
    /// ```rust
    /// use hirofa_utils::eventloop::EventLoop;
    /// use futures::executor::block_on;
    /// let test_loop = EventLoop::new();
    /// let fut = test_loop.add_future(async move {
    ///    // this is an async block, you can .await async functions here
    ///    123
    /// });
    /// let res = block_on(fut); // get result
    /// assert_eq!(res, 123);
    /// ```
    pub fn add_future<R: Send + 'static, F: Future<Output = R> + Send + 'static>(
        &self,
        fut: F,
    ) -> impl Future<Output = R> {
        let (tx, rx) = channel();
        self.add_void(move || {
            let res_fut = Self::add_local_future(fut);
            tx.send(res_fut).ok().expect("send failed");
        });
        rx.recv().ok().expect("recv failed")
    }

    /// add a Future to the pool, for when you don't need the result
    /// #Example
    /// ```rust
    /// use hirofa_utils::eventloop::EventLoop;
    /// use futures::executor::block_on;
    /// use std::sync::mpsc::channel;
    /// let test_loop = EventLoop::new();
    /// let (tx, rx) = channel(); // just to see if it works
    /// let fut = test_loop.add_future(async move {
    ///    // this is an async block, you can .await async functions here
    ///    println!("running async here");
    ///    tx.send(1234);
    /// });
    ///
    /// let res = rx.recv().ok().expect("could not recv");
    /// assert_eq!(res, 1234);
    /// ```    
    pub fn add_future_void<F: Future<Output = ()> + Send + 'static>(&self, fut: F) {
        self.add_void(move || EventLoop::add_local_future_void(fut))
    }

    /// add a task to the pool
    pub fn add_void<T: FnOnce() + Send + 'static>(&self, task: T) {
        self.tx.send(Box::new(task)).ok().expect("send failed");
    }
}

impl Drop for EventLoop {
    fn drop(&mut self) {
        self.exe(|| {
            SPAWNER.with(|rc| {
                let spawner = &mut *rc.borrow_mut();
                let _ = spawner.take();
            })
        });
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::eventloop::EventLoop;
    use futures::executor::block_on;
    use std::sync::mpsc::channel;

    async fn test_as(input: i32) -> i32 {
        input * 12
    }

    #[test]
    fn test() {
        //

        let test_loop = EventLoop::new();

        let res = test_loop.exe(|| 123);
        assert_eq!(res, 123);

        let (tx, rx) = channel();
        test_loop.add_void(move || {
            tx.send("async".to_string()).ok().unwrap();
        });

        let res = rx.recv().ok().unwrap();
        assert_eq!(res.as_str(), "async");

        let i = 43;
        let fut = test_loop.add_future(async move { test_as(i).await });
        let out = block_on(fut);
        assert_eq!(43 * 12, out);

        log::debug!("dropping loop");
        drop(test_loop);
        log::debug!("after loop dropped");
    }
}
