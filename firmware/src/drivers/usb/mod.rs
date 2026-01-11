pub mod descriptors;
pub mod usb;
use crate::drivers::buttons::{Button, Buttons};
use crate::hal;
use crate::hal::{Peri, gpio::Pin, pac, peripherals::*};

use ch32_hal::interrupt::InterruptExt;
use qingke_rt::interrupt;
use usb::UsbIf;

use core::mem::MaybeUninit;
// This is GPIOD, but i haven't figured out how to do this nicely yet
static mut USB_IF: MaybeUninit<UsbIf<0x4001_1000usize, 3, 2, 3>> = MaybeUninit::uninit();

#[interrupt]
fn EXTI7_0_IRQHandler() {
    // IMPORTANT: Keep latency low here
    #[allow(static_mut_refs)]
    unsafe {
        USB_IF.assume_init_mut().usb_interrupt_handler()
    };
}

pub fn init(
    dp: Peri<'static, PC3>,
    dm: Peri<'static, PC2>,
    _afio: Peri<'static, AFIO>,
    _systick: Peri<'static, SYSTICK>,
) {
    pac::SYSTICK.ctlr().write(|w| {
        w.set_stclk(pac::systick::vals::Stclk::HCLK);
        w.set_ste(true);
    });

    let mut _usb_dp = hal::gpio::Input::new(dp, hal::gpio::Pull::None);
    let pin_number = dm.pin() as usize;
    let port_number = dm.port();
    let mut _usb_dm = hal::gpio::Input::new(dm, hal::gpio::Pull::None);
    let usb_if = UsbIf::new(
        |_e, _scratchpad, endp, sendtok, usbif| {
            if endp == 1 {
                let mut tsajoystick_keyboard: [u8; 8] = [0x00; 8];
                let mut nextkc = tsajoystick_keyboard.iter_mut().skip(2);
                // Keyboard (8 bytes)
                if Buttons::get(Button::R) {
                    *nextkc.next().unwrap() = 0x4f; // Right
                }
                if Buttons::get(Button::L) {
                    *nextkc.next().unwrap() = 0x50; // Left
                }
                unsafe {
                    usbif.usb_send_data(tsajoystick_keyboard.as_ptr(), 8, 0, sendtok);
                }
            } else {
                // If it's a control transfer, empty it.
                usbif.usb_send_empty(sendtok);
            }
        },
        descriptors::get_descriptor_info,
    );

    pac::AFIO
        .exticr()
        .write(|w| w.set_exti(pin_number, port_number));
    //Warning: The interrupts perform HSI trimming and should run with 48MHz HSI settings
    pac::EXTI.intenr().write(|w| w.set_mr(pin_number, true)); // enable interrupt
    pac::EXTI.ftenr().write(|w| w.set_tr(pin_number, true));
    pac::EXTI.rtenr().write(|w| w.set_tr(pin_number, false));

    unsafe {
        #[allow(static_mut_refs)]
        USB_IF.write(usb_if);

        hal::interrupt::EXTI7_0.enable();
    }
}

pub fn usb_up(dpu: Peri<'static, PC5>) {
    let mut usb_dpu = hal::gpio::Output::new(dpu, hal::gpio::Level::Low, hal::gpio::Speed::High);

    usb_dpu.set_high();
}
