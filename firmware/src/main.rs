#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

pub mod matrix;

use core::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize};

use panic_halt as _;
use ch32_hal::{self as hal, delay::Delay, gpio::{Level, Output, OutputOpenDrain}, pac::{self, gpio::vals::{Cnf, Mode}, rcc::vals::{Hpre, Pllsrc, Ppre, Sw}}, timer::{Channel, low_level::{CountingMode, OutputCompareMode}}};

use crate::matrix::{ButtonPins, LedPins, Matrix};

static I: (AtomicUsize, AtomicBool) = (AtomicUsize::new(0), AtomicBool::new(false));

#[qingke_rt::entry]
fn main() -> ! {
    let p = hal::init(hal::Config {
        rcc: hal::rcc::Config::SYSCLK_FREQ_48MHZ_HSI,
        dma_interrupt_priority: qingke::interrupt::Priority::P0
    });
    let led = LedPins::new(p.PD0, p.PA2, p.PA1, p.PD6, p.PD5, p.PD2, p.PC7, p.PC4, p.PC1);
    let btn = ButtonPins::new(p.PD7, p.PD4, p.PC0, p.PD3);
    let matrix = Matrix::new(led, btn, p.TIM1, p.TIM2);

    loop {
        for i in 0..9 {
            I.0.store(i, core::sync::atomic::Ordering::Relaxed);
            I.1.store(false, core::sync::atomic::Ordering::Relaxed);
            Delay.delay_ms(1000);
            I.1.store(true, core::sync::atomic::Ordering::Relaxed);
            Delay.delay_ms(1000);
        }
        Delay.delay_ms(1);
    }
}
