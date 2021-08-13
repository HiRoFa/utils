use crate::debug_mutex::DebugMutex;
use futures::task::{Context, Poll, Waker};
use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc::{sync_channel, Receiver, SendError, SyncSender};
use std::sync::Arc;

pub struct ResolvableFutureResolver<R> {
    sender: SyncSender<R>,
    waker: DebugMutex<Option<Waker>>,
}

impl<R> ResolvableFutureResolver<R> {
    pub fn new(tx: SyncSender<R>) -> Self {
        Self {
            sender: tx,
            waker: DebugMutex::new(None, "TaskFutureResolver::waker"),
        }
    }
    pub fn resolve(&self, resolution: R) -> Result<(), SendError<R>> {
        log::trace!("TaskFutureResolver.resolve");
        self.sender.send(resolution)?;

        let waker_opt = &mut *self.waker.lock("resolve2").unwrap();
        if let Some(waker) = waker_opt.take() {
            waker.wake();
        }
        Ok(())
    }
}

pub struct ResolvableFuture<R> {
    result: Receiver<R>,
    resolver: Arc<ResolvableFutureResolver<R>>,
}
impl<R> ResolvableFuture<R> {
    pub fn new() -> Self {
        let (tx, rx) = sync_channel(1);

        Self {
            result: rx,
            resolver: Arc::new(ResolvableFutureResolver::new(tx)),
        }
    }
    pub fn get_resolver(&self) -> Arc<ResolvableFutureResolver<R>> {
        self.resolver.clone()
    }
}
impl<R> Default for ResolvableFuture<R> {
    fn default() -> Self {
        Self::new()
    }
}
impl<R> Future for ResolvableFuture<R> {
    type Output = R;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        log::trace!("TaskFuture::poll");
        match self.result.try_recv() {
            Ok(res) => {
                log::trace!("TaskFuture::poll -> Ready");
                Poll::Ready(res)
            }
            Err(_) => {
                log::trace!("TaskFuture::poll -> Pending");
                let mtx = &self.resolver.waker;
                let waker_opt = &mut *mtx.lock("poll").unwrap();
                let _ = waker_opt.replace(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}
