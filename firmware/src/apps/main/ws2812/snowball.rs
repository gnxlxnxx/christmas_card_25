use crate::drivers::ws2812::Color;
use crate::util::rand;
use super::{HUETABLE, Ws2812Mode};
use embassy_time::Timer;

// "Snowball" mode
pub struct Snowball {
    counter: u32,
    snowballs: [Color; 6],
    noisegen: rand::WhiteNoiseGenerator,
}

impl Snowball {
    pub fn new() -> Self {
        let counter = 0;
        let snowballs: [Color; 6] = [Color::new(0, 0, 0); 6];
        let noisegen = rand::WhiteNoiseGenerator::new();
        Self {
            counter,
            snowballs,
            noisegen,
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
            self.snowballs[2] = if self.noisegen.rand8() % (3 + num_snowballs_r) != 0 {
                Color::new(0, 0, 0)
            } else {
                Color::new(
                    HUETABLE[((hue + 85) & 0xff) as usize],
                    HUETABLE[(hue + 0) as usize],
                    HUETABLE[((hue + 170) & 0xff) as usize],
                )
            };

            hue = self.noisegen.rand8();
            self.snowballs[5] = self.snowballs[4];
            self.snowballs[4] = self.snowballs[3];
            self.snowballs[3] = if self.noisegen.rand8() % (3 + num_snowballs_l) != 0 {
                Color::new(0, 0, 0)
            } else {
                Color::new(
                    HUETABLE[((hue + 85) & 0xff) as usize],
                    HUETABLE[(hue + 0) as usize],
                    HUETABLE[((hue + 170) & 0xff) as usize],
                )
            };
            self.counter = 0;
        }
        self.counter += 1;
        self.snowballs
    }
}
