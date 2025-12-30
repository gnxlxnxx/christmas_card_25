use core::cell::{Cell};

use critical_section::Mutex;
use embassy_time_driver::{Driver, time_driver_impl};

time_driver_impl!(static TIME_DRIVER: TimeDriver = TimeDriver {
    now: Mutex::new(Cell::new(0)),
});

#[derive(Debug)]
pub(super) struct TimeDriver {
    now: Mutex<Cell<u64>>,
}

impl TimeDriver {
    pub(super) fn advance(&self) {
        critical_section::with(|cs| {
            let now_box = self.now.borrow(cs);

            let now = now_box.get() + 1;
            now_box.set(now);
        });
    }
}

impl Driver for TimeDriver {
    fn now(&self) -> u64 {
        critical_section::with(|cs| self.now.borrow(cs).get())
    }

    fn schedule_wake(&self, at: u64, waker: &core::task::Waker) {}
}

pub(super) fn time_driver() -> &'static TimeDriver {
    &TIME_DRIVER
}
