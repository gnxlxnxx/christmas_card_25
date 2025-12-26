use super::{HUETABLE, Ws2812Mode};
use crate::drivers::ws2812::Color;
use crate::util::rand;
use embassy_time::{Duration, Ticker, Timer};

// "Snowball" mode
pub struct Snowball {
    counter: u32,
    snowballs: [Color; 6],
    noisegen: rand::WhiteNoiseGenerator,
}

impl Snowball {
    pub fn new() -> Self {
        Self {
            counter: 0,
            snowballs: [Color::new(0, 0, 0); 6],
            noisegen: rand::WhiteNoiseGenerator::new(),
        }
    }
}

impl Ws2812Mode for Snowball {
    async fn animate(&mut self) -> [Color; 6] {
        Timer::after_millis(5).await;

        if self.counter == 300 {
            let num_snowballs_l: u8 = self.snowballs[3..6]
                .iter()
                .filter(|snowball| **snowball != Color::new(0, 0, 0))
                .count() as u8;
            let num_snowballs_r: u8 = self.snowballs[0..3]
                .iter()
                .filter(|snowball| **snowball != Color::new(0, 0, 0))
                .count() as u8;
            let mut hue: u8 = self.noisegen.rand8();
            self.snowballs[0] = self.snowballs[1];
            self.snowballs[1] = self.snowballs[2];
            self.snowballs[2] = if !self.noisegen.rand8().is_multiple_of(3 + num_snowballs_r) {
                Color::new(0, 0, 0)
            } else {
                Color::new(
                    HUETABLE[hue.wrapping_add(85) as usize],
                    HUETABLE[hue as usize],
                    HUETABLE[hue.wrapping_add(170) as usize],
                )
            };

            hue = self.noisegen.rand8();
            self.snowballs[5] = self.snowballs[4];
            self.snowballs[4] = self.snowballs[3];
            self.snowballs[3] = if !self.noisegen.rand8().is_multiple_of(3 + num_snowballs_l) {
                Color::new(0, 0, 0)
            } else {
                Color::new(
                    HUETABLE[hue.wrapping_add(85) as usize],
                    HUETABLE[hue as usize],
                    HUETABLE[hue.wrapping_add(170) as usize],
                )
            };
            self.counter = 0;
        }
        self.counter += 1;
        self.snowballs
    }
}
