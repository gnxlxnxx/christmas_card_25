use core::{sync::atomic::Ordering, task::Poll};

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

    pub fn draw_score(&self, brightness: u8) {
        let fb = Matrix::fb();
        fb.clear_all();
        self.top.draw_score(fb, brightness);
        self.bottom.draw_score(fb, brightness);
    }

    pub fn draw(&self) {
        let fb = Matrix::fb();
        self.draw_score(SCORE_BRIGHTNESS);
        self.top.draw_paddle(fb);
        self.bottom.draw_paddle(fb);
        self.ball.draw(fb);
    }
}

enum TaskState {
    Game(Game, Instant, Duration),
    Score(Timer),
}

pub struct Task(TaskState);

impl Task {
    pub fn new() -> Self {
        let game = Game::new();
        game.draw();

        Self(
            TaskState::Game(
                game,
                Instant::now() + RESET_PAUSE_DURATION,
                INITIAL_DURATION,
            ),
        )
    }

    pub fn poll(&mut self) -> bool {
        match &mut self.0 {
            TaskState::Game(game, next_instant, duration) => {
                if let Poll::Ready(Event { pressed: true, button }) = Buttons::event() {
                    match button {
                        Button::Start => game.top.move_paddle(-1),
                        Button::Select => game.top.move_paddle(1),
                        Button::L => game.bottom.move_paddle(-1),
                        Button::R => game.bottom.move_paddle(1),
                    }
                    game.draw();
                }

                if *next_instant <= Instant::now() {
                    *next_instant += match game.advance() {
                        CollideResult::Missed => {
                            if game.has_ended() {
                                game.draw_score(END_SCORE_BRIGHTNESS);
                                self.0 = TaskState::Score(Timer::after_secs(5));

                                return false;
                            } else {
                                *duration = INITIAL_DURATION;

                                RESET_PAUSE_DURATION
                            }
                        }
                        CollideResult::Hit => {
                            let dur = duration.as_ticks();
                            *duration = Duration::from_ticks(((dur << 5) - dur) >> 5)
                                .max(matrix::FRAME_DURATION);

                            *duration
                        }
                        CollideResult::None => *duration,
                    };
                    game.draw();
                }

                false
            }
            TaskState::Score(timer) => {
                timer.expired()
            }
        }
    }
}
