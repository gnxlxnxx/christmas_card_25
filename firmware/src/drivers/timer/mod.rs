pub mod buttons;
pub mod matrix;
mod time;

use crate::drivers::timer::matrix::Matrix;
use ch32_metapac::{
    self as pac, Interrupt,
    gpio::vals::{Cnf, Mode},
    timer::vals::{CcmrInputCcs, CcmrOutputCcs, Cms, Dir, FilterValue, Mms, Ocm, Urs},
};
use qingke::pfic;
use qingke_rt::interrupt;

const CYCLES: u16 = 4800;

const _: () = assert!(CYCLES >= matrix::PWM_MAX);

static mut TIMER_DRIVER: TimerDriver = TimerDriver {
    start_cnt: 0,
    fcount: buttons::Group([0, 0, 0, 0]),

    row: 0,
    next_group: AltGroup::PosIn,
};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum AltGroup {
    PosIn,
    NegOut,
}

//
// Timer usage
//
// ## TIM1
// | MAP  |  CH1  |  CH1N |  CH2  |  CH2N |  CH3  |  CH3N |  CH4  |   USE  |
// |------|-------|-------|-------|-------|-------|-------|-------|--------|
// |**00**|**PD2**|**PD0**|**PA1**|**PA2**|  PC3  |  PD1  |**PC4**|**LEDs**|
// |  01  |  PC6  |  PC3  |  PC7  |  PC4  |  PC0  |  PD1  |  PD3  |        |
// |  10  |  PD2  |  PD0  |  PA1  |  PA2  |  PC3  |  PD1  |  PC4  |        |
// |  11  |  PC4  |  PC3  |  PC7  |  PD2  |  PC5  |  PC6  |  PD4  |        |
//
// ## TIM2
// | MAP  |  CH1  |  CH2  |  CH3  |  CH4  |    USE    |
// |------|-------|-------|-------|-------|-----------|
// |**00**|**PD4**|**PD3**|**PC0**|**PD7**|**Buttons**|
// |  01  |  PD5  |  PC2  |  PD2  |  PC1  |           |
// |  10  |  PC1  |  PD3  |  PC0  |  PD7  |           |
// |**11**|**PC1**|**PC7**|**PD6**|**PD5**|  **LEDs** |
//

struct TimerDriver {
    start_cnt: u16,
    fcount: buttons::Group<u8>,

    row: usize,
    next_group: AltGroup,
}

impl TimerDriver {
    fn get_pwm(&self, col: usize) -> u16 {
        Matrix::fb().get_pwm(col, self.row, CYCLES)
    }

    fn setup_next_group(&mut self) -> Option<(buttons::Group<u16>, pac::timer::regs::Intfr)> {
        match self.next_group {
            AltGroup::PosIn => {
                self.row = if self.row < matrix::ROWS - 1 {
                    self.row + 1
                } else {
                    0
                };
                self.next_group = AltGroup::NegOut;

                // TIM1: Enable outputs
                pac::TIM1.ccer().write(|w| {
                    w.set_cce(0, true);
                    w.set_ccp(0, true);
                    w.set_cce(1, true);
                    w.set_ccp(1, true);
                    w.set_cce(3, true);
                    w.set_ccp(3, true);
                });

                // Set pwm values
                pac::TIM1.chcvr(0).write_value(self.get_pwm(5));
                pac::TIM1.chcvr(1).write_value(self.get_pwm(2));
                pac::TIM1.chcvr(3).write_value(self.get_pwm(7));

                // TIM1: Attach positive led pins
                // TIM2: Attach button pins as inputs with pull resistor
                pac::GPIOA.cfglr().modify(|w| {
                    w.set_mode(1, Mode::OUTPUT_50MHZ);
                    w.set_cnf(1, Cnf::AF_OPEN_DRAIN_OUT);
                });
                critical_section::with(|_| {
                    pac::GPIOC.cfglr().modify(|w| {
                        w.set_mode(4, Mode::OUTPUT_50MHZ);
                        w.set_cnf(4, Cnf::AF_OPEN_DRAIN_OUT);

                        w.set_mode(0, Mode::INPUT);
                        w.set_cnf(0, Cnf::PULL_IN__AF_PUSH_PULL_OUT);
                    })
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
                pac::TIM2.ccer().write(|_| ());

                // TIM2: Select alternate mapping with button pins
                pac::AFIO.pcfr1().modify(|w| w.set_tim2_rm(0));

                // TIM2: Configure input capturing filter and input
                pac::TIM2.chctlr_input(0).write(|w| {
                    w.set_ccs(0, CcmrInputCcs::TI4);
                    w.set_icf(0, FilterValue::FCK_INT_N4);
                    w.set_ccs(1, CcmrInputCcs::TI4);
                    w.set_icf(1, FilterValue::FCK_INT_N4);
                });
                pac::TIM2.chctlr_input(1).write(|w| {
                    w.set_ccs(0, CcmrInputCcs::TI4);
                    w.set_icf(0, FilterValue::FCK_INT_N4);
                    w.set_ccs(1, CcmrInputCcs::TI4);
                    w.set_icf(1, FilterValue::FCK_INT_N4);
                });

                // TIM2: Clear capture flags
                pac::TIM2.intfr().modify(|w| {
                    w.set_ccif(0, false);
                    w.set_ccif(1, false);
                    w.set_ccif(2, false);
                    w.set_ccif(3, false);
                });

                // TIM2: Enable falling edge capture inputs
                pac::TIM2.ccer().write(|w| {
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
                    self.start_cnt = pac::TIM2.cnt().read();
                    buttons::Pins::set_low_all();
                });

                None
            }

            AltGroup::NegOut => {
                self.next_group = AltGroup::PosIn;

                let intfr = pac::TIM2.intfr().read();
                let btn_cnt = buttons::Group([
                    pac::TIM2.chcvr(0).read(),
                    pac::TIM2.chcvr(1).read(),
                    pac::TIM2.chcvr(2).read(),
                    pac::TIM2.chcvr(3).read(),
                ]);

                // TIM1: Enable outputs
                pac::TIM1.ccer().write(|w| {
                    w.set_ccne(0, true);
                    w.set_ccnp(0, true);
                    w.set_ccne(1, true);
                    w.set_ccnp(1, true);
                });

                // TIM2: Disable all channels to allow changing ccs bits
                pac::TIM2.ccer().write(|_| ());

                // TIM2: Select alternate mapping with leds
                pac::AFIO.pcfr1().modify(|w| w.set_tim2_rm(3));

                // TIM2: Configure output compare mode
                pac::TIM2.chctlr_output(0).write(|w| {
                    w.set_ccs(0, CcmrOutputCcs::OUTPUT);
                    w.set_ocm(0, Ocm::PWMMODE2);
                    w.set_ccs(1, CcmrOutputCcs::OUTPUT);
                    w.set_ocm(1, Ocm::PWMMODE2);
                });
                pac::TIM2.chctlr_output(1).write(|w| {
                    w.set_ccs(0, CcmrOutputCcs::OUTPUT);
                    w.set_ocm(0, Ocm::PWMMODE2);
                    w.set_ccs(1, CcmrOutputCcs::OUTPUT);
                    w.set_ocm(1, Ocm::PWMMODE2);
                });

                // TIM2: Enable outputs
                pac::TIM2.ccer().write(|w| {
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
                pac::TIM1.chcvr(0).write_value(self.get_pwm(0));
                pac::TIM1.chcvr(1).write_value(self.get_pwm(1));
                pac::TIM2.chcvr(0).write_value(self.get_pwm(8));
                pac::TIM2.chcvr(1).write_value(self.get_pwm(6));
                pac::TIM2.chcvr(2).write_value(self.get_pwm(3));
                pac::TIM2.chcvr(3).write_value(self.get_pwm(4));

                buttons::Pins::set_high_all();

                // TIM1: Attach negative led pins
                // TIM2: Attach led pins
                // Set button pins as outputs
                pac::GPIOA.cfglr().modify(|w| {
                    w.set_mode(2, Mode::OUTPUT_50MHZ);
                    w.set_cnf(2, Cnf::AF_OPEN_DRAIN_OUT);
                });
                critical_section::with(|_| {
                    pac::GPIOC.cfglr().modify(|w| {
                        w.set_mode(1, Mode::OUTPUT_50MHZ);
                        w.set_cnf(1, Cnf::AF_OPEN_DRAIN_OUT);
                        w.set_mode(7, Mode::OUTPUT_50MHZ);
                        w.set_cnf(7, Cnf::AF_OPEN_DRAIN_OUT);

                        w.set_mode(0, Mode::OUTPUT_50MHZ);
                        w.set_cnf(0, Cnf::ANALOG_IN__PUSH_PULL_OUT);
                    })
                });
                pac::GPIOD.cfglr().modify(|w| {
                    w.set_mode(0, Mode::OUTPUT_50MHZ);
                    w.set_cnf(0, Cnf::AF_OPEN_DRAIN_OUT);
                    w.set_mode(5, Mode::OUTPUT_50MHZ);
                    w.set_cnf(5, Cnf::AF_OPEN_DRAIN_OUT);
                    w.set_mode(6, Mode::OUTPUT_50MHZ);
                    w.set_cnf(6, Cnf::AF_OPEN_DRAIN_OUT);

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
        matrix::Pins::set_float_all();
        let result = self.setup_next_group();
        matrix::Pins::set_high(self.row);

        if let Some((btn_cnt, intfr)) = result {
            buttons::process_samples(
                buttons::Sample {
                    start_cnt: self.start_cnt,
                    btn_cnt,
                    intfr,
                },
                &mut self.fcount,
            );
        }
    }
}

#[interrupt]
fn TIM1_UP() {
    let mut intfr = pac::TIM1.intfr().read();

    if intfr.uif() {
        intfr.set_uif(false);
        pac::TIM1.intfr().write_value(intfr);

        unsafe {
            #[allow(static_mut_refs)]
            TIMER_DRIVER.advance();
        }

        time::time_driver().advance();
    }
}

pub unsafe fn init() {
    matrix::Pins::init();

    pac::TIM1.atrlr().write_value(CYCLES - 1);
    pac::TIM2.atrlr().write_value(CYCLES - 1);

    pac::TIM1.chctlr_output(0).write(|w| {
        w.set_ocm(0, Ocm::PWMMODE2);
        w.set_ocm(1, Ocm::PWMMODE2);
    });
    pac::TIM1.chctlr_output(1).write(|w| {
        w.set_ocm(1, Ocm::PWMMODE2);
    });
    pac::TIM1.bdtr().write(|w| w.set_moe(true));

    // Enable interrupt
    pac::TIM1.dmaintenr().write(|r| r.set_uie(true));

    // Configure tim2 as slave of tim1 (tim1 enable also controls tim2)
    pac::TIM1.ctlr2().write(|w| w.set_mms(Mms::ENABLE));
    pac::TIM2.smcfgr().write(|w| w.set_sms(0b101));

    pac::TIM2.ctlr1().write(|w| {
        w.set_arpe(true);
        w.set_dir(Dir::UP);
        w.set_cms(Cms::EDGEALIGNED);

        w.set_cen(true);
    });

    pac::TIM1.ctlr1().write(|w| {
        w.set_arpe(true);
        w.set_dir(Dir::UP);
        w.set_cms(Cms::EDGEALIGNED);

        w.set_urs(Urs::COUNTERONLY); // Trigger interrupt only on overflow

        w.set_cen(true);
    });

    unsafe { pfic::enable_interrupt(Interrupt::TIM1_UP as u8); }
}
