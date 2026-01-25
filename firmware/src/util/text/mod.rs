mod font;

use core::sync::atomic::Ordering;

use embassy_time::Duration;

use crate::{
    drivers::matrix::{Framebuffer, Matrix},
    util::text::font::Letter,
};

pub const TEXT_DURATION: Duration = Duration::from_millis(75);
pub const TEXT_BRIGHTNESS: u8 = 64;

const _: () = {
    assert!(Framebuffer::HEIGHT == 9);
    assert!(Letter::MAX_WIDTH <= u8::MAX as usize);
};

fn move_left() {
    for row in Matrix::fb().0.iter() {
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

pub struct TextScroller {
    pos: usize,
    col: u8,
}

impl TextScroller {
    pub const fn new() -> Self {
        Self { pos: 0, col: 0 }
    }

    pub fn advance_clear(&mut self) -> bool {
        self.advance_brightness(b"EE", 0)
    }

    pub fn advance(&mut self, text: &[u8]) -> bool {
        self.advance_brightness(text, TEXT_BRIGHTNESS)
    }

    pub fn advance_brightness(&mut self, text: &[u8], brightness: u8) -> bool {
        move_left();

        if let Some(&c) = text.get(self.pos) {
            if let Some(&(mut col)) = Letter::get(c).0.get(self.col as usize)
                && let Some(ds) = Letter::downshift(col)
            {
                for r in ds..Matrix::fb().0.len() {
                    let val = if col & 1 != 0 { brightness } else { 0 };
                    Matrix::fb().0[r].last().unwrap().store(val, Ordering::Relaxed);
                    col >>= 1;
                }
                self.col += 1;
            } else {
                self.pos += 1;
                self.col = 0;
            }
        }

        self.pos >= text.len()
    }
}
