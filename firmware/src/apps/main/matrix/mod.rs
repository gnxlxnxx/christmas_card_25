use embassy_futures::select::select;
use embassy_sync::{blocking_mutex::raw::RawMutex};

use crate::util::sync::{Event, Signal};

mod message;
mod rand_pulse;
mod snowfall;
mod sparkle;

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

    async fn animate(&mut self) {
        match self {
            Self::Message => message::run().await,
            Self::Sparkle => sparkle::run().await,
            Self::Snowfall => snowfall::run().await,
            Self::RandPulse => rand_pulse::run().await,
        }
    }
}

pub async fn run(next_event: &Event) -> ! {
    let mut mode = Mode::new();

    loop {
        if select(mode.animate(), next_event.wait()).await.is_second() {
            mode.next();
        }
    }
}
