use crate::drivers::matrix::{Framebuffer, Matrix};
use crate::util::rand;
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Ticker};

pub async fn run() {
    let mut update_clock = Ticker::every(Duration::from_millis(4));
    let mut gen_clock = Ticker::every(Duration::from_millis(256));
    let mut noisegen = rand::WhiteNoiseGenerator::new();

    loop {
        match select(gen_clock.next(), update_clock.next()).await {
            Either::First(()) => {
                if noisegen.rand8() < 128 {
                    let row = noisegen.rand8() as usize % Framebuffer::HEIGHT;
                    let col = noisegen.rand8() as usize % Framebuffer::WIDTH;
                    Matrix::fb().store(col, row, 255);
                }
            }
            Either::Second(()) => {
                for row in 0..Framebuffer::HEIGHT {
                    for col in 0..Framebuffer::WIDTH {
                        let cur = Matrix::fb().load(col, row) as u32;
                        Matrix::fb().store(col, row, (((cur << 7) - cur) >> 7) as u8);
                    }
                }
            }
        }
    }
}
