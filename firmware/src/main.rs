#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

pub mod drivers;


use panic_halt as _;
use embassy_executor::Spawner;
use ch32_hal::{self as hal};

use drivers::{matrix::{self, Matrix}, buttons::{self, Buttons}};

#[embassy_executor::main(entry = "ch32_hal::entry")]
async fn main(spawner: Spawner) -> ! {
    let p = hal::init(hal::Config {
        rcc: hal::rcc::Config::SYSCLK_FREQ_48MHZ_HSI,
        dma_interrupt_priority: qingke::interrupt::Priority::P0
    });
    let led = matrix::Pins::new(p.PD0, p.PA2, p.PA1, p.PD6, p.PD5, p.PD2, p.PC7, p.PC4, p.PC1);
    let btn = buttons::Pins::new(p.PD7, p.PD4, p.PC0, p.PD3);
    drivers::timer_init(spawner, led, btn, p.TIM1, p.TIM2);

    loop {
        let event = Buttons::event().await;
        Matrix::fb().store(3 + event.button as usize, 7, if event.pressed { 32 } else { 0 });
    }
}
