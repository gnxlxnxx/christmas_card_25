use ch32_hal as hal;
use hal::spi::{Config, Spi};
use hal::{Peri, peripherals};

pub const LEDS: usize = 6;

const BITQUARTETS: [u16; 16] = [
    0b1000100010001000,
    0b1000100010001110,
    0b1000100011101000,
    0b1000100011101110,
    0b1000111010001000,
    0b1000111010001110,
    0b1000111011101000,
    0b1000111011101110,
    0b1110100010001000,
    0b1110100010001110,
    0b1110100011101000,
    0b1110100011101110,
    0b1110111010001000,
    0b1110111010001110,
    0b1110111011101000,
    0b1110111011101110,
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Color([u8; 3]);

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self([g, r, b])
    }

    pub fn r(&self) -> u8 {
        self.0[1]
    }

    pub fn g(&self) -> u8 {
        self.0[0]
    }

    pub fn b(&self) -> u8 {
        self.0[2]
    }

    pub fn set_r(&mut self, val: u8) {
        self.0[1] = val;
    }

    pub fn set_g(&mut self, val: u8) {
        self.0[0] = val;
    }

    pub fn set_b(&mut self, val: u8) {
        self.0[2] = val;
    }

    pub fn transition(&mut self, desired_value: &Self) {
        for (cur, target) in self.0.iter_mut().zip(desired_value.0) {
            if target > *cur {
                *cur += 1;
            } else if target < *cur {
                *cur -= 1;
            }
        }
    }

    fn gen_grb_data(&self, buf: &mut [u16; 6]) {
        for (val, data) in self.0.iter().zip(buf.as_chunks_mut::<2>().0) {
            data[0] = BITQUARTETS[(val >> 4) as usize];
            data[1] = BITQUARTETS[(val & 0xf) as usize];
        }
    }
}

pub struct Ws2812<'a> {
    spi: Spi<'a, peripherals::SPI1, ch32_hal::mode::Async>,
}

impl<'a> Ws2812<'a> {

    pub fn new(
        pin: Peri<'static, peripherals::PC6>,
        spi1: Peri<'static, peripherals::SPI1>,
        dma1_ch3: Peri<'static, peripherals::DMA1_CH3>,
    ) -> Self {
        let mut spi_config = Config::default();
        spi_config.mode = embedded_hal::spi::MODE_0; // CPOL LOW, CPHA

        spi_config.frequency = hal::prelude::Hertz::hz(3_000_000);

        let spi = Spi::new_txonly_nosck::<0>(spi1, pin, dma1_ch3, spi_config);

        Self { spi }
    }

    pub async fn write(&mut self, colors: &[Color; LEDS]) {
        // I have 2 leading and one trailing led full of '0's
        let mut buf = [[0u16; 6]; LEDS + 3];

        for (signal, color) in buf[2..LEDS + 2].iter_mut().zip(colors) {
            color.gen_grb_data(signal);
        }

        let _ = self.spi.write::<u16>(buf.as_flattened()).await;
    }
}
