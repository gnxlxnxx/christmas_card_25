use ch32_hal as hal;
use hal::pac;
use hal::spi::{Config, Spi};
use hal::{Peri, peripherals};

use crate::util::ws2812::Ws2812Mode;

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

    fn transition(&mut self, desired_value: Self) {
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

        #[cfg(feature = "star")]
        pac::GPIOC.cfglr().modify(|w| {
            w.set_cnf(6, pac::gpio::vals::Cnf::AF_OPEN_DRAIN_OUT);
        });

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

    pub async fn run_mode<T: Ws2812Mode>(&mut self, mode: &mut T) {
        let mut desired_output: [Color; 6] = mode.animate().await;

        for (led, value) in self.output.iter_mut().zip(desired_output) {
            led.transition(value);
        }

        self.start().await;
    }
}
