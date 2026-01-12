use core::sync::atomic::Ordering;

use crate::drivers::matrix::{Framebuffer, Matrix};
use crate::util::rand;
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Ticker};

const TARGET_BRIGHTNESS: u8 = 150;

pub async fn run() -> ! {
    let mut gen_clock = Ticker::every(Duration::from_millis(300));
    let mut update_clock = Ticker::every(Duration::from_millis(10));
    const {
        assert!(Framebuffer::WIDTH <= u8::BITS as usize);
    }
    let mut target_buf = [0u8; Framebuffer::HEIGHT];
    let mut noisegen = rand::WhiteNoiseGenerator::new();

    loop {
        match select(gen_clock.next(), update_clock.next()).await {
            Either::First(()) => {
                let row = noisegen.rand8() as usize % Framebuffer::HEIGHT;
                let col = noisegen.rand8() as usize % 8;
                target_buf[row] |= 1 << col;
            }
            Either::Second(()) => {
                for (target_row, fb_row) in target_buf.iter_mut().zip(Matrix::fb().0.iter()) {
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
}
