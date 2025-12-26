use super::{HUETABLE, SINTABLE, Ws2812Mode};
use crate::drivers::ws2812::Color;
use crate::util::rand;
use embassy_time::{Duration, Ticker};

// Original "Fire" mode
pub struct Fire {
    ticker: Ticker,
    phases: [u16; 6],
    noisegen: rand::WhiteNoiseGenerator,
}

impl Fire {
    pub fn new() -> Self {
        let ticker = Ticker::every(Duration::from_millis(30));
        let mut phases: [u16; 6] = [0; 6];
        let mut noisegen = rand::WhiteNoiseGenerator::new();
        for i in 0..6 {
            phases[i] = (noisegen.rand8() as u16) << 7;
        }
        Self { ticker, phases, noisegen }
    }
}

impl Ws2812Mode for Fire {
    async fn animate(&mut self) -> [Color; 6] {
        self.ticker.next().await;

        let mut desired_output: [Color; 6] = [Color::new(0, 0, 0); 6];

        for phase in &mut self.phases {
            *phase += (((self.noisegen.rand8() as u16 + 0xf) << 2)
                + ((self.noisegen.rand8() as u16 + 0xf) << 1))
                >> 1;
        }

        for (ledno, led) in desired_output.iter_mut().enumerate() {
            let index: usize = (self.phases[ledno] >> 8) as usize;
            let rs: u8 = SINTABLE[index] >> 3;

            led.r = (HUETABLE[rs.wrapping_add(30) as usize] as u32 >> 2) as u8;
            led.g = (HUETABLE[rs as usize] as u32 >> 3) as u8;
            led.b = (HUETABLE[rs.wrapping_add(190) as usize] as u32 >> 3) as u8;
        }
        desired_output
    }
}
