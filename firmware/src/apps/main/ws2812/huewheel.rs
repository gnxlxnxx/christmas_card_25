use super::{HUETABLE};
use crate::{drivers::ws2812::{self, Color}, util::ws2812::FilteredWs2812};
use embassy_time::{Duration, Ticker};

pub async fn run(filt_ws2812: &mut FilteredWs2812<'_, '_>) -> ! {
    let mut ticker = Ticker::every(Duration::from_millis(2));
    let mut counter = 0;

    loop {
        for (ledno, led) in filt_ws2812.target_mut().iter_mut().enumerate() {
            let ang: usize = (counter as usize >> 3) + ledno * (255 / ws2812::LEDS);
            led.set_r(HUETABLE[(ang + 85) % HUETABLE.len()]);
            led.set_g(HUETABLE[ang % HUETABLE.len()]);
            led.set_b(HUETABLE[(ang + 170) % HUETABLE.len()]);
        }
        counter = (counter + 1) & 0x7ff;

        filt_ws2812.update().await;
        ticker.next().await;
    }
}
