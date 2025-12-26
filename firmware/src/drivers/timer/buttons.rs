use core::sync::atomic::{AtomicBool, Ordering};

use ch32_hal::{Peri, pac, peripherals};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal,
};

const HIST_LOW: u16 = 80;
const HIST_HIGH: u16 = 96;
const FILT_LEN: u8 = 6;

static BTN_STATE: Group<AtomicBool> = Group([
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
]);

pub(super) static BTN_SAMPLE_SIGNAL: Signal<CriticalSectionRawMutex, Sample> = Signal::new();
static BTN_EVENT_CHANNEL: Channel<CriticalSectionRawMutex, Event, 3> = Channel::new();

#[derive(Debug)]
pub struct Pins<'a> {
    start: Peri<'a, peripherals::PD7>,
    select: Peri<'a, peripherals::PD4>,
    l: Peri<'a, peripherals::PC0>,
    r: Peri<'a, peripherals::PD3>,
}

impl<'a> Pins<'a> {
    pub fn new(
        start: Peri<'a, peripherals::PD7>,
        select: Peri<'a, peripherals::PD4>,
        l: Peri<'a, peripherals::PC0>,
        r: Peri<'a, peripherals::PD3>,
    ) -> Self {
        Self { start, select, l, r }
    }

    pub(super) fn set_high_all(&mut self) {
        pac::GPIOC.bshr().write(|w| {
            w.set_bs(0, true);
        });
        pac::GPIOD.bshr().write(|w| {
            w.set_bs(3, true);
            w.set_bs(4, true);
            w.set_bs(7, true);
        });
    }

    pub(super) fn set_low_all(&mut self) {
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

    pub fn set(&mut self, btn: Button, val: T) {
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

#[embassy_executor::task]
pub(super) async fn process_samples() {
    let mut fcount = Group::<u8>::default();

    loop {
        let sample = BTN_SAMPLE_SIGNAL.wait().await;
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
                    BTN_EVENT_CHANNEL.send(Event {
                        button: ch.try_into().unwrap(),
                        pressed: next_state,
                    }).await;

                    0
                }
            } else {
                0
            }
        }
    }
}

pub struct Buttons;

impl Buttons {
    pub fn get(btn: Button) -> bool {
        BTN_STATE.get(btn).load(Ordering::Relaxed)
    }

    pub async fn event() -> Event {
        BTN_EVENT_CHANNEL.receive().await
    }

    pub async fn event_filtered(btn: Option<Button>, pressed: Option<bool>) -> Event {
        loop {
            let event = Self::event().await;
            if btn.map_or(true, |b| b == event.button)
                && pressed.map_or(true, |p| p == event.pressed)
            {
                return event;
            }
        }
    }
}
