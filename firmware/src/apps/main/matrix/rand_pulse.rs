use crate::drivers::matrix::{Framebuffer, Matrix};
use super::MatrixMode;
use crate::util::rand;
use core::sync::atomic::Ordering;
use embassy_time::{Duration, Ticker, Timer};

pub struct RandPulse {
    ticker: Ticker,
    counter: u8,
    buffer_matrix: [[u8; Framebuffer::WIDTH]; Framebuffer::HEIGHT],
    noisegen: rand::WhiteNoiseGenerator,
}

impl RandPulse {
    pub fn new() -> Self {
        Self {
            ticker: Ticker::every(Duration::from_millis(10)),
            counter: 0,
            buffer_matrix: [[0; Framebuffer::WIDTH]; Framebuffer::HEIGHT],
            noisegen: rand::WhiteNoiseGenerator::new(),
        }
    }
}

impl MatrixMode for RandPulse {
    async fn animate(&mut self) {
        if self.counter >= 30 {
            let row = self.noisegen.rand8() as usize % Framebuffer::HEIGHT;
            let col = self.noisegen.rand8() as usize % Framebuffer::WIDTH;
            self.buffer_matrix[row][col] = 150;
            self.counter = 0;
        }
        self.counter += 1;

        for row in 0..Framebuffer::HEIGHT {
            for col in 0..Framebuffer::WIDTH {
                if self.buffer_matrix[row][col] < Matrix::fb().load(col, row) {
                    Matrix::fb().store(col, row, Matrix::fb().load(col, row) - 1);
                } else if self.buffer_matrix[row][col] > Matrix::fb().load(col, row) {
                    Matrix::fb().store(col, row, Matrix::fb().load(col, row) + 1);
                }
                if self.buffer_matrix[row][col] == Matrix::fb().load(col, row) {
                    self.buffer_matrix[row][col] = 0;
                }
            }
        }
        self.ticker.next().await;
    }
}
