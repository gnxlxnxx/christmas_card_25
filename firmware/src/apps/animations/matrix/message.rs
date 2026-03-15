use embassy_time::Ticker;

use crate::util::text::{TEXT_DURATION, TextScroller};

const MESSAGE: &[u8] = b"Die Fachschaft Elektro- und Informationstechnik an der \x7f Universit\x80t Stuttgart w\x82nscht Euch allen recht herzlich ein frohes Weihnachtsfest!";

pub struct Task {
    ticker: Ticker,
    scroller: TextScroller,
    clear: bool,
}

impl Task {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ticker: Ticker::every(TEXT_DURATION),
            scroller: TextScroller::new(),
            clear: true,
        }
    }

    pub fn poll(&mut self) {
        if self.ticker.consume_expired() {
            if self.clear {
                if self.scroller.advance_clear() {
                    self.scroller = TextScroller::new();
                    self.clear = false;
                }
            } else {
                if self.scroller.advance(MESSAGE) {
                    self.scroller = TextScroller::new();
                    self.clear = true;
                }
            }
        }
    }
}
