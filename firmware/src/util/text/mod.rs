mod font;

use core::{iter, sync::atomic::Ordering};

use embassy_time::{Duration, Ticker};

use crate::{
    drivers::matrix::{Framebuffer, Matrix},
    util::text::font::Letter,
};

pub const TEXT_DURATION: Duration = Duration::from_millis(75);
pub const TEXT_BRIGHTNESS: u8 = 64;

fn move_left(fb: &Framebuffer) {
    for row in fb.0.iter() {
        let len = row.len();
        if len == 0 {
            continue;
        }

        for i in 0..len - 1 {
            let next_val = row[i + 1].load(Ordering::Relaxed);
            row[i].store(next_val, Ordering::Relaxed);
        }

        row[len - 1].store(0, Ordering::Relaxed);
    }
}

pub fn clear_scroll(clock: &mut Ticker) -> impl Future<Output = ()> {
    scroll(b"EE", 0, clock)
}

pub async fn scroll(text: &[u8], brightness: u8, clock: &mut Ticker) {
    let fb = Matrix::fb();

    for &c in text {
        let glyph = Letter::get(c).0;

        let mut i = 0;
        while i < 5 {
            let mut col = glyph[i];

            let downshift = match Letter::downshift(col) {
                Some(n) => n,
                None => break,
            };

            move_left(fb);

            for r in downshift..fb.0.len() {
                let val = if col & 1 != 0 { brightness } else { 0 };
                fb.0[r].last().unwrap().store(val, Ordering::Relaxed);
                col >>= 1;
            }


            clock.next().await;
            i += 1;
        }

        move_left(fb);
        clock.next().await;
    }
}
