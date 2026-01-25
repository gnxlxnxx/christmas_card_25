pub mod descriptors;
pub mod rv003usb;
use crate::drivers::buttons::{Button, Buttons};
use ch32_metapac::{
    self as pac, Interrupt,
    gpio::vals::{Cnf, Mode},
};
use qingke::pfic;
use qingke_rt::interrupt;
use rv003usb::UsbIf;

const USB_PORT: usize = 2;
const USB_DP_PIN: usize = 3;
const USB_DM_PIN: usize = 2;
const USB_DPU_PORT: usize = USB_PORT;
const USB_DPU_PIN: usize = 5;

// This is GPIOC, but i haven't figured out how to do this nicely yet
static mut USB_IF: UsbIf<0x4001_1000usize, USB_DP_PIN, USB_DM_PIN, 3> = UsbIf::new(
    |_e, _scratchpad, endp, sendtok, usbif| {
        if endp == 1 {
            let mut tsajoystick_keyboard = [0u8; 8];
            let mut nextkc = 2;

            // Keyboard (8 bytes)
            if Buttons::get(Button::R) {
                tsajoystick_keyboard[nextkc] = 0x4f; // Right
                nextkc += 1;
            }
            if Buttons::get(Button::L) {
                tsajoystick_keyboard[nextkc] = 0x50; // Left
            }
            unsafe {
                usbif.usb_send_data(
                    tsajoystick_keyboard.as_ptr(),
                    tsajoystick_keyboard.len() as u32,
                    0,
                    sendtok,
                );
            }
        } else {
            // If it's a control transfer, empty it.
            usbif.usb_send_empty(sendtok);
        }
    },
    descriptors::get_descriptor_info,
);

#[interrupt]
fn EXTI7_0_IRQHandler() {
    // IMPORTANT: Keep latency low here
    unsafe {
        #[allow(static_mut_refs)]
        USB_IF.usb_interrupt_handler();
    }
}

pub unsafe fn init() {
    pac::SYSTICK.ctlr().write(|w| {
        w.set_stclk(pac::systick::vals::Stclk::HCLK);
        w.set_ste(true);
    });

    let port_number = USB_PORT as u8;
    let pin_number = USB_DM_PIN;
    pac::AFIO
        .exticr()
        .write(|w| w.set_exti(pin_number, port_number));
    // Warning: The interrupts perform HSI trimming and should run with 48MHz HSI settings
    pac::EXTI.intenr().write(|w| w.set_mr(pin_number, true)); // enable interrupt
    pac::EXTI.ftenr().write(|w| w.set_tr(pin_number, true));
    pac::EXTI.rtenr().write(|w| w.set_tr(pin_number, false));

    unsafe { pfic::enable_interrupt(Interrupt::EXTI7_0 as u8); }

    pac::GPIO(USB_DPU_PORT).cfglr().modify(|w| {
        w.set_mode(USB_DPU_PIN, Mode::OUTPUT_50MHZ);
        w.set_cnf(USB_DPU_PIN, Cnf::ANALOG_IN__PUSH_PULL_OUT);
    });
    pac::GPIO(USB_DPU_PORT).bshr().write(|w| w.set_bs(USB_DPU_PIN, true));
}
