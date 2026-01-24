use core::task::Poll;

use embassy_time::{Duration, Ticker};

use crate::{
    drivers::buttons::{Button, Buttons, Event},
    util::ws2812::FilteredWs2812,
};

pub mod matrix;
pub mod ws2812;

const AUTO_DURATION: Duration = Duration::from_secs(45);

pub struct Task {
    matrix: matrix::Task,
    ws2812: ws2812::Task,
    ticker: Option<Ticker>,
}

impl Task {
    pub fn new() -> Self {
        Self {
            matrix: matrix::Task::new(),
            ws2812: ws2812::Task::new(),
            ticker: Some(Ticker::every(AUTO_DURATION)),
        }
    }

    pub fn poll(&mut self, ws2812: &mut FilteredWs2812) -> bool {
        if let Poll::Ready(Event { pressed: true, button }) = Buttons::event() {
            match button {
                Button::Start => return true,
                Button::Select => {
                    self.ticker = Some(Ticker::every(AUTO_DURATION));
                }
                Button::L => {
                    self.ws2812.next();
                    self.ticker = None;
                }
                Button::R => {
                    self.matrix.next();
                    self.ticker = None;
                }
            }
        }

        if self.ticker.as_mut().is_some_and(|t| t.consume_expired()) {
            self.matrix.next();
            self.ws2812.next();
        }

        self.matrix.poll();
        self.ws2812.poll(ws2812);

        false
    }
}
