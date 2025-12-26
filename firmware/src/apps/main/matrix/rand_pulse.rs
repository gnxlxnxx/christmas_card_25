use crate::drivers::matrix::{Framebuffer, Matrix};
use super::MatrixMode;
use crate::util::rand;
use core::sync::atomic::Ordering;
use embassy_time::{Duration, Ticker, Timer};

pub async fn run() -> ! {
    let mut ticker = Ticker::every(Duration::from_millis(10));
    let mut counter = 0;
    let mut buffer_matrix = [[0; Framebuffer::WIDTH]; Framebuffer::HEIGHT];
    let mut noisegen = rand::WhiteNoiseGenerator::new();

    loop {
        if counter >= 30 {
            let row = noisegen.rand8() as usize % Framebuffer::HEIGHT;
            let col = noisegen.rand8() as usize % Framebuffer::WIDTH;
            buffer_matrix[row][col] = 150;
            counter = 0;
        }
        counter += 1;

        for row in 0..Framebuffer::HEIGHT {
            for col in 0..Framebuffer::WIDTH {
                if buffer_matrix[row][col] < Matrix::fb().load(col, row) {
                    Matrix::fb().store(col, row, Matrix::fb().load(col, row) - 1);
                } else if buffer_matrix[row][col] > Matrix::fb().load(col, row) {
                    Matrix::fb().store(col, row, Matrix::fb().load(col, row) + 1);
                }
                if buffer_matrix[row][col] == Matrix::fb().load(col, row) {
                    buffer_matrix[row][col] = 0;
                }
            }
        }
        ticker.next().await;
    }
}
