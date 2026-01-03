use super::HUETABLE;
use crate::drivers::ws2812::Color;
use crate::util::rand;
use crate::util::ws2812::FilteredWs2812;
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Ticker};

pub async fn run(filt_ws2812: &mut FilteredWs2812<'_>) -> ! {
    let mut progress_clock = Ticker::every(Duration::from_millis(1500));
    let mut update_clock = Ticker::every(Duration::from_millis(5));
    let mut noisegen = rand::WhiteNoiseGenerator::new();

    loop {
        match select(progress_clock.next(), update_clock.next()).await {
            Either::First(()) => {
                let snowballs = filt_ws2812.target_mut();

                let num_snowballs = snowballs
                    .iter()
                    .filter(|f| **f != Color::new(0, 0, 0))
                    .count() as u8;
                let probability = (256 / 3) as u8 - 8 * num_snowballs;

                snowballs[0] = snowballs[1];
                snowballs[1] = snowballs[2];
                snowballs[2] = if noisegen.rand8() < probability {
                    let hue: u8 = noisegen.rand8();

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
                snowballs[3] = if noisegen.rand8() < probability {
                    let hue = noisegen.rand8();

                    Color::new(
                        HUETABLE[hue.wrapping_add(85) as usize],
                        HUETABLE[hue as usize],
                        HUETABLE[hue.wrapping_add(170) as usize],
                    )
                } else {
                    Color::new(0, 0, 0)
                };
            }
            Either::Second(()) => {
                filt_ws2812.update().await;
            }
        }
    }
}
