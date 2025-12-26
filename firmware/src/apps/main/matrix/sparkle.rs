use crate::drivers::matrix::{Framebuffer, Matrix};
use super::MatrixMode;
use crate::util::rand;
use core::sync::atomic::Ordering;
use embassy_time::Timer;

pub struct Sparkle {
    counter: u32,
    noisegen: rand::WhiteNoiseGenerator,
}

impl Sparkle {
    pub fn new() -> Self {
        let noisegen = rand::WhiteNoiseGenerator::new();
        let counter = 0;
        Self { noisegen, counter }
    }
}

impl MatrixMode for Sparkle {
    async fn animate(&mut self) {
        for row in 0..Framebuffer::HEIGHT {
            for col in 0..Framebuffer::WIDTH {
                let cur: u8 = Matrix::fb().load(col, row);
                Matrix::fb().store(col, row, ((cur << 7) - cur) >> 7);
            }
        }
        if self.counter >= 0x20 {
            if self.noisegen.rand8() < 128 {
                let row = self.noisegen.rand8() as usize % Framebuffer::HEIGHT;
                let col = self.noisegen.rand8() as usize % Framebuffer::WIDTH;
                Matrix::fb().store(col, row, 255);
            }
            self.counter = 0;
        }
        self.counter += 1;
        Timer::after_millis(10).await;
    }
}
