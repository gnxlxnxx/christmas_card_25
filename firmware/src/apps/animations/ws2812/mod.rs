use crate::util::ws2812::FilteredWs2812;

pub mod fire;
pub mod huewheel;
pub mod snowball;

pub enum Task {
    Fire(fire::Task),
    Snowball(snowball::Task),
    Huewheel(huewheel::Task),
}

impl Task {
    pub fn new() -> Self {
        Self::Fire(fire::Task::new())
    }

    pub fn next(&mut self) {
        *self = match self {
            Self::Fire(_) => Self::Snowball(snowball::Task::new()),
            Self::Snowball(_) => Self::Huewheel(huewheel::Task::new()),
            Self::Huewheel(_) => Self::Fire(fire::Task::new()),
        };
    }

    pub fn poll(&mut self, ws2812: &mut FilteredWs2812) {
        match self {
            Self::Fire(task) => task.poll(ws2812),
            Self::Snowball(task) => task.poll(ws2812),
            Self::Huewheel(task) => task.poll(ws2812),
        }
    }
}
