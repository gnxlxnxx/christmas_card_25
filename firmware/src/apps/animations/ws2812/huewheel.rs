use crate::{
    drivers::ws2812::{self},
    util::ws2812::{FilteredWs2812, HUETABLE},
};
use embassy_time::{Duration, Ticker};

pub struct Task {
    progress_ticker: Ticker,
    update_ticker: Ticker,
    counter: u8,
}

impl Task {
    pub fn new() -> Self {
        Self {
            progress_ticker: Ticker::every(Duration::from_millis(320)),
            update_ticker: Ticker::every(Duration::from_millis(10)),
            counter: 0,
        }
    }

    pub fn poll(&mut self, ws2812: &mut FilteredWs2812) {
        if self.progress_ticker.consume_expired() {
            for (ledno, led) in ws2812.target_mut().iter_mut().enumerate() {
                let ang = self.counter as usize + ledno * (256 / ws2812::LEDS);
                led.set_r(HUETABLE[(ang + 85) % HUETABLE.len()]);
                led.set_g(HUETABLE[ang % HUETABLE.len()]);
                led.set_b(HUETABLE[(ang + 170) % HUETABLE.len()]);
            }
            self.counter = self.counter.wrapping_add(1);
        }

        if self.update_ticker.expired() && ws2812.try_update() {
            self.update_ticker.consume_next();
        }
    }
}
