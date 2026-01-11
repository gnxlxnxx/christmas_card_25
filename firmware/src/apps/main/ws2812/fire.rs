use crate::drivers::ws2812::{self};
use crate::util::rand;
use crate::util::ws2812::{FilteredWs2812, HUETABLE, SINTABLE};
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Ticker};

pub async fn run(filt_ws2812: &mut FilteredWs2812<'_>) -> ! {
    let mut progress_clock = Ticker::every(Duration::from_millis(100));
    let mut update_clock = Ticker::every(Duration::from_millis(10));

    let mut noisegen = rand::WhiteNoiseGenerator::new();

    let mut phases: [u16; ws2812::LEDS] = [0; ws2812::LEDS];
    for phase in &mut phases {
        *phase = (noisegen.rand8() as u16) << 7;
    }

    loop {
        match select(progress_clock.next(), update_clock.next()).await {
            Either::First(()) => {
                for phase in &mut phases {
                    *phase = phase.wrapping_add(noisegen.rand8() as u16);
                }

                for (led, phase) in filt_ws2812.target_mut().iter_mut().zip(phases) {
                    let index: usize = (phase >> 8) as usize;
                    let rs: u8 = SINTABLE[index] >> 3;

                    led.set_r((HUETABLE[rs.wrapping_add(30) as usize] as u32 >> 2) as u8);
                    led.set_g((HUETABLE[rs as usize] as u32 >> 3) as u8);
                    led.set_b((HUETABLE[rs.wrapping_add(190) as usize] as u32 >> 3) as u8);
                }
            }
            Either::Second(()) => {
                filt_ws2812.update().await;
            }
        }
    }
}
