mod font;

use core::{iter, sync::atomic::Ordering};

use embassy_time::Ticker;

use crate::{
    drivers::matrix::{Framebuffer, Matrix},
    util::text::font::Letter,
};

fn move_left(fb: &Framebuffer) {
    for row in fb.0.iter() {
        row.iter()
            .zip(
                row.iter()
                    .skip(1)
                    .map(|r| r.load(Ordering::Relaxed))
                    .chain(iter::repeat(0)),
            )
            .for_each(|(l, r)| l.store(r, Ordering::Relaxed));
    }
}

pub async fn clear_scroll(clock: &mut Ticker) {
    let fb = Matrix::fb();

    for _ in 0..Framebuffer::WIDTH - 1 {
        move_left(fb);
        clock.next().await;
    }
}

pub async fn scroll(text: &[u8], brightness: u8, clock: &mut Ticker) {
    let fb = Matrix::fb();

    move_left(fb);
    clock.next().await;

    for c in text {
        for mut col in Letter::get(*c).0 {
            let downshift = match Letter::downshift(col) {
                Some(n) => n,
                None => break,
            };

            move_left(fb);

            for row in fb.0.iter().skip(downshift).take(fb.0.len() - 2) {
                row.last().unwrap().store(
                    if col & 1 != 0 { brightness } else { 0 },
                    Ordering::Relaxed
                );
                col >>= 1;
            }

            clock.next().await;
        }

        move_left(fb);
        clock.next().await;
    }
}
