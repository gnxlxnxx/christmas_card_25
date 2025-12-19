use core::{any::Any, arch::asm, mem, sync::atomic::{AtomicU8, Ordering}, u16};

use ch32_hal::{self as hal, Peri, delay::Delay, gpio::{AnyPin, Pin}, interrupt::InterruptExt, pac::{self, gpio::vals::{Cnf, Mode}, timer::vals::{CcmrInputCcs, CcmrOutputCcs, FilterValue, Mms, Ocm, Urs}}, peripherals, time::Hertz, timer::{Channel, CoreInstance, low_level::{CountingMode, OutputCompareMode, Timer}}};
use crate::hal::interrupt;

// Gamma brightness lookup table <https://victornpb.github.io/gamma-table-generator>
// gamma = 2.20 steps = 256 range = 0-4095
const GAMMA_LUT: [u16; 256] = [
       0,   0,   0,   0,   0,   1,   1,   2,   2,   3,   3,   4,   5,   6,   7,   8,
       9,  11,  12,  14,  15,  17,  19,  21,  23,  25,  27,  29,  32,  34,  37,  40,
      43,  46,  49,  52,  55,  59,  62,  66,  70,  73,  77,  82,  86,  90,  95,  99,
     104, 109, 114, 119, 124, 129, 135, 140, 146, 152, 158, 164, 170, 176, 182, 189,
     196, 202, 209, 216, 224, 231, 238, 246, 254, 261, 269, 277, 286, 294, 302, 311,
     320, 328, 337, 347, 356, 365, 375, 384, 394, 404, 414, 424, 435, 445, 456, 467,
     477, 488, 500, 511, 522, 534, 545, 557, 569, 581, 594, 606, 619, 631, 644, 657,
     670, 683, 697, 710, 724, 738, 752, 766, 780, 794, 809, 823, 838, 853, 868, 884,
     899, 914, 930, 946, 962, 978, 994,1011,1027,1044,1061,1078,1095,1112,1130,1147,
    1165,1183,1201,1219,1237,1256,1274,1293,1312,1331,1350,1370,1389,1409,1429,1449,
    1469,1489,1509,1530,1551,1572,1593,1614,1635,1657,1678,1700,1722,1744,1766,1789,
    1811,1834,1857,1880,1903,1926,1950,1974,1997,2021,2045,2070,2094,2119,2143,2168,
    2193,2219,2244,2270,2295,2321,2347,2373,2400,2426,2453,2479,2506,2534,2561,2588,
    2616,2644,2671,2700,2728,2756,2785,2813,2842,2871,2900,2930,2959,2989,3019,3049,
    3079,3109,3140,3170,3201,3232,3263,3295,3326,3358,3390,3421,3454,3486,3518,3551,
    3584,3617,3650,3683,3716,3750,3784,3818,3852,3886,3920,3955,3990,4025,4060,4095,
];

const FB_EXAMPLE: [[u8; 8]; 9] = [
    [32, 0, 0, 0, 0, 0, 0, 0],
    [0, 32, 0, 0, 0, 0, 0, 0],
    [0, 0, 32, 0, 0, 0, 0, 0],
    [0, 0, 0, 32, 0, 0, 0, 0],
    [0, 0, 0, 0, 32, 0, 0, 0],
    [0, 0, 0, 0, 0, 32, 0, 0],
    [0, 0, 0, 0, 0, 0, 32, 0],
    [0, 0, 0, 0, 0, 0, 0, 32],
    [0, 0, 0, 0, 64, 0, 0, 0],
];

static MATRIX_FB: MatrixFb = MatrixFb::from_u8_array(FB_EXAMPLE);
static mut MATRIX_INT: Option<MatrixInterrupt> = None;

#[derive(Debug)]
pub struct MatrixFb(pub [[AtomicU8; Self::WIDTH]; Self::HEIGHT]);

impl MatrixFb {
    const WIDTH: usize = 8;
    const HEIGHT: usize = 9;

    pub const fn new() -> Self {
        Self::new_filled(0)
    }

    pub const fn new_filled(val: u8) -> Self {
        Self::from_u8_array([[val; Self::WIDTH]; Self::HEIGHT])
    }

    pub const fn from_u8_array(arr: [[u8; Self::WIDTH]; Self::HEIGHT]) -> Self {
        Self(unsafe { mem::transmute(arr) })
    }

    pub fn load(&self, x: usize, y: usize) -> u8 {
        self.0[y][x].load(Ordering::Relaxed)
    }

    pub fn store(&self, x: usize, y: usize, val: u8) {
        self.0[y][x].store(val, Ordering::Relaxed)
    }
}

pub struct LedPins<'a> ([Peri<'a, AnyPin>; 9]);

impl<'a> LedPins<'a> {
    pub fn new(
        led1: Peri<'a, peripherals::PD0>,
        led2: Peri<'a, peripherals::PA2>,
        led3: Peri<'a, peripherals::PA1>,
        led4: Peri<'a, peripherals::PD6>,
        led5: Peri<'a, peripherals::PD5>,
        led6: Peri<'a, peripherals::PD2>,
        led7: Peri<'a, peripherals::PC7>,
        led8: Peri<'a, peripherals::PC4>,
        led9: Peri<'a, peripherals::PC1>
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

        Self ([
            led1.into(),
            led2.into(),
            led3.into(),
            led4.into(),
            led5.into(),
            led6.into(),
            led7.into(),
            led8.into(),
            led9.into()
        ])
    }

    fn set_float_all(&mut self) {
        pac::GPIOA.cfglr().modify(|w| {
            w.set_mode(1, Mode::INPUT);
            w.set_cnf(1, Cnf::FLOATING_IN__OPEN_DRAIN_OUT);
            w.set_mode(2, Mode::INPUT);
            w.set_cnf(2, Cnf::FLOATING_IN__OPEN_DRAIN_OUT);
        });
        pac::GPIOC.cfglr().modify(|w| {
            w.set_mode(1, Mode::INPUT);
            w.set_cnf(1, Cnf::FLOATING_IN__OPEN_DRAIN_OUT);
            w.set_mode(4, Mode::INPUT);
            w.set_cnf(4, Cnf::FLOATING_IN__OPEN_DRAIN_OUT);
            w.set_mode(7, Mode::INPUT);
            w.set_cnf(7, Cnf::FLOATING_IN__OPEN_DRAIN_OUT);
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

    fn set_high(&mut self, row: usize) {
        let pin = &self.0[row];
        pac::GPIO(pin.port().into()).cfglr().modify(|w| {
            w.set_mode(pin.pin().into(), Mode::OUTPUT_50MHZ);
            w.set_cnf(pin.pin().into(), Cnf::ANALOG_IN__PUSH_PULL_OUT);
        });
    }
}

#[derive(Debug)]
pub struct ButtonPins<'a> {
    start: Peri<'a, peripherals::PD7>,
    select: Peri<'a, peripherals::PD4>,
    l: Peri<'a, peripherals::PC0>,
    r: Peri<'a, peripherals::PD3>
}

impl<'a> ButtonPins<'a> {
    pub fn new(
        start: Peri<'a, peripherals::PD7>,
        select: Peri<'a, peripherals::PD4>,
        l: Peri<'a, peripherals::PC0>,
        r: Peri<'a, peripherals::PD3>
    ) -> Self {
        Self { start, select, l, r }
    }

    fn set_high_all(&mut self) {
        pac::GPIOC.bshr().write(|w| {
            w.set_bs(0, true);
        });
        pac::GPIOD.bshr().write(|w| {
            w.set_bs(3, true);
            w.set_bs(4, true);
            w.set_bs(7, true);
        });
    }

    fn set_low_all(&mut self) {
        pac::GPIOC.bshr().write(|w| {
            w.set_br(0, true);
        });
        pac::GPIOD.bshr().write(|w| {
            w.set_br(3, true);
            w.set_br(4, true);
            w.set_br(7, true);
        });
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum AltGroup {
    PosIn,
    NegOut
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Default)]
struct ButtonArray<T>([T; 4]);

impl<T> ButtonArray<T> {
    fn start(&self) -> &T {
        &self.0[3]
    }

    fn select(&self) -> &T {
        &self.0[0]
    }

    fn l(&self) -> &T {
        &self.0[2]
    }

    fn r(&self) -> &T {
        &self.0[1]
    }

    fn set_start(&mut self, val: T) {
        self.0[3] = val;
    }

    fn set_select(&mut self, val: T) {
        self.0[0] = val;
    }

    fn set_l(&mut self, val: T) {
        self.0[2] = val;
    }

    fn set_r(&mut self, val: T) {
        self.0[1] = val;
    }
}

#[derive(Eq, PartialEq, Clone, Copy)]
enum ButtonMeasurement {
    StartTime(u16),
    Measurements(ButtonArray<u16>, pac::timer::regs::Intfr)
}

struct MatrixInterrupt {
    led: LedPins<'static>,
    btn: ButtonPins<'static>,
    tim1: Timer<'static, peripherals::TIM1>,
    tim2: Timer<'static, peripherals::TIM2>,

    cycles: u32,

    row: usize,
    next_group: AltGroup
}

impl MatrixInterrupt {
    fn get_val(&self, row: usize, col: usize) -> u32 {
        let x = if col <= row { col } else { col - 1 };
        let y = row;

        self.cycles - (GAMMA_LUT[MATRIX_FB.load(x, y) as usize] as u32)
    }

    fn advance_group(&mut self) -> ButtonMeasurement {
        match self.next_group {
            AltGroup::PosIn => {
                self.row = (self.row + 1) % 9;
                self.next_group = AltGroup::NegOut;

                // TIM1: Enable outputs
                self.tim1.regs_gp16().ccer().write(|w| {
                    w.set_cce(0, true);
                    w.set_ccp(0, true);
                    w.set_cce(1, true);
                    w.set_ccp(1, true);
                    w.set_cce(3, true);
                    w.set_ccp(3, true);
                });

                // Set pwm values
                self.tim1.set_compare_value(Channel::Ch1, self.get_val(self.row, 5).into());
                self.tim1.set_compare_value(Channel::Ch2, self.get_val(self.row, 2).into());
                self.tim1.set_compare_value(Channel::Ch4, self.get_val(self.row, 7).into());

                // TIM1: Attach positive led pins
                // TIM2: Attach button pins as inputs with pull resistor
                pac::GPIOA.cfglr().modify(|w| {
                    w.set_mode(1, Mode::OUTPUT_50MHZ);
                    w.set_cnf(1, Cnf::AF_OPEN_DRAIN_OUT);
                });
                pac::GPIOC.cfglr().modify(|w| {
                    w.set_mode(4, Mode::OUTPUT_50MHZ);
                    w.set_cnf(4, Cnf::AF_OPEN_DRAIN_OUT);

                    w.set_mode(0, Mode::INPUT);
                    w.set_cnf(0, Cnf::PULL_IN__AF_PUSH_PULL_OUT);
                });
                pac::GPIOD.cfglr().modify(|w| {
                    w.set_mode(2, Mode::OUTPUT_50MHZ);
                    w.set_cnf(2, Cnf::AF_OPEN_DRAIN_OUT);

                    w.set_mode(3, Mode::INPUT);
                    w.set_cnf(3, Cnf::PULL_IN__AF_PUSH_PULL_OUT);
                    w.set_mode(4, Mode::INPUT);
                    w.set_cnf(4, Cnf::PULL_IN__AF_PUSH_PULL_OUT);
                    w.set_mode(7, Mode::INPUT);
                    w.set_cnf(7, Cnf::PULL_IN__AF_PUSH_PULL_OUT);
                });

                // TIM2: Disable all channels to allow changing ccs bits
                self.tim2.regs_gp16().ccer().write(|w| ());

                // TIM2: Select alternate mapping with button pins
                pac::AFIO.pcfr1().modify(|w| w.set_tim2_rm(0));

                // TIM2: Configure input capturing filter and input
                self.tim2.regs_gp16().chctlr_input(0).write(|w| {
                    w.set_ccs(0, CcmrInputCcs::TI4);
                    w.set_icf(0, FilterValue::FCK_INT_N4);
                    w.set_ccs(1, CcmrInputCcs::TI4);
                    w.set_icf(1, FilterValue::FCK_INT_N4);
                });
                self.tim2.regs_gp16().chctlr_input(1).write(|w| {
                    w.set_ccs(0, CcmrInputCcs::TI4);
                    w.set_icf(0, FilterValue::FCK_INT_N4);
                    w.set_ccs(1, CcmrInputCcs::TI4);
                    w.set_icf(1, FilterValue::FCK_INT_N4);
                });

                // TIM2: Clear capture flags
                self.tim2.regs_gp16().intfr().modify(|w| {
                    w.set_ccif(0, false);
                    w.set_ccif(1, false);
                    w.set_ccif(2, false);
                    w.set_ccif(3, false);
                });

                // TIM2: Enable falling edge capture inputs
                self.tim2.regs_gp16().ccer().write(|w| {
                    w.set_cce(0, true);
                    w.set_ccp(0, true);
                    w.set_cce(1, true);
                    w.set_ccp(1, true);
                    w.set_cce(2, true);
                    w.set_ccp(2, true);
                    w.set_cce(3, true);
                    w.set_ccp(3, true);
                });

                // TIM2: Set pulldown on button pins
                self.btn.set_low_all();


                ButtonMeasurement::StartTime(self.tim2.regs_basic().cnt().read())
            },

            AltGroup::NegOut => {
                self.next_group = AltGroup::PosIn;

                let intfr = self.tim2.regs_gp16().intfr().read();
                let measurement = ButtonArray([
                    self.tim2.get_capture_value(Channel::Ch1) as u16,
                    self.tim2.get_capture_value(Channel::Ch2) as u16,
                    self.tim2.get_capture_value(Channel::Ch3) as u16,
                    self.tim2.get_capture_value(Channel::Ch4) as u16
                ]);

                // TIM1: Enable outputs
                self.tim1.regs_gp16().ccer().write(|w| {
                    w.set_ccne(0, true);
                    w.set_ccnp(0, true);
                    w.set_ccne(1, true);
                    w.set_ccnp(1, true);
                });

                // TIM2: Disable all channels to allow changing ccs bits
                self.tim2.regs_gp16().ccer().write(|w| ());

                // TIM2: Select alternate mapping with leds
                pac::AFIO.pcfr1().modify(|w| w.set_tim2_rm(3));

                // TIM2: Configure output compare mode
                self.tim2.regs_gp16().chctlr_output(0).write(|w| {
                    w.set_ccs(0, CcmrOutputCcs::OUTPUT);
                    w.set_ocm(0, Ocm::PWMMODE2);
                    w.set_ccs(1, CcmrOutputCcs::OUTPUT);
                    w.set_ocm(1, Ocm::PWMMODE2);
                });
                self.tim2.regs_gp16().chctlr_output(1).write(|w| {
                    w.set_ccs(0, CcmrOutputCcs::OUTPUT);
                    w.set_ocm(0, Ocm::PWMMODE2);
                    w.set_ccs(1, CcmrOutputCcs::OUTPUT);
                    w.set_ocm(1, Ocm::PWMMODE2);
                });

                // TIM2: Enable outputs
                self.tim2.regs_gp16().ccer().write(|w| {
                    w.set_cce(0, true);
                    w.set_ccp(0, true);
                    w.set_cce(1, true);
                    w.set_ccp(1, true);
                    w.set_cce(2, true);
                    w.set_ccp(2, true);
                    w.set_cce(3, true);
                    w.set_ccp(3, true);
                });

                // Set pwm values
                self.tim1.set_compare_value(Channel::Ch1, self.get_val(self.row, 0).into());
                self.tim1.set_compare_value(Channel::Ch2, self.get_val(self.row, 1).into());
                if self.row < 8 {
                    self.tim2.set_compare_value(Channel::Ch1, self.get_val(self.row, 8).into());
                }
                self.tim2.set_compare_value(Channel::Ch2, self.get_val(self.row, 6).into());
                self.tim2.set_compare_value(Channel::Ch3, self.get_val(self.row, 3).into());
                self.tim2.set_compare_value(Channel::Ch4, self.get_val(self.row, 4).into());

                self.btn.set_high_all();

                // TIM1: Attach negative led pins
                // TIM2: Attach led pins
                // Set button pins as outputs
                pac::GPIOA.cfglr().modify(|w| {
                    w.set_mode(2, Mode::OUTPUT_50MHZ);
                    w.set_cnf(2, Cnf::AF_OPEN_DRAIN_OUT);
                });
                pac::GPIOC.cfglr().modify(|w| {
                    w.set_mode(1, Mode::OUTPUT_50MHZ);
                    w.set_cnf(1, Cnf::AF_OPEN_DRAIN_OUT);
                    w.set_mode(7, Mode::OUTPUT_50MHZ);
                    w.set_cnf(7, Cnf::AF_OPEN_DRAIN_OUT);

                    w.set_mode(0, Mode::OUTPUT_50MHZ);
                    w.set_cnf(0, Cnf::ANALOG_IN__PUSH_PULL_OUT);
                });
                pac::GPIOD.cfglr().modify(|w| {
                    w.set_mode(0, Mode::OUTPUT_50MHZ);
                    w.set_cnf(0, Cnf::AF_OPEN_DRAIN_OUT);
                    w.set_mode(5, Mode::OUTPUT_50MHZ);
                    w.set_cnf(5, Cnf::AF_OPEN_DRAIN_OUT);
                    w.set_mode(6,  Mode::OUTPUT_50MHZ);
                    w.set_cnf(6,  Cnf::AF_OPEN_DRAIN_OUT);

                    w.set_mode(3, Mode::OUTPUT_50MHZ);
                    w.set_cnf(3, Cnf::ANALOG_IN__PUSH_PULL_OUT);
                    w.set_mode(4, Mode::OUTPUT_50MHZ);
                    w.set_cnf(4, Cnf::ANALOG_IN__PUSH_PULL_OUT);
                    w.set_mode(7, Mode::OUTPUT_50MHZ);
                    w.set_cnf(7, Cnf::ANALOG_IN__PUSH_PULL_OUT);
                });

                ButtonMeasurement::Measurements(measurement, intfr)
            }
        }
    }

    fn advance(&mut self) {
        self.led.set_float_all();

        let measurement = self.advance_group();

        self.led.set_high(self.row);

        if let ButtonMeasurement::Measurements(measurement, intfr) = measurement {
        // if let ButtonMeasurement::StartTime(time) = measurement {
        //     let mut val = time;

            for (i, val) in measurement.0.iter().enumerate() {
                let mut val = *val;
                for j in 0..16 {
                    MATRIX_FB.store(j%8, 2*i + j/8, if val&1 != 0 { 32 } else { 0 });
                    val >>= 1;
                }
            }

            for i in 0..4 {
                MATRIX_FB.store(i, 8, if intfr.ccif(i) { 32 } else { 0 });
            }
        }
    }
}

#[interrupt]
fn TIM1_UP() {
    let matrix_int = unsafe {
        (&mut *&raw mut MATRIX_INT).as_mut().unwrap_unchecked()
    };

    if matrix_int.tim1.clear_update_interrupt() {
        matrix_int.advance();
    }
}

pub struct Matrix {
    fb: &'static MatrixFb
}

impl Matrix {
    pub fn new(
        led: LedPins<'static>,
        btn: ButtonPins<'static>,

        tim1: Peri<'static, peripherals::TIM1>,
        tim2: Peri<'static, peripherals::TIM2>
    ) -> Self {
        let tim1 = Timer::new(tim1);
        let tim2 = Timer::new(tim2);

        tim1.set_frequency(Hertz::khz(10));
        tim2.set_frequency(Hertz::khz(10));
        assert_eq!(tim1.get_max_compare_value(), tim2.get_max_compare_value());

        tim1.set_autoreload_preload(true);
        tim2.set_autoreload_preload(true);

        tim1.set_counting_mode(CountingMode::EdgeAlignedUp);
        tim2.set_counting_mode(CountingMode::EdgeAlignedUp);

        tim1.set_output_compare_mode(Channel::Ch1, OutputCompareMode::PwmMode2);
        tim1.set_output_compare_mode(Channel::Ch2, OutputCompareMode::PwmMode2);
        tim1.set_output_compare_mode(Channel::Ch4, OutputCompareMode::PwmMode2);
        tim1.set_moe(true);

        // Trigger TIM1_UP interrupt on timer overflow
        tim1.regs_gp16().ctlr1().modify(|w| w.set_urs(Urs::COUNTERONLY));
        tim1.enable_update_interrupt(true);

        // Set capture register to -1 on counter overflow
        // tim2.regs_gp16().ctlr1().modify(|w| w.set_capov(true));

        // Configure tim2 as slave of tim1 (tim1 enable also controls tim2)
        tim1.regs_gp16().ctlr2().modify(|w| w.set_mms(Mms::ENABLE));
        tim2.regs_gp16().smcfgr().modify(|w| w.set_sms(0b101));
        tim2.start();

        tim1.start();

        let cycles = tim1.get_max_compare_value() + 1;
        assert!(cycles >= GAMMA_LUT[255] as u32);

        unsafe {
            MATRIX_INT = Some(MatrixInterrupt {
                led,
                btn,
                tim1,
                tim2,
                cycles,
                row: 8,
                next_group: AltGroup::PosIn
            });

            hal::interrupt::TIM1_UP.enable();
        }

        Self { fb: &MATRIX_FB }
    }

    pub fn fb(&self) -> &MatrixFb {
        self.fb
    }
}
