use core::sync::atomic::{AtomicI32, Ordering};

use embassy_time_driver::{Driver, time_driver_impl};

time_driver_impl!(static TIME_DRIVER: TimeDriver = TimeDriver {
    now: AtomicI32::new(0),
});

#[derive(Debug)]
pub(super) struct TimeDriver {
    now: AtomicI32,
}

impl TimeDriver {
    pub(super) fn advance(&self) {
        let now = self.now.load(Ordering::Relaxed);
        self.now.store(now.wrapping_add(1), Ordering::Relaxed);
    }
}

impl Driver for TimeDriver {
    fn now(&self) -> i32 {
        self.now.load(Ordering::Relaxed)
    }
}

pub(super) fn time_driver() -> &'static TimeDriver {
    &TIME_DRIVER
}
