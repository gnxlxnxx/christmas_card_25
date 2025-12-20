#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

pub mod drivers;
pub mod ws2812;

use embassy_time::Timer;
use panic_halt as _;
use embassy_executor::Spawner;
use ch32_hal::{self as hal};

use drivers::{matrix::{self, Matrix}, buttons::{self, Buttons}};

use crate::ws2812::{Color, HUETABLE, RANDS, SINTABLE};

#[embassy_executor::task]
async fn ws2812_exec(
    pin: hal::Peri<'static, hal::peripherals::PC6>,
    spi1: hal::Peri<'static, hal::peripherals::SPI1>,
    dma1_ch3: hal::Peri<'static, hal::peripherals::DMA1_CH3>,
) {
    let mut ws2812 = ws2812::Ws2812::init(pin, spi1, dma1_ch3);

    let mut phases: [u16; 6] = [0; 6];
    for i in 0..6 {
        phases[i] = (i as u16) << 8;
    }

    let mut desired_output: [Color; 6] = [Color::new(0, 0, 0); 6];
    let mut output: [Color; 6] = [Color::new(0, 0, 0); 6];

    loop {
        for k in 0..6 {
            phases[k] += (((RANDS[k] as u16 + 0xf) << 2) + ((RANDS[k] as u16 + 0xf) << 1)) >> 1;
        }

        for ledno in 0..6 {
            // TODO add more modes

            // Original "Fire" mode
            let index: usize = ((phases[ledno]) >> 8) as usize;
            let rs: u8 = SINTABLE[index] >> 3;

            desired_output[ledno].r = (HUETABLE[((rs + 30) & 0xff) as usize] as u32 >> 2) as u8;
            desired_output[ledno].g = (HUETABLE[(rs + 0) as usize] as u32 >> 3) as u8;
            desired_output[ledno].b = (HUETABLE[((rs + 190) & 0xff) as usize] as u32 >> 3) as u8;

            if output[ledno].r > desired_output[ledno].r {
                output[ledno].r -= 1;
            } else if output[ledno].r < desired_output[ledno].r {
                output[ledno].r += 1;
            }

            if output[ledno].g > desired_output[ledno].g {
                output[ledno].g -= 1;
            } else if output[ledno].g < desired_output[ledno].g {
                output[ledno].g += 1;
            }

            if output[ledno].b > desired_output[ledno].b {
                output[ledno].b -= 1;
            } else if output[ledno].b < desired_output[ledno].b {
                output[ledno].b += 1;
            }
        }
        ws2812.start(output).await;
        Timer::after_millis(30).await;
    }
}

#[embassy_executor::main(entry = "ch32_hal::entry")]
async fn main(spawner: Spawner) -> ! {
    let p = hal::init(hal::Config {
        rcc: hal::rcc::Config::SYSCLK_FREQ_48MHZ_HSI,
        dma_interrupt_priority: qingke::interrupt::Priority::P0,
    });
    let led = matrix::Pins::new(p.PD0, p.PA2, p.PA1, p.PD6, p.PD5, p.PD2, p.PC7, p.PC4, p.PC1);
    let btn = buttons::Pins::new(p.PD7, p.PD4, p.PC0, p.PD3);
    drivers::timer_init(spawner, led, btn, p.TIM1, p.TIM2);
    spawner.spawn(ws2812_exec(p.PC6, p.SPI1, p.DMA1_CH3)).unwrap();

    loop {
        let event = Buttons::event().await;
        Matrix::fb().store(3 + event.button as usize, 7, if event.pressed { 32 } else { 0 });
    }
}
