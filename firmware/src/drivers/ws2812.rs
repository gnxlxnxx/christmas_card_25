use ch32_hal as hal;
use hal::spi::{Config, Spi};
use hal::{Peri, peripherals};
use hal::pac;
use crate::util::sync::Event;
use hal::interrupt;
use hal::interrupt::InterruptExt;

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
        for (val, data) in self.0.iter().zip(buf.chunks_exact_mut(2)) {
            data[0] = BITQUARTETS[(val >> 4) as usize];
            data[1] = BITQUARTETS[(val & 0xf) as usize];
        }
    }
}

pub struct Ws2812 {
    //spi: Spi<'a, peripherals::SPI1, ch32_hal::mode::Async>,
}

static SPI_DMA_EVENT: Event =  Event::new();
// I have 2 leading and one trailing led full of '0's
static mut SPI_DMA_TRANSFER_BUFFER: [[u16; 6]; LEDS + 3] = [[0u16; 6]; LEDS + 3];

impl Ws2812 {
    pub fn new(
        _pin: Peri<'static, peripherals::PC6>,
        _spi1: Peri<'static, peripherals::SPI1>,
        _dma1_ch3: Peri<'static, peripherals::DMA1_CH3>,
    ) -> Self {
        // Remap is implicitly set as 0
        pac::GPIOC.cfglr().modify(|w| {w.set_mode(6, pac::gpio::vals::Mode::OUTPUT_10MHZ); w.set_cnf(6, pac::gpio::vals::Cnf::PULL_IN__AF_PUSH_PULL_OUT);});


        pac::RCC.ahbpcenr().modify(|w| w.set_dma1en(true));
        pac::RCC.apb2pcenr().modify(|w| w.set_spi1en(true));

        pac::SPI1.ctlr2().write(|w| {w.set_ssoe(false);
                                 w.set_txdmaen(true);
        });
        pac::SPI1.ctlr1().write(|w| {
            w.set_mstr(true); // master
            w.set_br(pac::spi::vals::BaudRate::DIV_16);
            w.set_spe(true); // Enable SPI
            // w.set_lsbfirst(config.lsb_first());
            w.set_ssi(true);
            w.set_ssm(true);
            w.set_crcen(false);
            w.set_bidimode(false); // undirectional output TODO might make sense to use bidimode, output only?
            w.set_rxonly(false);
            w.set_dff(true); // send/receive u16
        });

        Self { }
    }

    #[allow(static_mut_refs)]
    pub async fn write(&mut self, colors: &[Color; LEDS]) {

        unsafe {
            for (signal, color) in SPI_DMA_TRANSFER_BUFFER[2..LEDS + 2].iter_mut().zip(colors) {
                color.gen_grb_data(signal);
            }
        }

        unsafe {hal::interrupt::DMA1_CHANNEL3.enable()};
        let tx_dst = pac::SPI1.datar().as_ptr();
        let ch = pac::DMA1.ch(3-1);
        ch.par().write_value(tx_dst as u32); // PADDR
        unsafe {
        ch.mar().write_value(SPI_DMA_TRANSFER_BUFFER.as_flattened() as *const _ as *const u16 as u32); // MADDR
        ch.ndtr().write(|w| w.set_ndt(SPI_DMA_TRANSFER_BUFFER.as_flattened().len() as u16)); // CNTR
        }
        ch.cr().write(|w| {
            w.set_psize(pac::dma::vals::Size::BITS16);
            w.set_msize(pac::dma::vals::Size::BITS16);
            w.set_minc(true); // Increase memory address
            w.set_dir(pac::dma::vals::Dir::FROMMEMORY);
            w.set_teie(false); // no interrupt on errror
            w.set_tcie(true); // interrupt on tx complete
            w.set_htie(false); // no interrupt half
            w.set_circ(false); // circular
            //w.set_pl(options.priority.into()); // priority
            w.set_en(true); // and start
        });

        SPI_DMA_EVENT.wait().await;
        // Wait for the SPI to be done
        while pac::SPI1.statr().read().bsy() {}
    }
}

#[interrupt]
fn DMA1_CHANNEL3_IRQHandler() {
    pac::DMA1.ifcr().write(|w| w.set_gif(3-1, true));
    SPI_DMA_EVENT.trigger();
}
