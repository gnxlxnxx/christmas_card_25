#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

pub mod matrix;

use panic_halt as _;
use ch32_hal::{self as hal, pac, delay::Delay, gpio::{Level, Output, OutputOpenDrain}, pac::gpio::vals::{Cnf, Mode}, timer::{Channel, low_level::{CountingMode, OutputCompareMode}}};

use crate::matrix::{ButtonPins, LedPins, Matrix};

#[qingke_rt::entry]
fn main() -> ! {
    hal::debug::SDIPrint::enable();
    let p = hal::init(hal::Config::default());
    let mut delay = Delay;
    let led = LedPins::new(p.PD0, p.PA2, p.PA1, p.PD6, p.PD5, p.PD2, p.PC7, p.PC4, p.PC1);
    let btn = ButtonPins::new(p.PD7, p.PD4, p.PC0, p.PD3);
    let mut matrix = Matrix::new(led, btn, p.TIM1, p.TIM2);
    // let mut tim = hal::timer::complementary_pwm::ComplementaryPwm::new(p.TIM1, None, Some(hal::timer::complementary_pwm::ComplementaryPwmPin::new_ch1::<0>(p.PD0)), None, Some(hal::timer::complementary_pwm::ComplementaryPwmPin::new_ch2::<0>(p.PA2)), None, None, None, ch32_hal::time::Hertz::khz(10), ch32_hal::timer::low_level::CountingMode::EdgeAlignedUp);
    // let mut tim = hal::timer::complementary_pwm::ComplementaryPwm::new(p.TIM1, None, Some(hal::timer::complementary_pwm::ComplementaryPwmPin::new_ch1::<0>(p.PD0)), None, None, None, None, None, ch32_hal::time::Hertz::khz(10), ch32_hal::timer::low_level::CountingMode::EdgeAlignedDown);
    // let mut tim = hal::timer::complementary_pwm::ComplementaryPwm::new(p.TIM1, None, None, None, Some(hal::timer::complementary_pwm::ComplementaryPwmPin::new_ch2::<0>(p.PA2)), None, None, None, ch32_hal::time::Hertz::khz(10), ch32_hal::timer::low_level::CountingMode::EdgeAlignedDown);
    // let mut tim1 = hal::timer::low_level::Timer::new(p.TIM1);
    // tim1.set_frequency(ch32_hal::time::Hertz::khz(10));
    // tim1.set_autoreload_preload(true);
    // tim1.set_counting_mode(CountingMode::EdgeAlignedUp);
    // tim1.set_moe(true);
    // tim1.set_output_compare_mode(Channel::Ch1, OutputCompareMode::PwmMode1);
    // tim1.set_output_compare_mode(Channel::Ch2, OutputCompareMode::PwmMode1);
    // tim1.set_output_compare_mode(Channel::Ch4, OutputCompareMode::PwmMode1);
    // tim1.start();
    //
    // tim1.regs_gp16().ccer().write(|w| {
    //     w.set_cce(1, true);
    //     w.set_ccp(1, true);
    //     w.set_ccne(1, true);
    //     w.set_ccnp(1, true);
    //     w.set_ccne(2, true);
    //     w.set_ccnp(2, true);
    // });
    //
    // pac::GPIOA.cfglr().modify(|w| {
    //     w.set_mode(2, Mode::OUTPUT_50MHZ);
    //     w.set_cnf(2, Cnf::AF_OPEN_DRAIN_OUT);
    // });
    // pac::GPIOD.cfglr().modify(|w| {
    //     w.set_mode(0, Mode::OUTPUT_50MHZ);
    //     w.set_cnf(0, Cnf::AF_OPEN_DRAIN_OUT);
    //     // w.set_mode(5, Mode::OUTPUT_50MHZ);
    //     // w.set_cnf(5, Cnf::AF_OPEN_DRAIN_OUT);
    //     // w.set_mode(6,  Mode::OUTPUT_50MHZ);
    //     // w.set_cnf(6,  Cnf::AF_OPEN_DRAIN_OUT);
    //     //
    //     // w.set_mode(3, Mode::OUTPUT_50MHZ);
    //     // w.set_cnf(3, Cnf::ANALOG_IN__PUSH_PULL_OUT);
    //     // w.set_mode(4, Mode::OUTPUT_50MHZ);
    //     // w.set_cnf(4, Cnf::ANALOG_IN__PUSH_PULL_OUT);
    //     // w.set_mode(7, Mode::OUTPUT_50MHZ);
    //     // w.set_cnf(7, Cnf::ANALOG_IN__PUSH_PULL_OUT);
    // });
    //
    // tim1.set_compare_value(Channel::Ch1, 0);
    // tim1.set_compare_value(Channel::Ch2, 32);


    // tim.set_polarity(ch32_hal::timer::Channel::Ch1, ch32_hal::timer::low_level::OutputPolarity::ActiveHigh);
    // tim.set_polarity(ch32_hal::timer::Channel::Ch2, ch32_hal::timer::low_level::OutputPolarity::ActiveHigh);
    //
    // tim.set_duty(ch32_hal::timer::Channel::Ch1, 0);
    // tim.set_duty(ch32_hal::timer::Channel::Ch2, 0);
    //
    // tim.enable(ch32_hal::timer::Channel::Ch1);
    // tim.enable(ch32_hal::timer::Channel::Ch2);
    //
    // let _pin = Output::new(p.PC1, Level::High, Default::default());

    // core::mem::forget(Output::new(p.PD0, Level::High, Default::default()));

    // Adjust the LED GPIO according to your board
    // let mut led2 = OutputOpenDrain::new(p.PA2, Level::Low, Default::default());
    loop {
        matrix.advance();
        // led2.toggle();
        delay.delay_ms(1);
        hal::println!("toggle!");
    }
}
