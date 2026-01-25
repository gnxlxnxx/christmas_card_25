use core::task::Poll;

use crate::drivers::buttons::{Button, Buttons, Event};
use crate::drivers::matrix::{Framebuffer, Matrix};
use crate::util::rand::Rng;
use embassy_time::{Duration, Ticker};

const BOARD_BRIGHTNESS: u8 = 50;
const PIECE_BRIGHTNESS: u8 = 100;

const PIECES: [u8; 7] = [
    0b0000_1111, // I
    0b0110_0110, // O
    0b1110_0100, // T
    0b0011_0110, // И
    0b0110_0011, // Z
    0b1000_1110, // Г
    0b0010_1110, // L
];

const WIDTH: u8 = Framebuffer::WIDTH as u8;
const HEIGHT: u8 = Framebuffer::HEIGHT as u8;

const _: () = {
    assert!(Framebuffer::WIDTH <= i8::MAX as usize);
    assert!(Framebuffer::HEIGHT <= i8::MAX as usize);
    assert!(WIDTH as u32 <= u8::BITS);
};

#[derive(Clone, Copy)]
struct GameResult {
    pub score: u32,
}

#[derive(Clone, Copy)]
struct Piece {
    shape: u16,
    x: u8,
    y: u8,
}

impl Piece {
    fn _bounds(&self) -> ((u8, u8), (u8, u8)) {
        let mut min_x = 4;
        let mut max_x = 0;
        let mut min_y = 4;
        let mut max_y = 0;
        for i in 0..16 {
            if (self.shape >> i) & 1 == 1 {
                let x = (i % 4) as u8;
                let y = (i / 4) as u8;
                if x < min_x {
                    min_x = x;
                }
                if x > max_x {
                    max_x = x;
                }
                if y < min_y {
                    min_y = y;
                }
                if y > max_y {
                    max_y = y;
                }
            }
        }
        ((min_x, max_x), (min_y, max_y))
    }

    fn rotate(&mut self, cw: bool) {
        let mut out = 0;
        for i in 0..16 {
            let x = i % 4;
            let y = i / 4;

            let src = y * 4 + x;
            let dst = if cw {
                x * 4 + 3 - y
            } else {
                (3 - x) * 4 + y
            };

            out |= ((self.shape >> src) & 1) << dst;
        }
        self.shape = out;
    }
}

struct Tetris<'a> {
    board: [u8; Framebuffer::HEIGHT],
    current: Piece,
    rng: Rng,
    fb: &'a Framebuffer,
    score: u32,
}

impl<'a> Tetris<'a> {
    pub fn new() -> Self {
        let mut t = Self {
            board: [0; Framebuffer::HEIGHT],
            current: Piece {
                shape: 0,
                x: 0,
                y: 0,
            },
            rng: Rng::new(),
            fb: Matrix::fb(),
            score: 0,
        };
        t.spawn();

        t
    }

    pub fn input(&mut self, but: Button) {
        let mut p = self.current;

        match but {
            Button::L => {
                p.x = p.x.wrapping_sub(1);
            }
            Button::R => {
                p.x = p.x.wrapping_add(1);
            }
            Button::Select => {
                p.rotate(true);
            }
            Button::Start => {
                p.rotate(false);
            }
        }

        if !self.collides(&p) {
            self.current = p;
        }
    }

    pub fn draw(&self) {
        self.fb.clear_all();

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if self.board[y as usize] & (1 << x) != 0 {
                    self.fb.store(x as usize, y as usize, BOARD_BRIGHTNESS);
                }
            }
        }

        let p = self.current;
        for i in 0..16 {
            if (p.shape >> i) & 1 == 0 {
                continue;
            }
            let x = p.x.wrapping_add((i % 4) as u8);
            let y = p.y.wrapping_add((i / 4) as u8);

            if x < WIDTH && y < HEIGHT {
                self.fb.store(x as usize, y as usize, PIECE_BRIGHTNESS);
            }
        }
    }

    fn spawn(&mut self) -> bool {
        let i = (self.rng.rand8() % 7) as usize;
        self.current = Piece {
            shape: (PIECES[i] as u16) << 4,
            x: (WIDTH / 2) - 2,
            y: -1i8 as u8,
        };

        self.collides(&self.current)
    }

    fn collides(&self, p: &Piece) -> bool {
        for i in 0..16 {
            if (p.shape >> i) & 1 == 0 {
                continue;
            }

            let px = p.x.wrapping_add((i as u8) % 4);
            let py = p.y.wrapping_add((i as u8) / 4);

            if (py as i8) < 0 {
                continue;
            }

            if py >= HEIGHT || px >= WIDTH {
                return true;
            }
            if self.board[py as usize] & (1 << px) != 0 {
                return true;
            }
        }
        false
    }

    fn lock(&mut self) {
        let p = self.current;

        for i in 0..16 {
            if (p.shape >> i) & 1 == 0 {
                continue;
            }

            let x = p.x.wrapping_add((i % 4) as u8);
            let y = p.y.wrapping_add((i / 4) as u8);
            if y < HEIGHT {
                self.board[y as usize] |= 1 << x;
            }
        }
    }

    fn clear_lines(&mut self) {
        let mut cleared = 0;

        for y in 0..(HEIGHT as usize) {
            let full = self.board[y] == ((1 << (WIDTH as u32)) - 1) as u8;
            if full {
                for y_up in (1..=y).rev() {
                    self.board[y_up] = self.board[y_up - 1];
                }
                self.board[0] = 0;
                cleared += 1;
            }
        }

        self.score += cleared * cleared;
    }

    fn tick(&mut self) -> Option<GameResult> {
        let mut p = self.current;
        p.y = p.y.wrapping_add(1);

        if self.collides(&p) {
            self.lock();
            self.clear_lines();

            if self.spawn() {
                return Some(GameResult { score: self.score });
            }
        } else {
            self.current = p;
        }

        None
    }
}

enum TaskState {
    Game(Ticker, Tetris<'static>),
    Score(super::ShowScoreTask),
}

pub struct Task(TaskState);

impl Task {
    pub fn new() -> Self {
        let game = Tetris::new();
        game.draw();

        Self(TaskState::Game(Ticker::every(Duration::from_millis(500)), game))
    }

    pub fn poll(&mut self) -> bool {
        match &mut self.0 {
            TaskState::Game(ticker, game) => {
                if let Poll::Ready(Event {
                    pressed: true,
                    button,
                }) = Buttons::event()
                {
                    game.input(button);
                    game.draw();
                }

                if ticker.consume_expired() {
                    if let Some(res) = game.tick() {
                        self.0 = TaskState::Score(super::ShowScoreTask::new(
                            super::GameSelection::Tetris,
                            false,
                            res.score,
                        ));
                    } else {
                        game.draw();
                    }
                }

                false
            }
            TaskState::Score(show_score_task) => show_score_task.poll(),
        }
    }
}
