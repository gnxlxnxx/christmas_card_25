const NOISE_BITS: u32 = 8;
const NOISE_MASK: u32 = ((1 << NOISE_BITS) - 1);
const NOISE_POLY_TAP0: u32 = 31;
const NOISE_POLY_TAP1: u32 = 21;
const NOISE_POLY_TAP2: u32 = 1;
const NOISE_POLY_TAP3: u32 = 0;

pub struct WhiteNoiseGenerator {
    lfsr: u32,
}

impl WhiteNoiseGenerator {
    pub fn new() -> Self {
        Self { lfsr: 1 }
    }

    pub fn rand8(&mut self) -> u8 {
        for bit in 0..NOISE_BITS {
            let new_data: u32 = (self.lfsr >> NOISE_POLY_TAP0)
                ^ (self.lfsr >> NOISE_POLY_TAP1)
                ^ (self.lfsr >> NOISE_POLY_TAP2)
                ^ (self.lfsr >> NOISE_POLY_TAP3);
            self.lfsr = (self.lfsr << 1) | (new_data & 1);
        }

        (self.lfsr & NOISE_MASK) as u8
    }
}
