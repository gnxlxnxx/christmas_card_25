#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

pub mod apps;
mod builtins;
pub mod drivers;
pub mod util;
mod vectors;

use ch32_hal as hal;
use qingke::{interrupt::Priority, pfic};
use ch32_metapac::{self as pac, Interrupt, rcc::vals::Sw};
use embassy_futures::block_on;
use panic_halt as _;

use drivers::{
    buttons::{self},
    matrix::{self},
};

use crate::apps::{games, main};
use crate::util::ws2812::FilteredWs2812;

fn init_hsi_pll_48mhz() {
    pac::RCC.ctlr().write(|w| {
        w.set_hsitrim(16);
        w.set_hsion(true);
        w.set_pllon(true);
    });
    while !pac::RCC.ctlr().read().pllrdy() {}
    pac::FLASH.actlr().write(|w| w.set_latency(1));
    pac::RCC.cfgr0().write(|w| w.set_sw(Sw::PLL));
    while pac::RCC.cfgr0().read().sws() != Sw::PLL {}
}

#[qingke_rt::entry]
fn main() -> ! {
    init_hsi_pll_48mhz();

    // Enable peripherals
    pac::RCC.apb2pcenr().write(|w| {
        w.set_spi1en(true);
        w.set_tim1en(true);
        w.set_iopden(true);
        w.set_iopcen(true);
        w.set_iopaen(true);
        w.set_afioen(true);
    });
    pac::RCC.apb1pcenr().write(|w| {
        w.set_tim2en(true)
    });

    unsafe {
        pfic::set_priority(Interrupt::EXTI7_0 as u8, Priority::P0.into());
        pfic::set_priority(Interrupt::TIM1_UP as u8, Priority::P8.into());
    }

    let p = unsafe { hal::Peripherals::steal() };

    let led = matrix::Pins::new(
        p.PD0, p.PA2, p.PA1, p.PD6, p.PD5, p.PD2, p.PC7, p.PC4, p.PC1,
    );
    let btn = buttons::Pins::new(p.PD7, p.PD4, p.PC0, p.PD3);
    drivers::timer_init(led, btn, p.TIM1, p.TIM2);

    let ws2812 = unsafe { drivers::ws2812::Ws2812::init() };
    let mut filt_ws2812 = FilteredWs2812::new(ws2812);

    drivers::usb::init(p.PC3, p.PC2, p.AFIO, p.SYSTICK);
    drivers::usb::usb_up(p.PC5);

    block_on(async {
        loop {
            main::run(&mut filt_ws2812).await;
            games::run(&mut filt_ws2812).await;
        }
    })
}
