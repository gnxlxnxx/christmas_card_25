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
use embassy_executor::Spawner;
use hal::gpio::Pin;
use hal::pac;
use panic_halt as _;
use usb::descriptors;
use usb::usb::UsbIf;

use drivers::{
    buttons::{self},
    matrix::{self},
};

use crate::apps::main;

use core::mem::MaybeUninit;
// This is GPIOD, but i haven't figured out how to do this nicely yet
static mut USB_IF: MaybeUninit<UsbIf<0x4001_1000usize, 3, 2, 3>> = MaybeUninit::uninit();

static mut I_MOUSE: i32 = 0;
static mut I_KEYBOARD: i32 = 0;

#[embassy_executor::main(entry = "ch32_hal::entry")]
async fn main(_spawner: Spawner) -> ! {
    let p = hal::init(hal::Config {
        rcc: hal::rcc::Config::SYSCLK_FREQ_48MHZ_HSI,
        dma_interrupt_priority: qingke::interrupt::Priority::P0,
    });

    pac::SYSTICK.ctlr().write(|w| {
        w.set_stclk(pac::systick::vals::Stclk::HCLK);
        w.set_ste(true);
    });

    let mut _usb_dp = hal::gpio::Input::new(p.PC3, hal::gpio::Pull::None);
    let pin_number = p.PC2.pin() as usize;
    let port_number = p.PC2.port();
    let mut _usb_dm = hal::gpio::Input::new(p.PC2, hal::gpio::Pull::None);
    let mut usb_dpu = hal::gpio::Output::new(p.PC5, hal::gpio::Level::Low, hal::gpio::Speed::High);
    // NOTE needs to have a fixed address
    let mut usb = UsbIf::new(
        |_e, _scratchpad, endp, sendtok, usbif| {
            if endp == 1 {
                let mut tsajoystick_mouse: [u8; 4] = [0x00, 0x00, 0x00, 0x00];
                // Mouse (4 bytes)
                unsafe {
                    I_MOUSE += 1;
                    let mut mode = I_MOUSE >> 2;

                    // Move the mouse right, down, left and up in a square.
                    if I_MOUSE & 0b11 == 0 {
                        match mode & 3 {
                            0 => {
                                tsajoystick_mouse[1] = 1;
                                tsajoystick_mouse[2] = 0;
                            }
                            1 => {
                                tsajoystick_mouse[1] = 0;
                                tsajoystick_mouse[2] = 1;
                            }
                            2 => {
                                tsajoystick_mouse[1] = -1i8 as u8; // Need to cast to u8 for the array
                                tsajoystick_mouse[2] = 0;
                            }
                            3 => {
                                tsajoystick_mouse[1] = 0;
                                tsajoystick_mouse[2] = -1i8 as u8; // Need to cast to u8 for the array
                            }
                            _ => {}
                        }
                    }
                    usbif.usb_send_data(tsajoystick_mouse.as_ptr(), 4, 0, sendtok);
                }
            } else if endp == 2 {
                let mut tsajoystick_keyboard: [u8; 8] = [0x00; 8];
                // Keyboard (8 bytes)
                unsafe {
                    //I_KEYBOARD += 1;

                    // Press a Key every second or so.
                    if (I_KEYBOARD & 0x7f) == 1 {
                        tsajoystick_keyboard[4] = 0x05; // 0x05 = "b"; 0x53 = NUMLOCK; 0x39 = CAPSLOCK;
                    } else {
                        tsajoystick_keyboard[4] = 0;
                    }
                    usbif.usb_send_data(tsajoystick_keyboard.as_ptr(), 8, 0, sendtok);
                }
            } else {
                // If it's a control transfer, empty it.
                usbif.usb_send_empty(sendtok);
            }
        },
        descriptors::get_descriptor_info,
    );

    #[allow(static_mut_refs)]
    unsafe {USB_IF.write(usb)};

    let exti = &hal::pac::EXTI;
    let afio = &hal::pac::AFIO;
    afio.exticr()
        .modify(|w| w.set_exti(pin_number, port_number));
    //Warning: The interrupts perform HSI trimming and should run with 48MHz HSI settings
    exti.intenr().write(|w| w.set_mr(pin_number, true)); // enable interrupt
    exti.ftenr().write(|w| w.set_tr(pin_number, true));
    exti.rtenr().write(|w| w.set_tr(pin_number, false));
    afio.exticr()
        .modify(|w| w.set_exti(pin_number, port_number));
    // TODO set interrupt activity high

    // EXTI7_0 is already enabled by the hal
    // unsafe {hal::interrupt::EXTI7_0.enable()};
    // USB setup done
    let led = matrix::Pins::new(
        p.PD0, p.PA2, p.PA1, p.PD6, p.PD5, p.PD2, p.PC7, p.PC4, p.PC1,
    );
    let btn = buttons::Pins::new(p.PD7, p.PD4, p.PC0, p.PD3);
    drivers::timer_init(led, btn, p.TIM1, p.TIM2);

    let mut ws2812 = drivers::ws2812::Ws2812::new(p.PC6, p.SPI1, p.DMA1_CH3);

    // WS2812 uses some DMA interrupts internally
    // Make the EXTI interrupt preempt all others, otherwise it might now work
    hal::interrupt::DMA1_CHANNEL1.set_priority(hal::interrupt::Priority::P15);
    hal::interrupt::DMA1_CHANNEL2.set_priority(hal::interrupt::Priority::P15);
    hal::interrupt::DMA1_CHANNEL3.set_priority(hal::interrupt::Priority::P15);
    hal::interrupt::DMA1_CHANNEL4.set_priority(hal::interrupt::Priority::P15);
    hal::interrupt::DMA1_CHANNEL5.set_priority(hal::interrupt::Priority::P15);
    hal::interrupt::DMA1_CHANNEL6.set_priority(hal::interrupt::Priority::P15);
    hal::interrupt::DMA1_CHANNEL7.set_priority(hal::interrupt::Priority::P15);
    hal::interrupt::EXTI7_0.set_priority(hal::interrupt::Priority::P0);
    hal::interrupt::TIM1_UP.set_priority(hal::interrupt::Priority::P15);
    unsafe { hal::interrupt::EXTI7_0.enable() };

    usb_dpu.set_high();

    loop {
        main::run(&mut ws2812).await;
    }
}

use ch32_hal::interrupt;

#[interrupt]
#[allow(static_mut_refs)]
fn EXTI7_0_IRQHandler() {
    // IMPORTANT: Keep latency low here
    unsafe { USB_IF.assume_init_mut().usb_interrupt_handler() };
}
