use core::sync::atomic::Ordering;

use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Instant, Timer};

use crate::drivers::{
    buttons::{Button, Buttons, Event},
    matrix::{self, Framebuffer, Matrix},
};

const SCORE_BRIGHTNESS: u8 = 16;
const END_SCORE_BRIGHTNESS: u8 = 32;
const PADDLE_BRIGHTNESS: u8 = 32;
const BALL_BRIGHTNESS: u8 = 96;

const RESET_PAUSE_DURATION: Duration = Duration::from_millis(1500);
const INITIAL_DURATION: Duration = Duration::from_millis(200);

const _: () = {
    assert!(Framebuffer::WIDTH <= i8::MAX as usize);
    assert!(Framebuffer::HEIGHT <= i8::MAX as usize);
};

struct Ball {
    pub x: i8,
    pub y: i8,
    pub x_dir: i8,
    pub y_dir: i8,
}

impl Ball {
    pub const fn with_y_dir(y_dir: i8) -> Self {
        Self {
            x: (Framebuffer::WIDTH as i8 - 1) / 2,
            y: (Framebuffer::HEIGHT as i8 - 1) / 2,
            x_dir: 0,
            y_dir,
        }
    }

    pub const fn new() -> Self {
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
    const PADDLE_OVERSHOOT: i8 = 1;
    const PADDLE_X_MIN: i8 = -Self::PADDLE_OVERSHOOT;
    const PADDLE_X_MAX: i8 = Framebuffer::WIDTH as i8 - Self::PADDLE_LEN + Self::PADDLE_OVERSHOOT;
    const PADDLE_X_INITIAL: i8 = (Framebuffer::WIDTH as i8 - Self::PADDLE_LEN) / 2;
    const PADDLE_Y: i8 = if TOP { 0 } else { Framebuffer::HEIGHT as i8 - 1 };

    pub const fn new() -> Self {
        Self {
            misses: 0,
            paddle_x: Self::PADDLE_X_INITIAL,
        }
    }

    pub fn collide_ball(&mut self, ball: &mut Ball) -> CollideResult {
        if ball.y == Self::PADDLE_Y {
            let delta = ball.x - self.paddle_x;

            if (0..=Self::PADDLE_LEN - 1).contains(&delta) {
                ball.x_dir = if delta == 0 {
                    -1
                } else if delta == Self::PADDLE_LEN - 1 {
                    1
                } else {
                    0
                };
                ball.y_dir = -ball.y_dir;

                CollideResult::Hit
            } else {
                *ball = Ball::with_y_dir(if TOP { 1 } else { -1 });
                self.misses += 1;

                CollideResult::Missed
            }
        } else {
            CollideResult::None
        }
    }

    pub fn move_paddle(&mut self, val: i8) {
        self.paddle_x = (self.paddle_x + val).clamp(Self::PADDLE_X_MIN, Self::PADDLE_X_MAX);
    }

    pub fn reset_paddle(&mut self) {
        self.paddle_x = Self::PADDLE_X_INITIAL;
    }

    pub fn draw_paddle(&self, fb: &Framebuffer) {
        let row = &fb.0[Self::PADDLE_Y as usize];

        for x in self.paddle_x..(self.paddle_x + Self::PADDLE_LEN) {
            if let Ok(x) = usize::try_from(x)
                && let Some(field) = row.get(x)
            {
                field.store(PADDLE_BRIGHTNESS, Ordering::Relaxed);
            }
        }
    }

    pub fn draw_score(&self, fb: &Framebuffer, brightness: u8) {
        let row = &fb.0[if TOP {
            Framebuffer::HEIGHT.div_ceil(2)
        } else {
            (Framebuffer::HEIGHT - 2) / 2
        }];

        for field in row.iter().take(self.misses as usize) {
            field.store(brightness, Ordering::Relaxed);
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum CollideResult {
    Missed,
    Hit,
    None,
}

impl CollideResult {
    pub fn or(self, f: impl FnOnce() -> Self) -> Self {
        if self == Self::None { f() } else { self }
    }
}

struct Game {
    ball: Ball,
    pub top: Player<true>,
    pub bottom: Player<false>,
}

impl Game {
    const SCORE_MAX: u8 = Framebuffer::WIDTH as u8;

    pub const fn new() -> Self {
        Self {
            ball: Ball::new(),
            top: Player::new(),
            bottom: Player::new(),
        }
    }

    pub fn advance(&mut self) -> CollideResult {
        let res = self.top.collide_ball(&mut self.ball)
            .or(|| self.bottom.collide_ball(&mut self.ball));

        if res == CollideResult::Missed {
            self.top.reset_paddle();
            self.bottom.reset_paddle();
        } else {
            self.ball.advance();
        }

        res
    }

    pub fn has_ended(&self) -> bool {
        self.top.misses == Self::SCORE_MAX || self.bottom.misses == Self::SCORE_MAX
    }

    pub fn draw_score(&self, fb: &Framebuffer, brightness: u8) {
        fb.clear_all();
        self.top.draw_score(fb, brightness);
        self.bottom.draw_score(fb, brightness);
    }

    pub fn draw(&self, fb: &Framebuffer) {
        self.draw_score(fb, SCORE_BRIGHTNESS);
        self.top.draw_paddle(fb);
        self.bottom.draw_paddle(fb);
        self.ball.draw(fb);
    }
}

pub async fn run() {
    let mut g = Game::new();
    let mut next_tick = Instant::now() + RESET_PAUSE_DURATION;
    const {
        assert!(INITIAL_DURATION.as_ticks() <= u32::MAX as u64);
    }
    let mut ticks = INITIAL_DURATION.as_ticks() as u32;

    loop {
        g.draw(Matrix::fb());

        match select(Buttons::event(), Timer::at(next_tick)).await {
            Either::First(Event { pressed: true, button }) => match button {
                Button::Start => g.top.move_paddle(-1),
                Button::Select => g.top.move_paddle(1),
                Button::L => g.bottom.move_paddle(-1),
                Button::R => g.bottom.move_paddle(1),
            },
            Either::First(Event { pressed: false, button: _ }) => (),
            Either::Second(()) => {
                next_tick += match g.advance() {
                    CollideResult::Missed => {
                        if g.has_ended() {
                            break;
                        } else {
                            ticks = INITIAL_DURATION.as_ticks() as u32;

                            RESET_PAUSE_DURATION
                        }
                    }
                    CollideResult::Hit => {
                        const {
                            assert!(matrix::FRAME_DURATION.as_ticks() <= u32::MAX as u64);
                        }
                        ticks = (((ticks << 5) - ticks) >> 5)
                            .max(matrix::FRAME_DURATION.as_ticks() as u32);

                        Duration::from_ticks(ticks as u64)
                    }
                    CollideResult::None => Duration::from_ticks(ticks as u64),
                }
            }
        }
    }

    g.draw_score(Matrix::fb(), END_SCORE_BRIGHTNESS);

    Timer::after_secs(5).await;
}
