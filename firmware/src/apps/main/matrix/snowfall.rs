use crate::drivers::matrix::Matrix;
use crate::util::rand;
use core::sync::atomic::Ordering;
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Ticker};

pub async fn run() -> ! {
    let fb = Matrix::fb();
    let mut update_clock = Ticker::every(Duration::from_millis(256));
    let mut clear_clock = Ticker::every(Duration::from_millis(4));
    let mut clear_bottom = false;
    let mut noisegen = rand::WhiteNoiseGenerator::new();

    loop {
        match select(update_clock.next(), clear_clock.next()).await {
            Either::First(()) => {
                if !clear_bottom {
                    clear_clock.reset();

                    clear_bottom = true;

                    for (t, b) in fb.0[fb.0.len() - 2].iter().zip(fb.0.last().unwrap()) {
                        let new = t.load(Ordering::Relaxed) | b.load(Ordering::Relaxed);
                        b.store(new, Ordering::Relaxed);

                        if new == 0 {
                            clear_bottom = false;
                        }
                    }
                }

                for (t_row, b_row) in fb.0.iter().rev().skip(2).zip(fb.0.iter().rev().skip(1)) {
                    for (t, b) in t_row.iter().zip(b_row).rev() {
                        b.store(t.load(Ordering::Relaxed), Ordering::Relaxed);
                    }
                }

                for field in fb.0.first().unwrap() {
                    field.store(
                        if noisegen.rand8() < 4 { 127 } else { 0 },
                        Ordering::Relaxed,
                    );
                }
            }
            Either::Second(()) => {
                if clear_bottom {
                    clear_bottom = false;

                    for field in fb.0.last().unwrap() {
                        let cur = field.load(Ordering::Relaxed) as u32;
                        let new = (((cur << 7) - cur) >> 7) as u8;
                        field.store(new, Ordering::Relaxed);

                        if new != 0 {
                            clear_bottom = true;
                        }
                    }
                }
            }
        }
    }
}
