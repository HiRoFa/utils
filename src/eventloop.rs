use crate::auto_id_map::AutoIdMap;
use futures::executor::{block_on, LocalPool, LocalSpawner};
use futures::task::{LocalSpawnExt, SpawnExt};
use std::cell::RefCell;
use std::future::Future;
use std::ops::Add;
use std::sync::mpsc::{channel, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// the EventLoop struct is a single thread event queue
pub struct EventLoop {
    tx: Sender<Box<dyn FnOnce() + Send + 'static>>,
    join_handle: Option<JoinHandle<()>>,
}

struct Timeout {
    next_run: Instant,
    task: Box<dyn FnOnce()>,
}

struct Interval {
    next_run: Instant,
    interval: Duration,
    task: Box<dyn Fn()>,
}

thread_local! {
    static TIMEOUTS: RefCell<AutoIdMap<Timeout>> = RefCell::new(AutoIdMap::new());
    static INTERVALS: RefCell<AutoIdMap<Interval>> = RefCell::new(AutoIdMap::new());
    // impl timeout and interval tasks as two separate thread_locals, add a single method to add jobs for timeouts and intervals which returns a next)runt instant, that may be used for recv on next loop
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
                let mut next_deadline = Instant::now().add(Duration::from_secs(10));
                loop {
                    // todo use deadline when stabilized
                    let timeout = next_deadline.duration_since(Instant::now());
                    // recv may fail on timeout

                    let recv_res = rx.recv_timeout(timeout);
                    if recv_res.is_ok() {
                        let fut: Box<dyn FnOnce() + Send + 'static> = recv_res.ok().unwrap();
                        // this seems redundant.. i could just run the task closure

                        spawner
                            .spawn(async move { fut() })
                            .ok()
                            .expect("spawn failed");
                    }

                    pool.run_until_stalled();

                    // add jobs for timeout and interval here, recalc next timout deadline based on next pending timeout or interval
                    next_deadline = Self::run_timeouts_and_intervals();

                    // shutdown indicator
                    if SPAWNER.with(|rc| rc.borrow().is_none()) {
                        log::debug!("EventLoop worker loop break");
                        // drop all timeouts and intervals here
                        TIMEOUTS.with(|rc| rc.borrow_mut().clear());
                        INTERVALS.with(|rc| rc.borrow_mut().clear());
                        // then do run_until_stalled again so finalizers may run
                        pool.run_until_stalled();
                        // exit loop
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

    /// run scheduled tasks and calculate next deadline for running other tasks
    fn run_timeouts_and_intervals() -> Instant {
        // this is probably not very efficient when there are lots of timeouts, could be optimized by sorting based on next_run and thus not looping over future jobs
        let now = Instant::now();

        let next_deadline = TIMEOUTS.with(|rc| {
            let timeouts = &mut rc.borrow_mut();
            let todos = timeouts.remove_values(|timeout| timeout.next_run.lt(&now));
            for todo in todos {
                let task = todo.task;
                task();
            }
            let mut ret = now.add(Duration::from_secs(10));
            for timeout in timeouts.map.values() {
                if timeout.next_run.lt(&ret) {
                    ret = timeout.next_run;
                }
            }
            ret
        });

        let next_deadline = INTERVALS.with(|rc| {
            let intervals = &mut *rc.borrow_mut();
            let mut ret = next_deadline.clone();
            for interval in intervals.map.values_mut() {
                if interval.next_run.lt(&now) {
                    let task = &interval.task;
                    task();
                    interval.next_run = now.add(interval.interval);
                } else {
                    if interval.next_run.lt(&ret) {
                        ret = interval.next_run.clone();
                    }
                }
            }
            ret
        });

        next_deadline
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

    /// add a timeout (delayed task) to the EventLoop
    pub fn add_timeout<F: FnOnce() + 'static>(task: F, delay: Duration) -> usize {
        Self::assert_is_pool_thread();
        let timeout = Timeout {
            next_run: Instant::now().add(delay),
            task: Box::new(task),
        };
        TIMEOUTS.with(|rc| rc.borrow_mut().insert(timeout))
    }

    /// add an interval (repeated task) to the EventLoop
    pub fn add_interval<F: Fn() + 'static>(task: F, delay: Duration, interval: Duration) -> usize {
        Self::assert_is_pool_thread();
        let interval = Interval {
            next_run: Instant::now().add(delay),
            interval,
            task: Box::new(task),
        };
        INTERVALS.with(|rc| rc.borrow_mut().insert(interval))
    }

    /// cancel a previously added timeout
    pub fn clear_timeout(id: usize) {
        Self::assert_is_pool_thread();
        TIMEOUTS.with(|rc| {
            let map = &mut *rc.borrow_mut();
            if map.contains_key(&id) {
                let _ = map.remove(&id);
            }
        });
    }

    /// cancel a previously added interval
    pub fn clear_interval(id: usize) {
        Self::assert_is_pool_thread();
        INTERVALS.with(|rc| {
            let map = &mut *rc.borrow_mut();
            if map.contains_key(&id) {
                let _ = map.remove(&id);
            }
        });
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
    use std::ops::Add;
    use std::sync::mpsc::channel;
    use std::time::{Duration, Instant};

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

        let (tx, rx) = channel();
        let start = Instant::now();
        let _ = test_loop.add(move || {
            EventLoop::add_timeout(
                move || {
                    tx.send(129).ok().expect("send failed");
                },
                Duration::from_secs(2),
            );
        });
        let res = rx.recv().unwrap();
        assert_eq!(res, 129);
        // we should be at least 2 seconds further
        assert!(Instant::now().gt(&start.add(Duration::from_millis(1999))));
        // but certainly not 3
        assert!(Instant::now().lt(&start.add(Duration::from_millis(2999))));

        log::debug!("dropping loop");
        drop(test_loop);
        log::debug!("after loop dropped");
    }
}
