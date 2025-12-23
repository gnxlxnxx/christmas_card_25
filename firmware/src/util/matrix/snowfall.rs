use crate::drivers::matrix::{Framebuffer, Matrix};
use crate::util::matrix::MatrixMode;
use crate::util::rand;
use core::sync::atomic::Ordering;
use embassy_time::Timer;

// TODO:
// - Wind
// - Make fall timing more random

pub struct Snowfall {
    noisegen: rand::WhiteNoiseGenerator,
}

impl Snowfall {
    pub fn new() -> Self {
        let noisegen = rand::WhiteNoiseGenerator::new();

        Self { noisegen }
    }
}

impl MatrixMode for Snowfall {
    async fn animate(&mut self) {
        let bottom_full: bool = Matrix::fb().0[Framebuffer::HEIGHT - 1]
            .iter()
            .filter(|col| (**col).load(Ordering::Relaxed) == 0)
            .next()
            .is_none();

        if bottom_full {
            for col in 0..Framebuffer::WIDTH {
                Matrix::fb().store(col, Framebuffer::HEIGHT - 1, 0);
            }
        }

        for col in 0..Framebuffer::WIDTH {
            Matrix::fb().store(
                col,
                Framebuffer::HEIGHT - 1,
                Matrix::fb().load(col, Framebuffer::HEIGHT - 1)
                    | Matrix::fb().load(col, Framebuffer::HEIGHT - 1 - 1),
            );
            for row in (1..(Framebuffer::HEIGHT - 1)).rev() {
                Matrix::fb().store(col, row, Matrix::fb().load(col, row - 1));
            }
            Matrix::fb().store(col, 0, if self.noisegen.rand8() > 240 { 127 } else { 0 });
        }
        Timer::after_millis(1000).await;
    }
}
