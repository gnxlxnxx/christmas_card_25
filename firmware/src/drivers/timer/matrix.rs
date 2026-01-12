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

// from itertools import batched
// GAMMA = 2.2
// STEPS = 256
// MAX = 4095
// values = (round(MAX * (i / (STEPS - 1)) ** GAMMA) for i in range(STEPS))
// entries = (f'[{a>>4:3}, {b>>4:3}, 0x{(b&0xf)<<4 | (a&0xf):02x}],' for a, b in batched(values, 2))
// for line_entries in batched(entries, 4):
//     print('    ' + ' '.join(line_entries))
const GAMMA_LUT: [[u8; 3]; 128] = [
    [  0,   0, 0x00], [  0,   0, 0x00], [  0,   0, 0x10], [  0,   0, 0x21],
    [  0,   0, 0x32], [  0,   0, 0x43], [  0,   0, 0x65], [  0,   0, 0x87],
    [  0,   0, 0xb9], [  0,   0, 0xec], [  0,   1, 0x1f], [  1,   1, 0x53],
    [  1,   1, 0x97], [  1,   1, 0xdb], [  2,   2, 0x20], [  2,   2, 0x85],
    [  2,   2, 0xeb], [  3,   3, 0x41], [  3,   3, 0xb7], [  3,   4, 0x2e],
    [  4,   4, 0x96], [  4,   5, 0x2d], [  5,   5, 0xa6], [  5,   6, 0x3f],
    [  6,   6, 0xd8], [  7,   7, 0x72], [  7,   8, 0x1c], [  8,   8, 0xc7],
    [  9,   9, 0x82], [  9,  10, 0x4e], [ 10,  11, 0x0a], [ 11,  11, 0xd6],
    [ 12,  12, 0xa4], [ 13,  13, 0x81], [ 14,  14, 0x70], [ 14,  15, 0x6e],
    [ 15,  16, 0x5e], [ 16,  17, 0x5d], [ 17,  18, 0x6e], [ 18,  19, 0x7e],
    [ 20,  20, 0x80], [ 21,  21, 0xb1], [ 22,  22, 0xd4], [ 23,  24, 0x07],
    [ 24,  25, 0x4a], [ 25,  26, 0x8e], [ 27,  27, 0xd3], [ 28,  29, 0x38],
    [ 29,  30, 0x8d], [ 31,  31, 0xf4], [ 32,  33, 0x6a], [ 34,  34, 0xd1],
    [ 35,  36, 0x59], [ 37,  37, 0xe2], [ 38,  39, 0x7b], [ 40,  41, 0x14],
    [ 41,  42, 0xbe], [ 43,  44, 0x69], [ 45,  46, 0x24], [ 47,  47, 0xe0],
    [ 48,  49, 0xac], [ 50,  51, 0x79], [ 52,  53, 0x56], [ 54,  55, 0x44],
    [ 56,  57, 0x23], [ 58,  59, 0x22], [ 60,  61, 0x22], [ 62,  63, 0x32],
    [ 64,  65, 0x43], [ 66,  67, 0x65], [ 68,  69, 0x87], [ 70,  71, 0xba],
    [ 72,  73, 0xfd], [ 75,  76, 0x31], [ 77,  78, 0x85], [ 79,  80, 0xda],
    [ 82,  83, 0x30], [ 84,  85, 0xa6], [ 86,  88, 0x1d], [ 89,  90, 0x95],
    [ 91,  93, 0x1d], [ 94,  95, 0xa5], [ 96,  98, 0x4f], [ 99, 100, 0xe9],
    [102, 103, 0x93], [104, 106, 0x4e], [107, 109, 0x0a], [110, 111, 0xd6],
    [113, 114, 0xa3], [116, 117, 0x81], [118, 120, 0x6f], [121, 123, 0x6e],
    [124, 126, 0x5d], [127, 129, 0x6d], [130, 132, 0x7e], [133, 135, 0x8f],
    [137, 138, 0xb1], [140, 141, 0xe4], [143, 145, 0x17], [146, 148, 0x5b],
    [150, 151, 0xa0], [153, 154, 0xf5], [156, 158, 0x6a], [160, 161, 0xc1],
    [163, 165, 0x48], [166, 168, 0xcf], [170, 172, 0x48], [174, 175, 0xd1],
    [177, 179, 0x7a], [181, 183, 0x24], [184, 186, 0xdf], [188, 190, 0x9b],
    [192, 194, 0x57], [196, 198, 0x24], [200, 202, 0x01], [203, 205, 0xff],
    [207, 209, 0xee], [211, 213, 0xde], [215, 217, 0xee], [219, 221, 0xfe],
    [224, 226, 0x10], [228, 230, 0x32], [232, 234, 0x64], [236, 238, 0xa8],
    [240, 242, 0xec], [245, 247, 0x30], [249, 251, 0x96], [253, 255, 0xfc],
];

pub const FRAME_DURATION: Duration = Duration::from_ticks(2 * Framebuffer::HEIGHT as u64);

pub(super) const ROWS: usize = 9;
pub(super) const PWM_MAX: u16 = (GAMMA_LUT[127][1] as u16) << 4 | (GAMMA_LUT[127][2] as u16) >> 4;
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

        let brightness = self.try_load(x, row).unwrap_or(0) as usize;
        let gamma_entry = &GAMMA_LUT[brightness / 2];
        let mut pwm = (gamma_entry[brightness % 2] as u16) << 4;
        pwm |= (gamma_entry[2] as u16) >> (4 * (brightness % 2)) & 0xf;

        cycles - pwm
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
