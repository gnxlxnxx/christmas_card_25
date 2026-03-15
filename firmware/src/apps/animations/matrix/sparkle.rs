use core::sync::atomic::Ordering;

use crate::drivers::matrix::{Framebuffer, Matrix};
use crate::util::rand;
use embassy_time::{Duration, Ticker};

pub struct Task {
    gen_ticker: Ticker,
    update_ticker: Ticker,
    rng: rand::Rng,
}

impl Task {
    #[must_use]
    pub fn new() -> Self {
        Self {
            gen_ticker: Ticker::every(Duration::from_millis(256)),
            update_ticker: Ticker::every(Duration::from_millis(4)),
            rng: rand::Rng::new(),
        }
    }

    pub fn poll(&mut self) {
        if self.gen_ticker.consume_expired() && self.rng.rand8() < 128 {
            let row = self.rng.rand8() as usize % Framebuffer::HEIGHT;
            let col = self.rng.rand8() as usize % Framebuffer::WIDTH;
            Matrix::fb().store(col, row, 255);
        }

        if self.update_ticker.consume_expired() {
            for row in &Matrix::fb().0 {
                for field in row {
                    let cur = field.load(Ordering::Relaxed) as u32;
                    field.store((((cur << 7) - cur) >> 7) as u8, Ordering::Relaxed);
                }
            }
        }
    }
}
