mod font;

use embassy_time::Ticker;

use crate::{
    drivers::matrix::{Framebuffer, Matrix},
    util::text::font::Letter,
};

fn move_left(fb: &Framebuffer) {
    for y in 0..Framebuffer::HEIGHT {
        for x in 0..Framebuffer::WIDTH {
            fb.store(x, y, fb.try_load(x + 1, y).unwrap_or(0));
        }
    }
}

pub async fn clear_scroll(clock: &mut Ticker) {
    for _ in 0..Framebuffer::WIDTH - 1 {
        move_left(Matrix::fb());
        clock.next().await;
    }
}

pub async fn scroll(text: &[u8], brightness: u8, clock: &mut Ticker) {
    let fb = Matrix::fb();

    move_left(fb);
    clock.next().await;

    for &c in text {
        for mut col in Letter::get(c).0 {
            let downshift = match Letter::downshift(col) {
                Some(n) => n,
                None => break,
            };

            move_left(fb);

            for base_y in 0..Framebuffer::HEIGHT - 2 {
                fb.store(
                    Framebuffer::WIDTH - 1,
                    base_y + downshift,
                    if col & 1 != 0 { brightness } else { 0 },
                );
                col >>= 1;
            }

            clock.next().await;
        }

        move_left(fb);
        clock.next().await;
    }
}
