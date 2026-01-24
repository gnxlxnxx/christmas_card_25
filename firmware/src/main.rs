#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

pub mod apps;
mod builtins;
pub mod drivers;
pub mod util;
mod vectors;

use qingke::{interrupt::Priority, pfic};
use ch32_metapac::{self as pac, Interrupt, rcc::vals::Sw};
use panic_halt as _;

use crate::util::ws2812::FilteredWs2812;

unsafe fn init_hsi_pll_48mhz() {
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
    unsafe { init_hsi_pll_48mhz(); }

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

    unsafe { drivers::usb::init(); }

    let ws2812 = unsafe { drivers::ws2812::Ws2812::init() };
    let mut ws2812 = FilteredWs2812::new(ws2812);

    unsafe { drivers::timer_init(); }

    let mut main = apps::Main::new();

    loop {
        main.poll(&mut ws2812);
    }
}
