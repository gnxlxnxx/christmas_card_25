#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

pub mod apps;
pub mod drivers;
mod usb;
pub mod util;
mod vectors;

use ch32_hal::interrupt::InterruptExt;
use ch32_hal::{self as hal};
use embassy_futures::block_on;
use panic_halt as _;
use usb::descriptors;
use usb::usb::UsbIf;

use drivers::{
    buttons::{self},
    matrix::{self},
};

use crate::{apps::{games, main}, drivers::matrix::Matrix};

use core::mem::MaybeUninit;
// This is GPIOD, but i haven't figured out how to do this nicely yet
static mut USB_IF: MaybeUninit<UsbIf<0x4001_1000usize, 3, 2, 3>> = MaybeUninit::uninit();

#[qingke_rt::entry]
fn main() -> ! {
    let p = hal::init(hal::Config {
        rcc: hal::rcc::Config::SYSCLK_FREQ_48MHZ_HSI,
        dma_interrupt_priority: qingke::interrupt::Priority::P0,
    });

    // It's enabled by the hal
    // We don't want an interrupt before we're ready
    unsafe { hal::interrupt::EXTI7_0.disable() };

    let usb = usb::init(p.PC3, p.PC2, &mut hal::pac::AFIO, &mut hal::pac::EXTI, &mut hal::pac::SYSTICK);
    #[allow(static_mut_refs)]
    unsafe {USB_IF.write(usb)};
    // USB setup done

    let led = matrix::Pins::new(
        p.PD0, p.PA2, p.PA1, p.PD6, p.PD5, p.PD2, p.PC7, p.PC4, p.PC1,
    );
    let btn = buttons::Pins::new(p.PD7, p.PD4, p.PC0, p.PD3);
    drivers::timer_init(led, btn, p.TIM1, p.TIM2);

    let mut ws2812 = drivers::ws2812::Ws2812::new(p.PC6, p.SPI1, p.DMA1_CH3);

    unsafe { hal::interrupt::EXTI7_0.enable() };

    usb::usb_up(p.PC5);

    block_on(async {
        loop {
            main::run(&mut ws2812).await;
            games::run(&mut ws2812).await;
        }
    })
}

use ch32_hal::interrupt;

#[interrupt]
#[allow(static_mut_refs)]
fn EXTI7_0_IRQHandler() {
    // IMPORTANT: Keep latency low here
    unsafe { USB_IF.assume_init_mut().usb_interrupt_handler() };
}
