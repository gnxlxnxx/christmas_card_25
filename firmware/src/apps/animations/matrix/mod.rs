pub mod message;
pub mod rand_pulse;
pub mod snowfall;
pub mod sparkle;

pub enum Task {
    Message(message::Task),
    Snowfall(snowfall::Task),
    Sparkle(sparkle::Task),
    RandPulse(rand_pulse::Task),
}

impl Task {
    #[must_use]
    pub fn new() -> Self {
        Self::Message(message::Task::new())
    }

    pub fn next(&mut self) {
        *self = match self {
            Self::Message(_) => Self::Snowfall(snowfall::Task::new()),
            Self::Snowfall(_) => Self::Sparkle(sparkle::Task::new()),
            Self::Sparkle(_) => Self::RandPulse(rand_pulse::Task::new()),
            Self::RandPulse(_) => Self::Message(message::Task::new()),
        }
    }

    pub fn poll(&mut self) {
        match self {
            Self::Message(task) => task.poll(),
            Self::Sparkle(task) => task.poll(),
            Self::Snowfall(task) => task.poll(),
            Self::RandPulse(task) => task.poll(),
        }
    }
}
