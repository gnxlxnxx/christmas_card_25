use core::sync::atomic::Ordering;

use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Ticker};

use crate::drivers::{buttons::{Button, Buttons, Event}, matrix::{Framebuffer, Matrix}};


const PADDLE_BRIGHTNESS: u8 = 32;
const BALL_BRIGHTNESS: u8 = 64;

struct Ball {
    pub x: i8,
    pub y: i8,
    pub x_dir: i8,
    pub y_dir: i8,
}

impl Ball {
    pub fn with_y_dir(y_dir: i8) -> Self {
        Self {
            x: Framebuffer::WIDTH as i8 / 2,
            y: Framebuffer::HEIGHT as i8 / 2,
            x_dir: 0,
            y_dir,
        }
    }

    pub fn new() -> Self {
        Self::with_y_dir(1)
    }

    pub fn advance(&mut self) {
        if self.x == 0 || self.x == Framebuffer::WIDTH as i8 - 1 {
            self.x_dir = -self.x_dir;
        }

        self.x += self.x_dir;
        self.y += self.y_dir;
    }

    pub fn draw(&self, fb: &Framebuffer) {
        fb.store(self.x as usize, self.y as usize, BALL_BRIGHTNESS);
    }
}

struct Player<const TOP: bool> {
    pub misses: u8,
    paddle_x: i8,
}

impl<const TOP: bool> Player<TOP> {
    const PADDLE_LEN: i8 = 3;
    const PADDLE_X_MAX: i8 = Framebuffer::WIDTH as i8 - Self::PADDLE_LEN;
    const PADDLE_Y: i8 = if TOP { 0 } else { Framebuffer::HEIGHT as i8 - 1 };

    pub fn new() -> Self {
        Self {
            misses: 0,
            paddle_x: Self::PADDLE_X_MAX / 2,
        }
    }

    pub fn collide_ball(&mut self, ball: &mut Ball) -> bool {
        if ball.y == Self::PADDLE_Y {
            let delta = ball.x - self.paddle_x;

            if 0 <= delta && delta <= Self::PADDLE_LEN - 1 {
                ball.x_dir = if delta == 0 {
                    -1
                } else if delta == Self::PADDLE_LEN - 1 {
                    1
                } else {
                    0
                };
                ball.y_dir = -ball.y_dir;

                ball.advance();
            } else {
                *ball = Ball::with_y_dir(if TOP { 1 } else { -1 });
                self.misses += 1;

                return true;
            }
        }

        false
    }

    pub fn move_paddle(&mut self, val: i8) {
        self.paddle_x = (self.paddle_x + val).max(0).min(Self::PADDLE_X_MAX);
    }

    pub fn reset_paddle(&mut self) {
        self.paddle_x = Self::PADDLE_X_MAX / 2;
    }

    pub fn draw_paddle(&self, fb: &Framebuffer) {
        let row = &fb.0[Self::PADDLE_Y as usize];

        for field in row.iter().skip(self.paddle_x as usize).take(Self::PADDLE_LEN as usize) {
            field.store(PADDLE_BRIGHTNESS, Ordering::Relaxed);
        }
    }
}

enum AdvanceResult {
    Finished {
        top_score: u8,
        bottom_score: u8,
    },
    Missed,
    None,
}

struct Game {
    ball: Ball,
    pub top: Player<true>,
    pub bottom: Player<false>,
}

impl Game {
    const SCORE_MAX: u8 = 10;

    pub fn new() -> Self {
        let g = Self {
            ball: Ball::new(),
            top: Player::new(),
            bottom: Player::new(),
        };

        g
    }

    pub fn advance(&mut self) -> AdvanceResult {
        self.ball.advance();

        if self.top.collide_ball(&mut self.ball) || self.bottom.collide_ball(&mut self.ball) {
            if self.top.misses == Self::SCORE_MAX || self.bottom.misses == Self::SCORE_MAX {
                AdvanceResult::Finished {
                    top_score: self.bottom.misses,
                    bottom_score: self.top.misses,
                }
            } else {
                self.top.reset_paddle();
                self.bottom.reset_paddle();

                AdvanceResult::Missed
            }
        } else {
            AdvanceResult::None
        }
    }

    pub fn draw(&self, fb: &Framebuffer) {
        fb.clear_all();
        self.top.draw_paddle(fb);
        self.bottom.draw_paddle(fb);
        self.ball.draw(fb);
    }
}

pub async fn run() {
    let mut g = Game::new();
    let mut clock = Ticker::every(Duration::from_millis(250));

    g.draw(Matrix::fb());

    loop {
        match select(Buttons::event(), clock.next()).await {
            Either::First(Event { pressed: true, button }) => match button {
                Button::Start => g.top.move_paddle(-1),
                Button::Select => g.top.move_paddle(1),
                Button::L => g.bottom.move_paddle(-1),
                Button::R => g.bottom.move_paddle(1),
            }
            Either::First(Event { pressed: false, button: _ }) => (),
            Either::Second(()) => {
                match g.advance() {
                    AdvanceResult::Finished { top_score, bottom_score } => break,
                    AdvanceResult::Missed => clock.reset_after(Duration::from_millis(1500)),
                    AdvanceResult::None => (),
                }
            }
        }

        g.draw(Matrix::fb());
    }
}
