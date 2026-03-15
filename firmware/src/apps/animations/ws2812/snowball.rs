use crate::drivers::ws2812::{Color, LEDS};
use crate::util::rand;
use crate::util::ws2812::{FilteredWs2812, HUETABLE};
use embassy_time::{Duration, Ticker};

const _: () = assert!(LEDS == 6);

pub struct Task {
    progress_ticker: Ticker,
    update_ticker: Ticker,
    rng: rand::Rng,
}

impl Task {
    #[must_use]
    pub fn new() -> Self {
        Self {
            progress_ticker: Ticker::every(Duration::from_millis(1500)),
            update_ticker: Ticker::every(Duration::from_millis(50)),
            rng: rand::Rng::new(),
        }
    }

    pub fn poll(&mut self, ws2812: &mut FilteredWs2812) {
        if self.progress_ticker.consume_expired() {
            let snowballs = ws2812.target_mut();

            let num_snowballs = snowballs
                .iter()
                .filter(|f| **f != Color::new(0, 0, 0))
                .count() as u8;
            let probability = (256 / 3) as u8 - 8 * num_snowballs;

            snowballs[0] = snowballs[1];
            snowballs[1] = snowballs[2];
            snowballs[2] = if self.rng.rand8() < probability {
                let hue: u8 = self.rng.rand8();

                Color::new(
                    HUETABLE[hue.wrapping_add(85) as usize],
                    HUETABLE[hue as usize],
                    HUETABLE[hue.wrapping_add(170) as usize],
                )
            } else {
                Color::new(0, 0, 0)
            };

            snowballs[5] = snowballs[4];
            snowballs[4] = snowballs[3];
            snowballs[3] = if self.rng.rand8() < probability {
                let hue = self.rng.rand8();

                Color::new(
                    HUETABLE[hue.wrapping_add(85) as usize],
                    HUETABLE[hue as usize],
                    HUETABLE[hue.wrapping_add(170) as usize],
                )
            } else {
                Color::new(0, 0, 0)
            };
        }

        if self.update_ticker.expired() && ws2812.try_update() {
            self.update_ticker.consume_next();
        }
    }
}
