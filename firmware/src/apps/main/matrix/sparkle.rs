use crate::drivers::matrix::{Framebuffer, Matrix};
use super::MatrixMode;
use crate::util::rand;
use core::sync::atomic::Ordering;
use embassy_time::{Duration, Ticker, Timer};

pub async fn run() {
    let mut ticker = Ticker::every(Duration::from_millis(10));
    let mut counter = 0;
    let mut noisegen = rand::WhiteNoiseGenerator::new();

    loop {
        for row in 0..Framebuffer::HEIGHT {
            for col in 0..Framebuffer::WIDTH {
                let cur: u8 = Matrix::fb().load(col, row);
                Matrix::fb().store(col, row, ((cur << 7) - cur) >> 7);
            }
        }
        if counter >= 0x20 {
            if noisegen.rand8() < 128 {
                let row = noisegen.rand8() as usize % Framebuffer::HEIGHT;
                let col = noisegen.rand8() as usize % Framebuffer::WIDTH;
                Matrix::fb().store(col, row, 255);
            }
            counter = 0;
        }
        counter += 1;
        ticker.next().await;
    }
}
