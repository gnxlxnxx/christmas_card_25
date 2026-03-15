use core::{
    sync::atomic::{AtomicBool, AtomicU8, Ordering},
    task::Poll,
};

use ch32_metapac as pac;

const HIST_LOW: u16 = 65;
const HIST_HIGH: u16 = 70;
const FILT_LEN: u8 = 16;

static BTN_STATE: Group<AtomicBool> = Group([
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
]);

static BTN_EVENT_SIGNAL: AtomicU8 = AtomicU8::new(0);

pub(super) struct Pins;

impl Pins {
    pub(super) fn set_high_all() {
        pac::GPIOC.bshr().write(|w| {
            w.set_bs(0, true);
        });
        pac::GPIOD.bshr().write(|w| {
            w.set_bs(3, true);
            w.set_bs(4, true);
            w.set_bs(7, true);
        });
    }

    pub(super) fn set_low_all() {
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
    R = 1,
}

impl TryFrom<usize> for Button {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Select),
            1 => Ok(Self::R),
            2 => Ok(Self::L),
            3 => Ok(Self::Start),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Event {
    pub button: Button,
    pub pressed: bool,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Default)]
pub(super) struct Group<T>(pub [T; 4]);

impl<T> Group<T> {
    pub const fn get(&self, btn: Button) -> &T {
        &self.0[btn as usize]
    }

    pub fn _set(&mut self, btn: Button, val: T) {
        self.0[btn as usize] = val;
    }

    pub const fn each_ref(&self) -> Group<&T> {
        Group(self.0.each_ref())
    }

    pub fn map<U>(self, f: impl FnMut(T) -> U) -> Group<U> {
        Group(self.0.map(f))
    }
}

#[derive(Clone, Copy, Default)]
pub(super) struct Sample {
    pub start_cnt: u16,
    pub btn_cnt: Group<u16>,
    pub intfr: pac::timer::regs::Intfr,
}

pub(super) fn process_samples(sample: Sample, fcount: &mut Group<u8>) {
    let state = BTN_STATE.each_ref().map(|s| s.load(Ordering::Relaxed));

    for (ch, (((prev_state, ext_state), cnt), fcount)) in state.0.iter()
        .zip(BTN_STATE.0.iter())
        .zip(sample.btn_cnt.0.iter())
        .zip(fcount.0.iter_mut())
        .enumerate()
    {
        let lvl = if sample.intfr.ccif(ch) {
            cnt - sample.start_cnt
        } else {
            u16::MAX
        };

        *fcount = if prev_state ^ (lvl >= if *prev_state { HIST_LOW } else { HIST_HIGH }) {
            if *fcount < FILT_LEN {
                *fcount + 1
            } else {
                let next_state = !prev_state;
                ext_state.store(next_state, Ordering::Relaxed);
                BTN_EVENT_SIGNAL.store(
                    (1 << 7) | (u8::from(next_state) << 2) | (ch as u8),
                    Ordering::Relaxed,
                );

                0
            }
        } else {
            0
        }
    }
}

pub struct Buttons;

impl Buttons {
    pub fn get(btn: Button) -> bool {
        BTN_STATE.get(btn).load(Ordering::Relaxed)
    }

    pub fn event() -> Poll<Event> {
        let val = BTN_EVENT_SIGNAL.load(Ordering::Relaxed);
        if val != 0 {
            BTN_EVENT_SIGNAL.store(0, Ordering::Relaxed);

            Poll::Ready(Event {
                button: Button::try_from(val as usize & ((1 << 2) - 1)).unwrap(),
                pressed: val & (1 << 2) != 0,
            })
        } else {
            Poll::Pending
        }
    }

    pub fn discard_events() {
        let _ = Self::event();
    }
}
