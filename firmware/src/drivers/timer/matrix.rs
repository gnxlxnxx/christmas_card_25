use core::{
    mem,
    sync::atomic::{AtomicU8, Ordering},
};

use ch32_hal::{
    Peri,
    gpio::{AnyPin, Pin},
    pac::{
        self,
        gpio::vals::{Cnf, Mode},
    },
    peripherals,
};
use embassy_time::Duration;

// Gamma brightness lookup table <https://victornpb.github.io/gamma-table-generator>
// gamma = 2.20 steps = 256 range = 0-255
const GAMMA_LUT: [u8; 256] = [
     0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   1,
     1,   1,   1,   1,   1,   1,   1,   1,   1,   2,   2,   2,   2,   2,   2,   2,
     3,   3,   3,   3,   3,   4,   4,   4,   4,   5,   5,   5,   5,   6,   6,   6,
     6,   7,   7,   7,   8,   8,   8,   9,   9,   9,  10,  10,  11,  11,  11,  12,
    12,  13,  13,  13,  14,  14,  15,  15,  16,  16,  17,  17,  18,  18,  19,  19,
    20,  20,  21,  22,  22,  23,  23,  24,  25,  25,  26,  26,  27,  28,  28,  29,
    30,  30,  31,  32,  33,  33,  34,  35,  35,  36,  37,  38,  39,  39,  40,  41,
    42,  43,  43,  44,  45,  46,  47,  48,  49,  49,  50,  51,  52,  53,  54,  55,
    56,  57,  58,  59,  60,  61,  62,  63,  64,  65,  66,  67,  68,  69,  70,  71,
    73,  74,  75,  76,  77,  78,  79,  81,  82,  83,  84,  85,  87,  88,  89,  90,
    91,  93,  94,  95,  97,  98,  99, 100, 102, 103, 105, 106, 107, 109, 110, 111,
   113, 114, 116, 117, 119, 120, 121, 123, 124, 126, 127, 129, 130, 132, 133, 135,
   137, 138, 140, 141, 143, 145, 146, 148, 149, 151, 153, 154, 156, 158, 159, 161,
   163, 165, 166, 168, 170, 172, 173, 175, 177, 179, 181, 182, 184, 186, 188, 190,
   192, 194, 196, 197, 199, 201, 203, 205, 207, 209, 211, 213, 215, 217, 219, 221,
   223, 225, 227, 229, 231, 234, 236, 238, 240, 242, 244, 246, 248, 251, 253, 255,
];

pub const FRAME_DURATION: Duration = Duration::from_ticks(2 * Framebuffer::HEIGHT as u64);

pub(super) const ROWS: usize = 9;
const DEBUG_BRIGHTNESS: u8 = 32;

static FB: Framebuffer = Framebuffer::new();

#[derive(Debug)]
pub struct Framebuffer(pub [[AtomicU8; Self::WIDTH]; Self::HEIGHT]);

impl Framebuffer {
    pub const WIDTH: usize = (ROWS - 1);
    pub const HEIGHT: usize = ROWS;

    pub const fn new() -> Self {
        Self::new_filled(0)
    }

    pub const fn new_filled(val: u8) -> Self {
        Self::from_u8_array([[val; Self::WIDTH]; Self::HEIGHT])
    }

    pub const fn from_u8_array(arr: [[u8; Self::WIDTH]; Self::HEIGHT]) -> Self {
        Self(unsafe { mem::transmute::<[[u8; 8]; 9], [[AtomicU8; 8]; 9]>(arr) })
    }

    pub fn load(&self, x: usize, y: usize) -> u8 {
        self.0[y][x].load(Ordering::Relaxed)
    }

    pub fn try_load(&self, x: usize, y: usize) -> Option<u8> {
        Some(self.0.get(y)?.get(x)?.load(Ordering::Relaxed))
    }

    pub fn store(&self, x: usize, y: usize, val: u8) {
        self.0[y][x].store(val, Ordering::Relaxed)
    }

    pub fn set_all(&self, val: u8) {
        for row in &self.0 {
            for pixel in row {
                pixel.store(val, Ordering::Relaxed);
            }
        }
    }

    pub fn clear_all(&self) {
        self.set_all(0);
    }

    // Debug output
    pub fn show_u8(&self, y: usize, mut val: u8) {
        for i in (0..8).rev() {
            self.store(i, y, if val & 1 != 0 { DEBUG_BRIGHTNESS } else { 0 });
            val >>= 1;
        }
    }
    pub fn show_u16(&self, y: usize, val: u16) {
        self.show_u8(y, (val >> 8) as u8);
        self.show_u8(y + 1, val as u8);
    }

    pub(super) fn get_pwm(&self, col: usize, row: usize, cycles: u16) -> u16 {
        let x = if col <= row { col } else { col - 1 };

        cycles - ((GAMMA_LUT[self.try_load(x, row).unwrap_or(0) as usize] as u16) << 4)
    }
}

pub struct Pins<'a>([Peri<'a, AnyPin>; ROWS]);

impl<'a> Pins<'a> {
    pub fn new(
        led1: Peri<'a, peripherals::PD0>,
        led2: Peri<'a, peripherals::PA2>,
        led3: Peri<'a, peripherals::PA1>,
        led4: Peri<'a, peripherals::PD6>,
        led5: Peri<'a, peripherals::PD5>,
        led6: Peri<'a, peripherals::PD2>,
        led7: Peri<'a, peripherals::PC7>,
        led8: Peri<'a, peripherals::PC4>,
        led9: Peri<'a, peripherals::PC1>,
    ) -> Self {
        // Set all pins high
        pac::GPIOA.bshr().write(|w| {
            w.set_bs(1, true);
            w.set_bs(2, true);
        });
        pac::GPIOC.bshr().write(|w| {
            w.set_bs(1, true);
            w.set_bs(4, true);
            w.set_bs(7, true);
        });
        pac::GPIOD.bshr().write(|w| {
            w.set_bs(0, true);
            w.set_bs(2, true);
            w.set_bs(5, true);
            w.set_bs(6, true);
        });

        Self([
            led1.into(),
            led2.into(),
            led3.into(),
            led4.into(),
            led5.into(),
            led6.into(),
            led7.into(),
            led8.into(),
            led9.into(),
        ])
    }

    pub(super) fn set_float_all(&mut self) {
        pac::GPIOA.cfglr().modify(|w| {
            w.set_mode(1, Mode::INPUT);
            w.set_cnf(1, Cnf::FLOATING_IN__OPEN_DRAIN_OUT);
            w.set_mode(2, Mode::INPUT);
            w.set_cnf(2, Cnf::FLOATING_IN__OPEN_DRAIN_OUT);
        });
        critical_section::with(|_| {
            pac::GPIOC.cfglr().modify(|w| {
                w.set_mode(1, Mode::INPUT);
                w.set_cnf(1, Cnf::FLOATING_IN__OPEN_DRAIN_OUT);
                w.set_mode(4, Mode::INPUT);
                w.set_cnf(4, Cnf::FLOATING_IN__OPEN_DRAIN_OUT);
                w.set_mode(7, Mode::INPUT);
                w.set_cnf(7, Cnf::FLOATING_IN__OPEN_DRAIN_OUT);
            });
        });
        pac::GPIOD.cfglr().modify(|w| {
            w.set_mode(0, Mode::INPUT);
            w.set_cnf(0, Cnf::FLOATING_IN__OPEN_DRAIN_OUT);
            w.set_mode(2, Mode::INPUT);
            w.set_cnf(2, Cnf::FLOATING_IN__OPEN_DRAIN_OUT);
            w.set_mode(5, Mode::INPUT);
            w.set_cnf(5, Cnf::FLOATING_IN__OPEN_DRAIN_OUT);
            w.set_mode(6, Mode::INPUT);
            w.set_cnf(6, Cnf::FLOATING_IN__OPEN_DRAIN_OUT);
        });
    }

    pub(super) fn set_high(&mut self, row: usize) {
        let pin = &self.0[row];

        let port = pin.port().into();
        let pin = pin.pin().into();

        critical_section::with(|_| {
            pac::GPIO(port).cfglr().modify(|w| {
                w.set_mode(pin, Mode::OUTPUT_50MHZ);
                w.set_cnf(pin, Cnf::ANALOG_IN__PUSH_PULL_OUT);
            });
        });
    }
}

pub struct Matrix;

impl Matrix {
    pub fn fb() -> &'static Framebuffer {
        &FB
    }
}
