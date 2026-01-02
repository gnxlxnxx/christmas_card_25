pub mod descriptors;
pub mod usb;
use crate::hal;
use crate::hal::{gpio::Pin, pac, peripherals::*, Peri};

use usb::UsbIf;
static mut I_KEYBOARD: i32 = 0;

pub fn init(
    dp: Peri<'static, PC3>,
    dm: Peri<'static, PC2>,
    afio: &mut hal::pac::afio::Afio,
    exti: &mut hal::pac::exti::Exti,
    systick: &mut hal::pac::systick::Systick,
) -> UsbIf<0x4001_1000usize, 3, 2, 3> {
    systick.ctlr().write(|w| {
        w.set_stclk(pac::systick::vals::Stclk::HCLK);
        w.set_ste(true);
    });

    let mut _usb_dp = hal::gpio::Input::new(dp, hal::gpio::Pull::None);
    let pin_number = dm.pin() as usize;
    let port_number = dm.port();
    let mut _usb_dm = hal::gpio::Input::new(dm, hal::gpio::Pull::None);
    // NOTE needs to have a fixed address
    let usb_if = UsbIf::new(
        |_e, _scratchpad, endp, sendtok, usbif| {
            if endp == 1 {
                let mut tsajoystick_keyboard: [u8; 8] = [0x00; 8];
                // Keyboard (8 bytes)
                unsafe {
                    I_KEYBOARD += 1;

                    // Press a Key every second or so.
                    // if (I_KEYBOARD & 0x7f) == 1 {
                    //     tsajoystick_keyboard[4] = 0x05; // 0x05 = "b"; 0x53 = NUMLOCK; 0x39 = CAPSLOCK;
                    // } else {
                    //     tsajoystick_keyboard[4] = 0;
                    // }
                    usbif.usb_send_data(tsajoystick_keyboard.as_ptr(), 8, 0, sendtok);
                }
            } else {
                // If it's a control transfer, empty it.
                usbif.usb_send_empty(sendtok);
            }
        },
        descriptors::get_descriptor_info,
    );

    afio.exticr()
        .modify(|w| w.set_exti(pin_number, port_number));
    //Warning: The interrupts perform HSI trimming and should run with 48MHz HSI settings
    exti.intenr().write(|w| w.set_mr(pin_number, true)); // enable interrupt
    exti.ftenr().write(|w| w.set_tr(pin_number, true));
    exti.rtenr().write(|w| w.set_tr(pin_number, false));
    afio.exticr()
        .modify(|w| w.set_exti(pin_number, port_number));
    usb_if
}

pub fn usb_up(dpu: Peri<'static, PC5>) {
    let mut usb_dpu = hal::gpio::Output::new(dpu, hal::gpio::Level::Low, hal::gpio::Speed::High);

    usb_dpu.set_high();
}
