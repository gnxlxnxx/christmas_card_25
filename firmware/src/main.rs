#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

pub mod matrix;

use core::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize};

use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_halt as _;
use ch32_hal::{self as hal, delay::Delay, gpio::{Level, Output, OutputOpenDrain}, pac::{self, gpio::vals::{Cnf, Mode}, rcc::vals::{Hpre, Pllsrc, Ppre, Sw}}, timer::{Channel, low_level::{CountingMode, OutputCompareMode}}};

use crate::matrix::{Button, ButtonPins, Framebuffer, LedPins, Matrix};

#[embassy_executor::main(entry = "ch32_hal::entry")]
async fn main(spawner: Spawner) -> ! {
    let p = hal::init(hal::Config {
        rcc: hal::rcc::Config::SYSCLK_FREQ_48MHZ_HSI,
        dma_interrupt_priority: qingke::interrupt::Priority::P0
    });
    let led = LedPins::new(p.PD0, p.PA2, p.PA1, p.PD6, p.PD5, p.PD2, p.PC7, p.PC4, p.PC1);
    let btn = ButtonPins::new(p.PD7, p.PD4, p.PC0, p.PD3);
    let matrix = Matrix::init(spawner, led, btn, p.TIM1, p.TIM2);

    // Adjust the LED GPIO according to your board
    loop {
        // matrix.fb().store(0, 7, if matrix.btn(Button::Start) { 32 } else { 0 });
        // matrix.fb().store(1, 7, if matrix.btn(Button::Select) { 32 } else { 0 });
        // matrix.fb().store(0, 8, if matrix.btn(Button::L) { 32 } else { 0 });
        // matrix.fb().store(1, 8, if matrix.btn(Button::R) { 32 } else { 0 });
        // Timer::after_millis(1).await;
        // matrix.fb().store(4, 8, 32);
        // Timer::after_millis(500).await;
        // matrix.fb().store(4, 8, 0);
        // Timer::after_millis(500).await;
        let event = matrix.btn_event().await;
        matrix.fb().store(3 + event.0 as usize, 7, if event.1 { 32 } else { 0 });
    }
}
