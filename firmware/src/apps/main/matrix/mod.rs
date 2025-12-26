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
    Snowfall,
    Sparkle,
    RandPulse,
}

impl Mode {
    pub fn new() -> Self {
        Self::Message
    }

    pub fn next(&mut self) {
        *self = match self {
            Self::Message => Self::Snowfall,
            Self::Snowfall => Self::Sparkle,
            Self::Sparkle => Self::RandPulse,
            Self::RandPulse => Self::Message,
        }
    }
}

impl MatrixMode for Mode {
    async fn animate(&mut self) {
        match self {
            Self::Message => message::run().await,
            Self::Sparkle => sparkle::run().await,
            Self::Snowfall => snowfall::run().await,
            Self::RandPulse => rand_pulse::run().await,
        }
    }
}

pub async fn run(next_signal: &Signal<impl RawMutex, ()>) -> ! {
    let mut mode = Mode::new();

    loop {
        if select(mode.animate(), next_signal.wait()).await.is_second() {
            mode.next();
        }
    }
}
