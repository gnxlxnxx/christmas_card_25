use crate::drivers::matrix::Matrix;
use crate::util::rand;
use core::sync::atomic::Ordering;
use embassy_time::{Duration, Ticker};

pub struct Task {
    advance_ticker: Ticker,
    clear_ticker: Option<Ticker>,
    rng: rand::Rng,
}

impl Task {
    pub fn new() -> Self {
        Self {
            advance_ticker: Ticker::every(Duration::from_millis(256)),
            clear_ticker: None,
            rng: rand::Rng::new(),
        }
    }

    pub fn poll(&mut self) {
        let fb = Matrix::fb();

        if self.advance_ticker.consume_expired() {
            if self.clear_ticker.is_none() {
                let mut full = true;

                for (t, b) in fb.0[fb.0.len() - 2].iter().zip(fb.0.last().unwrap()) {
                    let new = t.load(Ordering::Relaxed) | b.load(Ordering::Relaxed);
                    b.store(new, Ordering::Relaxed);

                    if new == 0 {
                        full = false;
                    }
                }

                if full {
                    self.clear_ticker = Some(Ticker::every(Duration::from_millis(4)));
                }
            }

            for i in (0..fb.0.len() - 2).rev() {
                let t_row = &fb.0[i];
                let b_row = &fb.0[i + 1];

                for (t, b) in t_row.iter().zip(b_row) {
                    b.store(t.load(Ordering::Relaxed), Ordering::Relaxed);
                }
            }

            for field in fb.0.first().unwrap() {
                field.store(
                    if self.rng.rand8() < 4 { 127 } else { 0 },
                    Ordering::Relaxed,
                );
            }
        }

        if self.clear_ticker.as_mut().is_some_and(|t| t.consume_expired()) {
            let mut empty = true;

            for field in fb.0.last().unwrap() {
                let cur = field.load(Ordering::Relaxed) as u32;
                let new = (((cur << 7) - cur) >> 7) as u8;
                field.store(new, Ordering::Relaxed);

                if new != 0 {
                    empty = false;
                }
            }

            if empty {
                self.clear_ticker = None;
            }
        }
    }
}
