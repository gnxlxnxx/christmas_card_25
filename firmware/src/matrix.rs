use core::{any::Any, arch::asm, cell::{Cell, RefCell}, default, mem::{self, MaybeUninit}, ops::Sub, primitive::u16, sync::atomic::{AtomicBool, AtomicU8, Ordering}};

use ch32_hal::{self as hal, Peri, delay::Delay, gpio::{AnyPin, Pin}, interrupt::InterruptExt, pac::{self, gpio::vals::{Cnf, Mode}, timer::vals::{CcmrInputCcs, CcmrOutputCcs, FilterValue, Mms, Ocm, Urs}}, peripherals, time::Hertz, timer::{Channel, CoreInstance, low_level::{CountingMode, OutputCompareMode, Timer}}};
use critical_section::Mutex;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex}, channel::Channel as EmbChannel, signal::Signal};
use embassy_time_driver::{Driver, time_driver_impl};
use embassy_time_queue_utils::Queue;
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

const ROWS: usize = 9;

const HIST_LOW: u16 = 80;
const HIST_HIGH: u16 = 96;
const FILT_LEN: u8 = 6;

const FB_EXAMPLE: [[u8; Framebuffer::WIDTH]; Framebuffer::HEIGHT] = [
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

static mut MATRIX_DRIVER: MaybeUninit<MatrixDriver> = MaybeUninit::uninit();

static FB: Framebuffer = Framebuffer::from_u8_array(FB_EXAMPLE);
static BTN_STATE: ButtonArray<AtomicBool> = ButtonArray([
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
]);

static BTN_SAMPLE_SIGNAL: Signal<CriticalSectionRawMutex, ButtonSample> = Signal::new();
static BTN_EVENT_CHANNEL: EmbChannel<CriticalSectionRawMutex, (Button, bool), 3> = EmbChannel::new();

time_driver_impl!(static TIME_DRIVER: TimeDriver = TimeDriver {
    now: Mutex::new(Cell::new(0)),
    next: Mutex::new(Cell::new(0)),
    queue: Mutex::new(RefCell::new(Queue::new()))
});

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

    fn show_u8(&self, y: usize, mut val: u8) {
        for i in (0..8).rev() {
            self.store(i, y, if val & 1 != 0 { 32 } else { 0 });
            val >>= 1;
        }
    }

    fn show_u16(&self, y: usize, val: u16) {
        self.show_u8(y, (val>>8) as u8);
        self.show_u8(y + 1, val as u8);
    }
}

pub struct LedPins<'a> ([Peri<'a, AnyPin>; ROWS]);

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
#[repr(usize)]
pub enum Button {
    Start = 3,
    Select = 0,
    L = 2,
    R = 1
}

impl TryFrom<usize> for Button {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match(value) {
            0 => Ok(Self::Select),
            1 => Ok(Self::R),
            2 => Ok(Self::L),
            3 => Ok(Self::Start),
            _ => Err(())
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Default)]
struct ButtonArray<T>([T; 4]);

impl<T> ButtonArray<T> {
    const fn get(&self, btn: Button) -> &T {
        &self.0[btn as usize]
    }

    fn set(&mut self, btn: Button, val: T) {
        self.0[btn as usize] = val;
    }

    const fn each_ref(&self) -> ButtonArray<&T> {
        ButtonArray(self.0.each_ref())
    }

    fn map<U>(self, f: impl FnMut(T) -> U) -> ButtonArray<U> {
        ButtonArray(self.0.map(f))
    }
}

impl<T: Sub + Copy> Sub<T> for ButtonArray<T> {
    type Output = ButtonArray<T::Output>;

    fn sub(self, rhs: T) -> Self::Output {
        self.map(|l| l - rhs)
    }
}

#[derive(Clone, Copy, Default)]
struct ButtonSample {
    start_cnt: u16,
    btn_cnt: ButtonArray<u16>,
    intfr: pac::timer::regs::Intfr
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum AltGroup {
    PosIn,
    NegOut
}

struct MatrixDriver {
    led: LedPins<'static>,
    btn: ButtonPins<'static>,
    tim1: Timer<'static, peripherals::TIM1>,
    tim2: Timer<'static, peripherals::TIM2>,

    cycles: u32,

    start_cnt: u16,
    row: usize,
    next_group: AltGroup
}

impl MatrixDriver {
    fn get_val(&self, col: usize) -> u32 {
        let x = if col <= self.row { col } else { col - 1 };

        self.cycles - (GAMMA_LUT[FB.try_load(x, self.row).unwrap_or(0) as usize] as u32)
    }

    fn setup_next_group(&mut self) -> Option<(ButtonArray<u16>, pac::timer::regs::Intfr)> {
        match self.next_group {
            AltGroup::PosIn => {
                self.row = if self.row < ROWS - 1 { self.row + 1 } else { 0 };
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
                self.tim1.set_compare_value(Channel::Ch1, self.get_val(5));
                self.tim1.set_compare_value(Channel::Ch2, self.get_val(2));
                self.tim1.set_compare_value(Channel::Ch4, self.get_val(7));

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
                self.tim2.regs_gp16().ccer().write(|_| ());

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
                critical_section::with(|_| {
                    self.start_cnt = self.tim2.regs_basic().cnt().read();
                    self.btn.set_low_all();
                });


                None
            },

            AltGroup::NegOut => {
                self.next_group = AltGroup::PosIn;

                let intfr = self.tim2.regs_gp16().intfr().read();
                let btn_cnt = ButtonArray([
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
                self.tim2.regs_gp16().ccer().write(|_| ());

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
                self.tim1.set_compare_value(Channel::Ch1, self.get_val(0));
                self.tim1.set_compare_value(Channel::Ch2, self.get_val(1));
                self.tim2.set_compare_value(Channel::Ch1, self.get_val(8));
                self.tim2.set_compare_value(Channel::Ch2, self.get_val(6));
                self.tim2.set_compare_value(Channel::Ch3, self.get_val(3));
                self.tim2.set_compare_value(Channel::Ch4, self.get_val(4));

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

                Some((btn_cnt, intfr))
            }
        }
    }

    fn advance(&mut self) {
        self.led.set_float_all();
        let result = self.setup_next_group();
        self.led.set_high(self.row);

        if let Some((btn_cnt, intfr)) = result {
            BTN_SAMPLE_SIGNAL.signal(ButtonSample { start_cnt: self.start_cnt, btn_cnt, intfr });
        }
    }
}

#[interrupt]
fn TIM1_UP() {
    #[allow(static_mut_refs)]
    let matrix_driver = unsafe { MATRIX_DRIVER.assume_init_mut() };

    if matrix_driver.tim1.clear_update_interrupt() {
        matrix_driver.advance();

        TIME_DRIVER.advance();
    }
}

#[derive(Debug)]
struct TimeDriver {
    now: Mutex<Cell<u64>>,
    next: Mutex<Cell<u64>>,
    queue: critical_section::Mutex<RefCell<Queue>>
}

impl TimeDriver {
    fn advance(&self) {
        critical_section::with(|cs| {
            let now_box = self.now.borrow(cs);
            let next_box = self.next.borrow(cs);

            let now = now_box.get() + 1;
            now_box.set(now);

            if next_box.get() <= now {
                next_box.set(self.queue.borrow_ref_mut(cs).next_expiration(now));
            }
        });
    }
}

impl Driver for TimeDriver {
    fn now(&self) -> u64 {
        critical_section::with(|cs| self.now.borrow(cs).get() )
    }

    fn schedule_wake(&self, at: u64, waker: &core::task::Waker) {
        critical_section::with(|cs| {
            let mut queue = self.queue.borrow_ref_mut(cs);
            if queue.schedule_wake(at, waker) {
                self.next.borrow(cs).set(queue.next_expiration(self.now()));
            }
        });
    }
}

pub struct Matrix {
    _private: ()
}

impl Matrix {
    pub fn init(
        spawner: Spawner,

        led: LedPins<'static>,
        btn: ButtonPins<'static>,

        tim1: Peri<'static, peripherals::TIM1>,
        tim2: Peri<'static, peripherals::TIM2>
    ) -> Self {
        let tim1 = Timer::new(tim1);
        let tim2 = Timer::new(tim2);

        tim1.set_frequency(Hertz::khz(10));
        tim2.set_frequency(Hertz::khz(10));
        let cycles = tim1.get_max_compare_value() + 1;
        assert_eq!(cycles, tim2.get_max_compare_value() + 1);
        assert!(cycles >= GAMMA_LUT[255] as u32);

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

        // Configure tim2 as slave of tim1 (tim1 enable also controls tim2)
        tim1.regs_gp16().ctlr2().modify(|w| w.set_mms(Mms::ENABLE));
        tim2.regs_gp16().smcfgr().modify(|w| w.set_sms(0b101));
        tim2.start();

        tim1.start();

        unsafe {
            #[allow(static_mut_refs)]
            MATRIX_DRIVER.write(MatrixDriver {
                led,
                btn,
                tim1,
                tim2,
                cycles,
                start_cnt: 0,
                row: 8,
                next_group: AltGroup::PosIn
            });

            hal::interrupt::TIM1_UP.enable();
        }

        spawner.spawn(process_btn_samples()).unwrap();

        Self { _private: () }
    }

    pub fn fb(&self) -> &Framebuffer {
        &FB
    }

    pub fn btn(&self, btn: Button) -> bool {
        BTN_STATE.get(btn).load(Ordering::Relaxed)
    }

    pub async fn btn_event(&self) -> (Button, bool) {
        BTN_EVENT_CHANNEL.receive().await
    }

    pub async fn btn_event_filtered(&self, btn: Option<Button>, pressed: Option<bool>)
        -> (Button, bool)
    {
        loop {
            let event = self.btn_event().await;
            if btn.map_or(true, |b| b == event.0) && pressed.map_or(true, |p| p == event.1) {
                return event;
            }
        }
    }
}

#[embassy_executor::task]
async fn process_btn_samples() {
    let mut fcount = ButtonArray::<u8>::default();

    loop {
        let sample = BTN_SAMPLE_SIGNAL.wait().await;
        let state = BTN_STATE.each_ref().map(|s| s.load(Ordering::Relaxed));

        for (ch, (((prev_state, next_state), cnt), fcount)) in state.0.iter()
            .zip(BTN_STATE.0.iter())
            .zip(sample.btn_cnt.0.iter())
            .zip(fcount.0.iter_mut())
            .enumerate()
        {
            let lvl = if sample.intfr.ccif(ch) { cnt - sample.start_cnt } else { u16::MAX };

            *fcount = if prev_state ^ (lvl >= if *prev_state { HIST_LOW } else { HIST_HIGH }) {
                if *fcount < FILT_LEN {
                    *fcount + 1
                } else {
                    next_state.store(!prev_state, Ordering::Relaxed);
                    BTN_EVENT_CHANNEL.send((ch.try_into().unwrap(), !prev_state)).await;
                    0
                }
            } else { 0 }
        }
    }
}
