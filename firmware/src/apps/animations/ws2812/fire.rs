use crate::drivers::ws2812::{self};
use crate::util::rand;
use crate::util::ws2812::{FilteredWs2812, HUETABLE, SINTABLE};
use embassy_time::{Duration, Ticker};

pub struct Task {
    progress_ticker: Ticker,
    update_ticker: Ticker,
    rng: rand::Rng,
    phases: [u16; ws2812::LEDS],
}

impl Task {
    #[must_use]
    pub fn new() -> Self {
        let mut task = Self {
            progress_ticker: Ticker::every(Duration::from_millis(100)),
            update_ticker: Ticker::every(Duration::from_millis(10)),
            rng: rand::Rng::new(),
            phases: Default::default(),
        };

        for phase in &mut task.phases {
            *phase = (task.rng.rand8() as u16) << 7;
        }

        task
    }

    pub fn poll(&mut self, ws2812: &mut FilteredWs2812) {
        if self.progress_ticker.consume_expired() {
            for phase in &mut self.phases {
                *phase = phase.wrapping_add(self.rng.rand8() as u16);
            }

            for (led, phase) in ws2812.target_mut().iter_mut().zip(self.phases) {
                let index: usize = (phase >> 8) as usize;
                let rs: u8 = if index >= 128 {
                    255 - SINTABLE[index - 128]
                } else {
                    SINTABLE[index]
                } >> 3;

                led.set_r((HUETABLE[rs.wrapping_add(30) as usize] as u32 >> 2) as u8);
                led.set_g((HUETABLE[rs as usize] as u32 >> 3) as u8);
                led.set_b((HUETABLE[rs.wrapping_add(190) as usize] as u32 >> 3) as u8);
            }
        }

        if self.update_ticker.expired() && ws2812.try_update() {
            self.update_ticker.consume_next();
        }
    }
}
