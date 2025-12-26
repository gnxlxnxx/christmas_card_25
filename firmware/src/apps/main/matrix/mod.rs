use embassy_futures::select::{Either, Either3, Either4, select, select3, select4};
use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal, watch};
use embassy_time::{Duration, Ticker, Timer};

mod message;
mod rand_pulse;
mod snowfall;
mod sparkle;

trait MatrixMode {
    fn animate(&mut self) -> impl core::future::Future<Output = ()>;
}

enum Mode {
    Message,
    Snowfall(snowfall::Snowfall),
    Sparkle(sparkle::Sparkle),
    RandPulse(rand_pulse::RandPulse),
}

impl Mode {
    pub fn new() -> Self {
        Self::Message
    }

    pub fn next(&mut self) {
        *self = match self {
            Self::Message => Self::Snowfall(snowfall::Snowfall::new()),
            Self::Snowfall(_) => Self::Sparkle(sparkle::Sparkle::new()),
            Self::Sparkle(_) => Self::RandPulse(rand_pulse::RandPulse::new()),
            Self::RandPulse(_) => Self::Message,
        }
    }

    pub fn auto_next(&mut self) {
        *self = match self {
            Self::Message => Self::Snowfall(snowfall::Snowfall::new()),
            Self::Snowfall(_) => Self::Sparkle(sparkle::Sparkle::new()),
            Self::Sparkle(_) => Self::RandPulse(rand_pulse::RandPulse::new()),
            Self::RandPulse(_) => Self::Snowfall(snowfall::Snowfall::new()),
        }
    }

    pub fn is_message(&self) -> bool {
        match self {
            Self::Message => true,
            _ => false,
        }
    }
}

impl MatrixMode for Mode {
    async fn animate(&mut self) {
        match self {
            Self::Message => message::run().await,
            Self::Sparkle(a) => a.animate().await,
            Self::Snowfall(a) => a.animate().await,
            Self::RandPulse(a) => a.animate().await,
        }
    }
}

pub async fn run(next_signal: &Signal<impl RawMutex, ()>) -> ! {
    let mut mode = Mode::new();

    loop {
        match select(
            mode.animate(),
            next_signal.wait()
        ).await {
            Either::First(()) => (),
            Either::Second(()) => {
                mode.next();
            },
        }
    }
}
