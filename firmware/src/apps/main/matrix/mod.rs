use embassy_futures::select::{Either3, Either4, select, select3, select4};
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

pub async fn run<const N: usize>(next_signal: &Signal<impl RawMutex, ()>, mut auto_receiver: watch::Receiver<'_, impl RawMutex, bool, N>) -> ! {
    let mut mode = Mode::new();
    let mut auto = auto_receiver.get().await;
    let mut clock = Ticker::every(Duration::MAX);

    loop {
        match select4(
            mode.animate(),
            next_signal.wait(),
            auto_receiver.changed(),
            clock.next()
        ).await {
            Either4::First(()) => {
                if auto && mode.is_message() {
                    clock = Ticker::every(super::AUTO_DURATION);
                    mode.auto_next();
                }
            },
            Either4::Second(()) => {
                mode.next();
            },
            Either4::Third(new_auto) => {
                auto = new_auto;
                if auto {
                    mode = Mode::new();
                } else {
                    clock = Ticker::every(Duration::MAX);
                }
            },
            Either4::Fourth(()) => {
                mode.auto_next();
            },
        }
    }
}
