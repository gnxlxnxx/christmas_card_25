use ch32_hal as hal;
use hal::spi::{Config, Spi};
use hal::{Peri, peripherals};

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
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn transition(&mut self, desired_value: Self) {
        if self.r > desired_value.r {
            self.r -= 1;
        } else if self.r < desired_value.r {
            self.r += 1;
        }

        if self.g > desired_value.g {
            self.g -= 1;
        } else if self.g < desired_value.g {
            self.g += 1;
        }

        if self.b > desired_value.b {
            self.b -= 1;
        } else if self.b < desired_value.b {
            self.b += 1;
        }
    }

    fn gen_grb_data(&self, result: &mut [u16; 6]) {
        result[0] = BITQUARTETS[(self.g >> 4) as usize];
        result[1] = BITQUARTETS[(self.g & 0xF) as usize];
        result[2] = BITQUARTETS[(self.r >> 4) as usize];
        result[3] = BITQUARTETS[(self.r & 0xF) as usize];
        result[4] = BITQUARTETS[(self.b >> 4) as usize];
        result[5] = BITQUARTETS[(self.b & 0xF) as usize];
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

    pub async fn write(&mut self, colors: &[Color; 6]) {
        // I have 2 leading and one trailing led full of '0's
        let mut buf = [[0u16; 6]; 6 + 3];

        for (signal, color) in buf[2..8].iter_mut().zip(colors) {
            color.gen_grb_data(signal);
        }

        let _ = self.spi.write::<u16>(buf.as_flattened()).await;
    }
}
