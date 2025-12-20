#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

pub mod matrix;

use core::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize};

use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_halt as _;
use ch32_hal::{self as hal, delay::Delay, gpio::{Level, Output, OutputOpenDrain}, pac::{self, gpio::vals::{Cnf, Mode}, rcc::vals::{Hpre, Pllsrc, Ppre, Sw}}, timer::{Channel, low_level::{CountingMode, OutputCompareMode}}};

use crate::matrix::{ButtonPins, LedPins, Matrix, MatrixFb};

#[embassy_executor::main(entry = "ch32_hal::entry")]
async fn main(spawner: Spawner) -> ! {
    let p = hal::init(hal::Config {
        rcc: hal::rcc::Config::SYSCLK_FREQ_48MHZ_HSI,
        dma_interrupt_priority: qingke::interrupt::Priority::P0
    });
    let led = LedPins::new(p.PD0, p.PA2, p.PA1, p.PD6, p.PD5, p.PD2, p.PC7, p.PC4, p.PC1);
    let btn = ButtonPins::new(p.PD7, p.PD4, p.PC0, p.PD3);
    let matrix = Matrix::new(led, btn, p.TIM1, p.TIM2);

    // Adjust the LED GPIO according to your board
    loop {
        matrix.fb().store(MatrixFb::WIDTH - 1, MatrixFb::HEIGHT - 1, 32);
        Timer::after_millis(500).await;
        matrix.fb().store(MatrixFb::WIDTH - 1, MatrixFb::HEIGHT - 1, 0);
        Timer::after_millis(500).await;
    }
}
