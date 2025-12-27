use super::{HUETABLE, Ws2812Mode};
use crate::drivers::ws2812::{self, Color};
use embassy_time::{Duration, Ticker};

// "Huewheel" mode
pub struct Huewheel {
    ticker: Ticker,
    counter: u32,
}

impl Huewheel {
    pub fn new() -> Self {
        Self {
            ticker: Ticker::every(Duration::from_millis(2)),
            counter: 0,
        }
    }
}

impl Ws2812Mode for Huewheel {
    async fn animate(&mut self) -> [Color; ws2812::LEDS] {
        self.ticker.next().await;

        let mut desired_output: [Color; ws2812::LEDS] = [Color::new(0, 0, 0); ws2812::LEDS];

        for (ledno, led) in desired_output.iter_mut().enumerate() {
            let ang: usize = (self.counter as usize >> 3) + ledno * 60;
            led.set_r(HUETABLE[(ang + 85) % HUETABLE.len()]);
            led.set_g(HUETABLE[ang % HUETABLE.len()]);
            led.set_b(HUETABLE[(ang + 170) % HUETABLE.len()]);
        }
        self.counter = (self.counter + 1) & 0x7ff;
        desired_output
    }
}
