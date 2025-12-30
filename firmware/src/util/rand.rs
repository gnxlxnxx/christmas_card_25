use core::sync::atomic::{AtomicU32, Ordering};
const NOISE_BITS: u32 = 8;
const NOISE_MASK: u32 = (1 << NOISE_BITS) - 1;
const NOISE_POLY_TAP0: u32 = 31;
const NOISE_POLY_TAP1: u32 = 21;
const NOISE_POLY_TAP2: u32 = 1;
const NOISE_POLY_TAP3: u32 = 0;

static LFSR: AtomicU32 = AtomicU32::new(1);

#[derive(Debug)]
pub struct WhiteNoiseGenerator;

impl WhiteNoiseGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn rand8(&mut self) -> u8 {
        let mut lfsr = LFSR.load(Ordering::Relaxed);
        for _bit in 0..NOISE_BITS {
            let new_data: u32 = (lfsr >> NOISE_POLY_TAP0)
                ^ (lfsr >> NOISE_POLY_TAP1)
                ^ (lfsr >> NOISE_POLY_TAP2)
                ^ (lfsr >> NOISE_POLY_TAP3);
            lfsr = (lfsr << 1) | (new_data & 1);
        }

        LFSR.store(lfsr, Ordering::Relaxed);
        (lfsr & NOISE_MASK) as u8
    }
}
