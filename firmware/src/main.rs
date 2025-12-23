#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

pub mod drivers;
pub mod util;

use ch32_hal::{self as hal};
use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker, Timer};
use panic_halt as _;

use drivers::{
    buttons::{self, Buttons},
    matrix::{self, Matrix},
};

use util::matrix::MatrixMode;
use util::ws2812;

#[embassy_executor::task]
async fn ws2812_exec(
    pin: hal::Peri<'static, hal::peripherals::PC6>,
    spi1: hal::Peri<'static, hal::peripherals::SPI1>,
    dma1_ch3: hal::Peri<'static, hal::peripherals::DMA1_CH3>,
) {
    let mut ws2812 = drivers::ws2812::Ws2812::new(pin, spi1, dma1_ch3);

    let mut mode = ws2812::Mode::new();
    loop {
        ws2812.run_mode(&mut mode).await;
    }
}

async fn test() {
    for _ in 0..10 {
        Matrix::fb().store(0, 0, 32);
        Timer::after_millis(500).await;
        Matrix::fb().store(0, 0, 0);
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::main(entry = "ch32_hal::entry")]
async fn main(spawner: Spawner) -> ! {
    let p = hal::init(hal::Config {
        rcc: hal::rcc::Config::SYSCLK_FREQ_48MHZ_HSI,
        dma_interrupt_priority: qingke::interrupt::Priority::P0,
    });
    let led = matrix::Pins::new(
        p.PD0, p.PA2, p.PA1, p.PD6, p.PD5, p.PD2, p.PC7, p.PC4, p.PC1,
    );
    let btn = buttons::Pins::new(p.PD7, p.PD4, p.PC0, p.PD3);
    drivers::timer_init(spawner, led, btn, p.TIM1, p.TIM2);
    spawner
        .spawn(ws2812_exec(p.PC6, p.SPI1, p.DMA1_CH3))
        .unwrap();

    let mut clock = Ticker::every(Duration::from_millis(50));

    let mut mode = util::matrix::Mode::new();

    loop {
        let mut snake = util::games::snake::SnakeGame::new();
        snake.run().await;

        //mode.animate().await;
        // util::text::scroll(b"Die Fachschaft Elektro- und Informationstechnik an der \x80 Universit\xE4t Stuttgart w\xFCnscht euch allen recht herzlich ein frohes Weihnachtsfest!", 32, &mut clock).await;
        // util::text::scroll(b" !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~", 32, &mut clock).await;
        // util::text::clear_scroll(&mut clock).await;
        // let event = Buttons::event().await;
        // Matrix::fb().store(3 + event.button as usize, 7, if event.pressed { 32 } else { 0 });
    }
}
