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
// gamma = 2.20 steps = 256 range = 0-4095
const GAMMA_LUT: [u16; 256] = [
       0,    0,    0,    0,    0,    1,    1,    2,    2,    3,    3,    4,    5,    6,    7,    8,
       9,   11,   12,   14,   15,   17,   19,   21,   23,   25,   27,   29,   32,   34,   37,   40,
      43,   46,   49,   52,   55,   59,   62,   66,   70,   73,   77,   82,   86,   90,   95,   99,
     104,  109,  114,  119,  124,  129,  135,  140,  146,  152,  158,  164,  170,  176,  182,  189,
     196,  202,  209,  216,  224,  231,  238,  246,  254,  261,  269,  277,  286,  294,  302,  311,
     320,  328,  337,  347,  356,  365,  375,  384,  394,  404,  414,  424,  435,  445,  456,  467,
     477,  488,  500,  511,  522,  534,  545,  557,  569,  581,  594,  606,  619,  631,  644,  657,
     670,  683,  697,  710,  724,  738,  752,  766,  780,  794,  809,  823,  838,  853,  868,  884,
     899,  914,  930,  946,  962,  978,  994, 1011, 1027, 1044, 1061, 1078, 1095, 1112, 1130, 1147,
    1165, 1183, 1201, 1219, 1237, 1256, 1274, 1293, 1312, 1331, 1350, 1370, 1389, 1409, 1429, 1449,
    1469, 1489, 1509, 1530, 1551, 1572, 1593, 1614, 1635, 1657, 1678, 1700, 1722, 1744, 1766, 1789,
    1811, 1834, 1857, 1880, 1903, 1926, 1950, 1974, 1997, 2021, 2045, 2070, 2094, 2119, 2143, 2168,
    2193, 2219, 2244, 2270, 2295, 2321, 2347, 2373, 2400, 2426, 2453, 2479, 2506, 2534, 2561, 2588,
    2616, 2644, 2671, 2700, 2728, 2756, 2785, 2813, 2842, 2871, 2900, 2930, 2959, 2989, 3019, 3049,
    3079, 3109, 3140, 3170, 3201, 3232, 3263, 3295, 3326, 3358, 3390, 3421, 3454, 3486, 3518, 3551,
    3584, 3617, 3650, 3683, 3716, 3750, 3784, 3818, 3852, 3886, 3920, 3955, 3990, 4025, 4060, 4095,
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

        cycles - GAMMA_LUT[self.try_load(x, row).unwrap_or(0) as usize]
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
