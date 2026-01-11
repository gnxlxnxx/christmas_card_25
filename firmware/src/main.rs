#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

pub mod apps;
mod builtins;
pub mod drivers;
pub mod util;
mod vectors;

use ch32_hal::interrupt::InterruptExt;
use ch32_hal::{self as hal};
use embassy_futures::block_on;
use panic_halt as _;

use drivers::{
    buttons::{self},
    matrix::{self},
};

use crate::apps::{games, main};
use crate::util::ws2812::FilteredWs2812;

#[qingke_rt::entry]
fn main() -> ! {
    let p = hal::init(hal::Config {
        rcc: hal::rcc::Config::SYSCLK_FREQ_48MHZ_HSI,
    });

    hal::interrupt::EXTI7_0.set_priority(hal::interrupt::Priority::P0);
    hal::interrupt::TIM1_UP.set_priority(hal::interrupt::Priority::P8);

    let led = matrix::Pins::new(
        p.PD0, p.PA2, p.PA1, p.PD6, p.PD5, p.PD2, p.PC7, p.PC4, p.PC1,
    );
    let btn = buttons::Pins::new(p.PD7, p.PD4, p.PC0, p.PD3);
    drivers::timer_init(led, btn, p.TIM1, p.TIM2);

    let mut ws2812 = drivers::ws2812::Ws2812::new(p.PC6, p.SPI1, p.DMA1_CH3);
    let mut filt_ws2812 = FilteredWs2812::new(&mut ws2812);

    drivers::usb::init(p.PC3, p.PC2, p.AFIO, p.SYSTICK);
    drivers::usb::usb_up(p.PC5);

    block_on(async {
        loop {
            main::run(&mut filt_ws2812).await;
            games::run(&mut filt_ws2812).await;
        }
    })
}
