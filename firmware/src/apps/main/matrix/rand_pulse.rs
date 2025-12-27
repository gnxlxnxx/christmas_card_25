use crate::drivers::matrix::{Framebuffer, Matrix};
use crate::util::rand;
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Ticker};

pub async fn run() -> ! {
    let mut update_clock = Ticker::every(Duration::from_millis(10));
    let mut gen_clock = Ticker::every(Duration::from_millis(300));
    let mut buffer_matrix = [[0; Framebuffer::WIDTH]; Framebuffer::HEIGHT];
    let mut noisegen = rand::WhiteNoiseGenerator::new();

    loop {
        match select(gen_clock.next(), update_clock.next()).await {
            Either::First(()) => {
                let row = noisegen.rand8() as usize % Framebuffer::HEIGHT;
                let col = noisegen.rand8() as usize % Framebuffer::WIDTH;
                buffer_matrix[row][col] = 150;
            }
            Either::Second(()) => {
                for (row, buffer_row) in buffer_matrix.iter_mut().enumerate().take(Framebuffer::HEIGHT) {
                    for (col, buffer_field) in buffer_row.iter_mut().enumerate().take(Framebuffer::WIDTH) {
                        if *buffer_field < Matrix::fb().load(col, row) {
                            Matrix::fb().store(col, row, Matrix::fb().load(col, row) - 1);
                        } else if *buffer_field > Matrix::fb().load(col, row) {
                            Matrix::fb().store(col, row, Matrix::fb().load(col, row) + 1);
                        }
                        if *buffer_field == Matrix::fb().load(col, row) {
                            *buffer_field = 0;
                        }
                    }
                }
            }
        }
    }
}
