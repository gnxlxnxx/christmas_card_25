use crate::util::ws2812::FilteredWs2812;

pub mod animations;
pub mod games;

pub enum Main {
    Animations(animations::Task),
    Games(games::Task),
}

impl Main {
    pub fn new() -> Self {
        Self::Animations(animations::Task::new())
    }

    pub fn poll(&mut self, ws2812: &mut FilteredWs2812) {
        match self {
            Self::Animations(app) => {
                if app.poll(ws2812) {
                    *self = Self::Games(games::Task::new(ws2812));
                }
            }
            Self::Games(app) => {
                if app.poll(ws2812) {
                    *self = Self::Animations(animations::Task::new());
                }
            }
        }
    }
}
