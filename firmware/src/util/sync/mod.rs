use core::{
    cell::Cell,
    future::poll_fn,
    sync::atomic::{AtomicBool, Ordering},
    task::Poll,
};

use embassy_sync::blocking_mutex::{Mutex, raw::RawMutex};

pub struct Event(AtomicBool);

impl Event {
    pub const fn new() -> Self {
        Self(AtomicBool::new(false))
    }

    pub fn trigger(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn is_triggered(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    pub fn wait(&self) -> impl Future<Output = ()> + Send + Sync {
        poll_fn(|_| {
            if self.is_triggered() {
                self.0.store(false, Ordering::Relaxed);

                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
    }
}

pub struct Signal<M: RawMutex, T>(Mutex<M, Cell<Poll<T>>>);

impl<M: RawMutex, T> Signal<M, T> {
    pub const fn new() -> Self {
        Self(Mutex::new(Cell::new(Poll::Pending)))
    }

    pub fn signal(&self, val: T) {
        self.0.lock(|c| c.set(Poll::Ready(val)));
    }

    pub fn reset(&self) {
        self.0.lock(|c| c.set(Poll::Pending));
    }

    pub fn wait(&self) -> impl Future<Output = T> {
        poll_fn(|_| self.0.lock(|c| c.replace(Poll::Pending)))
    }

    pub fn try_take(&self) -> Option<T> {
        match self.0.lock(|c| c.replace(Poll::Pending)) {
            Poll::Pending => None,
            Poll::Ready(val) => Some(val),
        }
    }

    pub fn signaled(&self) -> bool {
        self.0.lock(|c| {
            let poll = c.replace(Poll::Pending);
            let res = poll.is_ready();
            c.set(poll);

            res
        })
    }
}

pub fn poll_while(mut f: impl FnMut() -> bool) -> impl Future<Output = ()> {
    poll_fn(move |_| if f() { Poll::Pending } else { Poll::Ready(()) })
}
