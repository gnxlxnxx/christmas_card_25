use core::cell::{Cell, RefCell};

use critical_section::Mutex;
use embassy_time_driver::{Driver, time_driver_impl};
use embassy_time_queue_utils::Queue;

time_driver_impl!(static TIME_DRIVER: TimeDriver = TimeDriver {
    now: Mutex::new(Cell::new(0)),
    next: Mutex::new(Cell::new(0)),
    queue: Mutex::new(RefCell::new(Queue::new()))
});

#[derive(Debug)]
pub(super) struct TimeDriver {
    now: Mutex<Cell<u64>>,
    next: Mutex<Cell<u64>>,
    queue: critical_section::Mutex<RefCell<Queue>>,
}

impl TimeDriver {
    pub fn advance(&self) {
        critical_section::with(|cs| {
            let now_box = self.now.borrow(cs);
            let next_box = self.next.borrow(cs);

            let now = now_box.get() + 1;
            now_box.set(now);

            if next_box.get() <= now {
                next_box.set(self.queue.borrow_ref_mut(cs).next_expiration(now));
            }
        });
    }
}

impl Driver for TimeDriver {
    fn now(&self) -> u64 {
        critical_section::with(|cs| self.now.borrow(cs).get())
    }

    fn schedule_wake(&self, at: u64, waker: &core::task::Waker) {
        critical_section::with(|cs| {
            let mut queue = self.queue.borrow_ref_mut(cs);
            if queue.schedule_wake(at, waker) {
                self.next.borrow(cs).set(queue.next_expiration(self.now()));
            }
        });
    }
}

pub(super) fn time_driver() -> &'static TimeDriver {
    &TIME_DRIVER
}
