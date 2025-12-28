use super::HUETABLE;
use crate::{
    drivers::ws2812::{self},
    util::ws2812::FilteredWs2812,
};
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Ticker};

pub async fn run(filt_ws2812: &mut FilteredWs2812<'_, '_>) -> ! {
    let mut progress_clock = Ticker::every(Duration::from_millis(32));
    let mut update_clock = Ticker::every(Duration::from_millis(1));
    let mut counter = 0u8;

    loop {
        match select(progress_clock.next(), update_clock.next()).await {
            Either::First(()) => {
                for (ledno, led) in filt_ws2812.target_mut().iter_mut().enumerate() {
                    let ang = counter as usize + ledno * (256 / ws2812::LEDS);
                    led.set_r(HUETABLE[(ang + 85) % HUETABLE.len()]);
                    led.set_g(HUETABLE[ang % HUETABLE.len()]);
                    led.set_b(HUETABLE[(ang + 170) % HUETABLE.len()]);
                }
                counter = counter.wrapping_add(1);
            }
            Either::Second(()) => {
                filt_ws2812.update().await;
            }
        }
    }
}
