use crate::drivers::matrix::{Framebuffer, Matrix};
use crate::util::rand;
use embassy_time::{Duration, Ticker};

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

        for (row, matrix_row) in buffer_matrix.iter_mut().enumerate() {
            for (col, matrix_field) in matrix_row.iter_mut().enumerate() {
                if *matrix_field < Matrix::fb().load(col, row) {
                    Matrix::fb().store(col, row, Matrix::fb().load(col, row) - 1);
                } else if *matrix_field > Matrix::fb().load(col, row) {
                    Matrix::fb().store(col, row, Matrix::fb().load(col, row) + 1);
                }
                if *matrix_field == Matrix::fb().load(col, row) {
                    *matrix_field = 0;
                }
            }
        }
        ticker.next().await;
    }
}
