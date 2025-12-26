use crate::drivers::ws2812::Color;
use super::{HUETABLE, Ws2812Mode};
use embassy_time::Timer;

// "Huewheel" mode
pub struct Huewheel {
    counter: u32,
}

impl Huewheel {
    pub fn new() -> Self {
        let counter = 0;
        Self { counter }
    }
}

impl Ws2812Mode for Huewheel {
    async fn animate(&mut self) -> [Color; 6] {
        let mut desired_output: [Color; 6] = [Color::new(0, 0, 0); 6];
        Timer::after_millis(2).await;

        for ledno in 0..6 {
            let ang: usize = (self.counter as usize >> 3) + ledno * 60;
            desired_output[ledno].r = HUETABLE[(ang + 85) & 0xff];
            desired_output[ledno].g = HUETABLE[ang & 0xff];
            desired_output[ledno].b = HUETABLE[(ang + 170) & 0xff];
        }
        self.counter = (self.counter + 1) & 0x7ff;
        desired_output
    }
}
