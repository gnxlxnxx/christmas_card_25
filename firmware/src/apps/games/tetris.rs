use crate::drivers::buttons::{Button, Buttons, Event};
use crate::drivers::flash;
use crate::matrix::{Framebuffer, Matrix};
use crate::util::{
    itoa::utoa10,
    rand::WhiteNoiseGenerator,
    text::{self, TEXT_BRIGHTNESS, TEXT_DURATION},
};
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Ticker};

const WIDTH: u8 = Framebuffer::WIDTH as u8;
const HEIGHT: u8 = Framebuffer::HEIGHT as u8;

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
    fn bounds(&self) -> ((u8, u8), (u8, u8)) {
        let mut min_x = 4;
        let mut max_x = 0;
        let mut min_y = 4;
        let mut max_y = 0;
        for i in 0..16 {
            if (self.shape >> i) & 1 == 1 {
                let x = (i & 3) as u8;
                let y = (i >> 2) as u8;
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
            let x = (i & 3) as u8;
            let y = (i >> 2) as u8;

            let src = (y << 2) + x; // y*4 + x
            let dst = if cw {
                (x << 2) + (3 - y) // x*4 + (3-y)
            } else {
                ((3 - x) << 2) + y // (3-x)*4 + y
            };

            out |= ((self.shape >> src) & 1) << dst;
        }
        self.shape = out;
    }
}

struct Tetris<'a> {
    board: [[u8; Framebuffer::WIDTH]; Framebuffer::HEIGHT],
    current: Piece,
    rng: WhiteNoiseGenerator,
    fb: &'a Framebuffer,
    score: u32,
}

const PIECES: [u8; 7] = [
    0b1111_0000, // I
    0b0110_0110, // O
    0b1110_0100, // T
    0b0011_0110, // И
    0b0110_0011, // Z
    0b1000_1110, // Г
    0b0010_1110, // L
];

impl<'a> Tetris<'a> {
    pub fn new() -> Self {
        let mut t = Self {
            board: [[0; Framebuffer::WIDTH]; Framebuffer::HEIGHT],
            current: Piece {
                shape: 0,
                x: 0,
                y: 0,
            },
            rng: WhiteNoiseGenerator::new(),
            fb: Matrix::fb(),
            score: 0,
        };
        t.spawn();
        t
    }

    fn spawn(&mut self) {
        let i = (self.rng.rand8() % 7) as usize;
        self.current = Piece {
            shape: (PIECES[i] as u16) << 4,
            x: (WIDTH / 2) - 2,
            y: 0,
        };
        self.current.y -= self.current.bounds().1.0;
    }

    fn collides(&self, p: &Piece) -> bool {
        for i in 0..16 {
            if (p.shape >> i) & 1 == 0 {
                continue;
            }

            let px = p.x + ((i as u8) & 3); // funny %4
            let py = p.y + ((i as u8) >> 2); // funny /4

            if py >= HEIGHT || px >= WIDTH {
                return true;
            }
            if self.board[py as usize][px as usize] != 0 {
                return true;
            }
        }
        false
    }

    fn lock(&mut self) -> Option<GameResult> {
        let p = self.current;

        for i in 0..16 {
            if (p.shape >> i) & 1 == 0 {
                continue;
            }

            let x = p.x + (i & 3) as u8;
            let y = p.y + (i >> 2) as u8;
            if x < WIDTH && y < HEIGHT {
                self.board[y as usize][x as usize] = 1;
            }
        }

        self.clear_lines();
        let lost = self.board[1].iter().any(|&c| c != 0);
        if lost {
            Some(GameResult { score: self.score })
        } else {
            self.spawn();
            None
        }
    }

    fn clear_lines(&mut self) {
        for y in 0..Framebuffer::HEIGHT {
            let full = self.board[y].iter().all(|&c| c != 0);
            if full {
                for y_up in (1..=y).rev() {
                    self.board[y_up] = self.board[y_up - 1];
                }
                self.board[0] = [0; Framebuffer::WIDTH];
                self.score += 1;
            }
        }
    }

    pub fn input(&mut self, but: Button) {
        let mut p = self.current;

        match but {
            Button::L => {
                p.x -= 1;
            }
            Button::R => {
                p.x += 1;
            }
            _ => {
                p.rotate(true);
            }
        }

        if !self.collides(&p) {
            self.current = p;
        }
    }

    fn tick(&mut self) -> Option<GameResult> {
        let mut p = self.current;
        p.y += 1;
        if self.collides(&p) {
            self.lock()
        } else {
            self.current = p;
            None
        }
    }

    pub fn draw(&mut self) {
        self.fb.clear_all();

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if self.board[y as usize][x as usize] != 0 {
                    self.fb.store(x as usize, y as usize, 50);
                }
            }
        }

        let p = self.current;
        for i in 0..16 {
            if (p.shape >> i) & 1 == 0 {
                continue;
            }
            let x = p.x + (i & 3) as u8;
            let y = p.y + (i >> 2) as u8;

            if x < WIDTH && y < HEIGHT {
                self.fb.store(x as usize, y as usize, 100);
            }
        }
    }
}

pub async fn run() {
    let mut game = Tetris::new();
    let mut ticker = Ticker::every(Duration::from_millis(500));

    let res = loop {
        let event = select(ticker.next(), Buttons::event()).await;

        match event {
            Either::First(_) => {
                if let Some(res) = game.tick() {
                    break res;
                }
            }
            Either::Second(ev) => match ev {
                Event {
                    button: b,
                    pressed: true,
                } => game.input(b),
                _ => {}
            },
        }

        game.draw();
    };

    let hs = flash::new_high_score(super::Game::Tetris.high_score_index(), res.score).await;

    super::show_score(false, res.score, hs).await;
}
