use ch32_hal as hal;
use hal::spi::{Config, Spi};
use hal::{Peri, peripherals};

use crate::util::rand;

use embassy_time::Timer;

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

    fn to_slices_grb(&self, result: &mut [u16; 6]) {
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
    output: [Color; 6],
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
        let output = [Color::new(0, 0, 0); 6];

        Self { spi, output }
    }

    pub async fn set_colors(&mut self, colors: [Color; 6]) {
        self.output = colors;
        self.start().await;
    }

    async fn start(&mut self) {
        // I have 2 leading and one trailing led full of '0's
        let mut buf = [[0u16; 6]; 6 + 3];

        for (i, led) in (&mut buf[2..8]).into_iter().enumerate() {
            self.output[i].to_slices_grb(led);
        }

        let _ = self.spi.write::<u16>(&buf.as_flattened()).await;
    }

    pub async fn run_mode(&mut self, mode: &mut Ws2812Mode) {
        let mut desired_output: [Color; 6] = [Color::new(0, 0, 0); 6];

        match mode {
            Ws2812Mode::Fire { phases, noisegen } => {
                // Original "Fire" mode

                Timer::after_millis(30).await;

                for k in 0..6 {
                    phases[k] += (((noisegen.rand8() as u16 + 0xf) << 2)
                        + ((noisegen.rand8() as u16 + 0xf) << 1))
                        >> 1;
                }

                for ledno in 0..6 {
                    let index: usize = (phases[ledno] >> 8) as usize;
                    let rs: u8 = SINTABLE[index] >> 3;

                    desired_output[ledno].r =
                        (HUETABLE[((rs + 30) & 0xff) as usize] as u32 >> 2) as u8;
                    desired_output[ledno].g = (HUETABLE[(rs + 0) as usize] as u32 >> 3) as u8;
                    desired_output[ledno].b =
                        (HUETABLE[((rs + 190) & 0xff) as usize] as u32 >> 3) as u8;
                }
            }
            Ws2812Mode::Snowball {
                counter,
                snowballs,
                noisegen,
            } => {
                // "Snowball" mode

                Timer::after_millis(5).await;

                if *counter == 300 {
                    let num_snowballs_l: u8 = snowballs[3..6]
                        .iter()
                        .filter(|snowball| **snowball != Color::new(0, 0, 0))
                        .count() as u8;
                    let num_snowballs_r: u8 = snowballs[0..3]
                        .iter()
                        .filter(|snowball| **snowball != Color::new(0, 0, 0))
                        .count() as u8;
                    let mut hue: u8 = noisegen.rand8();
                    snowballs[0] = snowballs[1];
                    snowballs[1] = snowballs[2];
                    snowballs[2] = if noisegen.rand8() % (3 + num_snowballs_r) != 0 {
                        Color::new(0, 0, 0)
                    } else {
                        Color::new(
                            HUETABLE[((hue + 85) & 0xff) as usize],
                            HUETABLE[(hue + 0) as usize],
                            HUETABLE[((hue + 170) & 0xff) as usize],
                        )
                    };

                    hue = noisegen.rand8();
                    snowballs[5] = snowballs[4];
                    snowballs[4] = snowballs[3];
                    snowballs[3] = if noisegen.rand8() % (3 + num_snowballs_l) != 0 {
                        Color::new(0, 0, 0)
                    } else {
                        Color::new(
                            HUETABLE[((hue + 85) & 0xff) as usize],
                            HUETABLE[(hue + 0) as usize],
                            HUETABLE[((hue + 170) & 0xff) as usize],
                        )
                    };
                    *counter = 0;
                }
                for ledno in 0..6 {
                    desired_output[ledno] = snowballs[ledno];
                }
                *counter += 1;
            }

            Ws2812Mode::Huewheel { ws2812_counter } => {
                // "Huewheel" mode

                Timer::after_millis(2).await;

                for ledno in 0..6 {
                    let ang: usize = (*ws2812_counter as usize >> 3) + ledno * 60;
                    desired_output[ledno].r = HUETABLE[(ang + 85) & 0xff];
                    desired_output[ledno].g = HUETABLE[ang & 0xff];
                    desired_output[ledno].b = HUETABLE[(ang + 170) & 0xff];
                }
                *ws2812_counter = (*ws2812_counter + 1) & 0x7ff;
            }
        }

        for ledno in 0..6 {
            if self.output[ledno].r > desired_output[ledno].r {
                self.output[ledno].r -= 1;
            } else if self.output[ledno].r < desired_output[ledno].r {
                self.output[ledno].r += 1;
            }

            if self.output[ledno].g > desired_output[ledno].g {
                self.output[ledno].g -= 1;
            } else if self.output[ledno].g < desired_output[ledno].g {
                self.output[ledno].g += 1;
            }

            if self.output[ledno].b > desired_output[ledno].b {
                self.output[ledno].b -= 1;
            } else if self.output[ledno].b < desired_output[ledno].b {
                self.output[ledno].b += 1;
            }
        }

        self.start().await;
    }
}

pub enum Ws2812Mode {
    Fire {
        phases: [u16; 6],
        noisegen: rand::WhiteNoiseGenerator,
    },
    Snowball {
        counter: u32,
        snowballs: [Color; 6],
        noisegen: rand::WhiteNoiseGenerator,
    },
    Huewheel {
        ws2812_counter: u32,
    },
}

impl Ws2812Mode {
    pub fn new() -> Self {
        Self::new_huewheel()
    }
    pub fn new_fire() -> Self {
        let mut phases: [u16; 6] = [0; 6];
        let mut noisegen = rand::WhiteNoiseGenerator::new();
        for i in 0..6 {
            phases[i] = (noisegen.rand8() as u16) << 7;
        }
        Self::Fire { phases, noisegen }
    }
    pub fn new_huewheel() -> Self {
        let ws2812_counter = 0;
        Self::Huewheel { ws2812_counter }
    }
    pub fn new_snowball() -> Self {
        let counter = 0;
        let snowballs: [Color; 6] = [Color::new(0, 0, 0); 6];
        let noisegen = rand::WhiteNoiseGenerator::new();
        Self::Snowball {
            counter,
            snowballs,
            noisegen,
        }
    }
}

pub const HUETABLE: [u8; 256] = [
    0x00, 0x06, 0x0c, 0x12, 0x18, 0x1e, 0x24, 0x2a, 0x30, 0x36, 0x3c, 0x42, 0x48, 0x4e, 0x54, 0x5a,
    0x60, 0x66, 0x6c, 0x72, 0x78, 0x7e, 0x84, 0x8a, 0x90, 0x96, 0x9c, 0xa2, 0xa8, 0xae, 0xb4, 0xba,
    0xc0, 0xc6, 0xcc, 0xd2, 0xd8, 0xde, 0xe4, 0xea, 0xf0, 0xf6, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xf9, 0xf3, 0xed, 0xe7, 0xe1, 0xdb, 0xd5, 0xcf, 0xc9, 0xc3, 0xbd, 0xb7, 0xb1, 0xab, 0xa5,
    0x9f, 0x99, 0x93, 0x8d, 0x87, 0x81, 0x7b, 0x75, 0x6f, 0x69, 0x63, 0x5d, 0x57, 0x51, 0x4b, 0x45,
    0x3f, 0x39, 0x33, 0x2d, 0x27, 0x21, 0x1b, 0x15, 0x0f, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

pub const SINTABLE: [u8; 256] = [
    0x80, 0x83, 0x86, 0x89, 0x8c, 0x8f, 0x92, 0x95, 0x99, 0x9c, 0x9f, 0xa2, 0xa5, 0xa8, 0xab, 0xad,
    0xb0, 0xb3, 0xb6, 0xb9, 0xbc, 0xbe, 0xc1, 0xc4, 0xc6, 0xc9, 0xcb, 0xce, 0xd0, 0xd3, 0xd5, 0xd7,
    0xda, 0xdc, 0xde, 0xe0, 0xe2, 0xe4, 0xe6, 0xe8, 0xe9, 0xeb, 0xed, 0xee, 0xf0, 0xf1, 0xf3, 0xf4,
    0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfc, 0xfd, 0xfe, 0xfe, 0xfe, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xfe, 0xfe, 0xfe, 0xfd, 0xfc, 0xfc, 0xfb, 0xfa, 0xf9, 0xf8, 0xf7, 0xf6,
    0xf5, 0xf4, 0xf3, 0xf1, 0xf0, 0xee, 0xed, 0xeb, 0xe9, 0xe8, 0xe6, 0xe4, 0xe2, 0xe0, 0xde, 0xdc,
    0xda, 0xd7, 0xd5, 0xd3, 0xd0, 0xce, 0xcb, 0xc9, 0xc6, 0xc4, 0xc1, 0xbe, 0xbc, 0xb9, 0xb6, 0xb3,
    0xb0, 0xad, 0xab, 0xa8, 0xa5, 0xa2, 0x9f, 0x9c, 0x99, 0x95, 0x92, 0x8f, 0x8c, 0x89, 0x86, 0x83,
    0x80, 0x7d, 0x79, 0x76, 0x73, 0x70, 0x6d, 0x6a, 0x67, 0x64, 0x61, 0x5e, 0x5b, 0x58, 0x55, 0x52,
    0x4f, 0x4c, 0x49, 0x47, 0x44, 0x41, 0x3e, 0x3c, 0x39, 0x36, 0x34, 0x31, 0x2f, 0x2d, 0x2a, 0x28,
    0x26, 0x24, 0x21, 0x1f, 0x1d, 0x1b, 0x1a, 0x18, 0x16, 0x14, 0x13, 0x11, 0x10, 0x0e, 0x0d, 0x0b,
    0x0a, 0x09, 0x08, 0x07, 0x06, 0x05, 0x04, 0x04, 0x03, 0x02, 0x02, 0x01, 0x01, 0x01, 0x01, 0x01,
    0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02, 0x02, 0x03, 0x04, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09,
    0x0a, 0x0b, 0x0d, 0x0e, 0x10, 0x11, 0x13, 0x14, 0x16, 0x18, 0x1a, 0x1b, 0x1d, 0x1f, 0x21, 0x24,
    0x26, 0x28, 0x2a, 0x2d, 0x2f, 0x31, 0x34, 0x36, 0x39, 0x3c, 0x3e, 0x41, 0x44, 0x47, 0x49, 0x4c,
    0x4f, 0x52, 0x55, 0x58, 0x5b, 0x5e, 0x61, 0x64, 0x67, 0x6a, 0x6d, 0x70, 0x73, 0x76, 0x79, 0x7d,
];
