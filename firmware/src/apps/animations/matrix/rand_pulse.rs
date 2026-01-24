use core::sync::atomic::Ordering;

use crate::drivers::matrix::{Framebuffer, Matrix};
use crate::util::rand;
use embassy_time::{Duration, Ticker};

const TARGET_BRIGHTNESS: u8 = 150;

const _: () = assert!(Framebuffer::WIDTH <= u8::BITS as usize);

pub struct Task {
    gen_ticker: Ticker,
    update_ticker: Ticker,
    rng: rand::Rng,
    target_buf: [u8; Framebuffer::HEIGHT],
}

impl Task {
    pub fn new() -> Self {
        Self {
            gen_ticker: Ticker::every(Duration::from_millis(300)),
            update_ticker: Ticker::every(Duration::from_millis(10)),
            rng: rand::Rng::new(),
            target_buf: Default::default(),
        }
    }

    pub fn poll(&mut self) {
        if self.gen_ticker.consume_expired() {
            let row = self.rng.rand8() as usize % Framebuffer::HEIGHT;
            let col = self.rng.rand8() as usize % Framebuffer::WIDTH;
            self.target_buf[row] |= 1 << col;
        }

        if self.update_ticker.consume_expired() {
            for (target_row, fb_row) in self.target_buf.iter_mut().zip(Matrix::fb().0.iter()) {
                for (col, fb_field) in fb_row.iter().enumerate() {
                    let fb_current = fb_field.load(Ordering::Relaxed);
                    let target = *target_row & (1 << col) != 0;

                    if target {
                        if fb_current < TARGET_BRIGHTNESS {
                            fb_field.store(fb_current + 1, Ordering::Relaxed);
                        } else {
                            *target_row &= !(1 << col);
                        }
                    } else if fb_current > 0 {
                        fb_field.store(fb_current - 1, Ordering::Relaxed);
                    }
                }
            }
        }
    }
}
