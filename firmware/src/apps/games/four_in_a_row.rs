use core::sync::atomic::Ordering;

use embassy_time::{Duration, Ticker};

use crate::drivers::matrix::{Framebuffer, Matrix};

const P1_BRIGHTNESS: u8 = 100;
const P2_BRIGHTNESS: u8 = P1_BRIGHTNESS / 2;

const WIDTH: u8 = 7;
const HEIGHT: u8 = 6;
const BASE_Y: usize = Framebuffer::HEIGHT - HEIGHT as usize;

const _: () = {
    assert!((WIDTH as usize) < Framebuffer::WIDTH);
    assert!((HEIGHT as usize) < Framebuffer::HEIGHT);
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Element {
    Player1,
    Player2,
    Empty,
}

struct LimitAnimation {
    ticker: Ticker,
    inc: bool,
}

impl LimitAnimation {
    pub fn new() -> Self {
        Self {
            ticker: Ticker::every(Duration::from_millis(20)),
            inc: false,
        }
    }

    pub fn poll(&mut self) {
        if self.ticker.consume_expired() {
            let mut brightness = Matrix::fb().0.last().unwrap().last().unwrap().load(Ordering::Relaxed);

            if brightness == 0 {
                self.inc = true;
            } else if brightness >= P1_BRIGHTNESS {
                self.inc = false;
            }

            if self.inc {
                brightness += 1;
            } else {
                brightness -= 1;
            }

            for row in Matrix::fb().0.iter().skip(BASE_Y) {
                row[WIDTH as usize].store(brightness, Ordering::Relaxed);
            }
        }
    }
}

struct Game {
    field: [[Element; WIDTH as usize]; HEIGHT as usize],
    selected: u8,
}

impl Game {
    pub fn new() -> Self {
        Self {
            field: [[Element::Empty; WIDTH as usize]; HEIGHT as usize],
            selected: WIDTH / 2,
        }
    }
}

pub struct Task {
    game: Game,
    limit_animation: LimitAnimation,
}

impl Task {
    pub fn new() -> Self {
        Self {
            game: Game::new(),
            limit_animation: LimitAnimation::new(),
        }
    }

    pub fn poll(&mut self) -> bool {
        self.limit_animation.poll();

        false
    }
}
