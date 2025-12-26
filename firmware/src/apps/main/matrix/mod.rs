pub mod rand_pulse;
pub mod snowfall;
pub mod sparkle;

pub trait MatrixMode {
    fn animate(&mut self) -> impl core::future::Future<Output = ()>;
}

pub enum Mode {
    Sparkle(sparkle::Sparkle),
    Snowfall(snowfall::Snowfall),
    RandPulse(rand_pulse::RandPulse),
}

impl Mode {
    pub fn new() -> Self {
        Self::Snowfall(snowfall::Snowfall::new())
    }

    pub fn next(&mut self) {
        *self = match self {
            Self::Sparkle(_) => Self::Snowfall(snowfall::Snowfall::new()),
            Self::Snowfall(_) => Self::RandPulse(rand_pulse::RandPulse::new()),
            Self::RandPulse(_) => Self::Sparkle(sparkle::Sparkle::new()),
        }
    }
}

impl MatrixMode for Mode {
    async fn animate(&mut self) {
        match self {
            Self::Sparkle(a) => a.animate().await,
            Self::Snowfall(a) => a.animate().await,
            Self::RandPulse(a) => a.animate().await,
        }
    }
}
