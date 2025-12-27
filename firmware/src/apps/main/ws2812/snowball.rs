use super::{HUETABLE};
use crate::drivers::ws2812::{self, Color};
use crate::util::rand;
use crate::util::ws2812::FilteredWs2812;
use embassy_time::{Duration, Ticker, Timer};

pub async fn run(filt_ws2812: &mut FilteredWs2812<'_, '_>) -> ! {
    let mut counter = 0;
    let mut noisegen = rand::WhiteNoiseGenerator::new();

    loop {
        if counter == 300 {
            let snowballs = filt_ws2812.target_mut();

            let num_snowballs_l: u8 = snowballs[3..ws2812::LEDS]
                .iter()
                .filter(|snowball| **snowball != Color::new(0, 0, 0))
                .count() as u8;
            let num_snowballs_r: u8 = snowballs[0..3]
                .iter()
                .filter(|snowball| **snowball != Color::new(0, 0, 0))
                .count() as u8;
            let mut hue: u8 = noisegen.rand8();
            snowballs[0] = snowballs[1];
            snowballs[1] = snowballs[2];
            snowballs[2] = if !noisegen.rand8().is_multiple_of(3 + num_snowballs_r) {
                Color::new(0, 0, 0)
            } else {
                Color::new(
                    HUETABLE[hue.wrapping_add(85) as usize],
                    HUETABLE[hue as usize],
                    HUETABLE[hue.wrapping_add(170) as usize],
                )
            };

            hue = noisegen.rand8();
            snowballs[5] = snowballs[4];
            snowballs[4] = snowballs[3];
            snowballs[3] = if !noisegen.rand8().is_multiple_of(3 + num_snowballs_l) {
                Color::new(0, 0, 0)
            } else {
                Color::new(
                    HUETABLE[hue.wrapping_add(85) as usize],
                    HUETABLE[hue as usize],
                    HUETABLE[hue.wrapping_add(170) as usize],
                )
            };
            counter = 0;
        }
        counter += 1;
        filt_ws2812.update().await;

        Timer::after_millis(5).await;
    }
}
