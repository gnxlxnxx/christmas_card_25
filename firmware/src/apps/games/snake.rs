use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Ticker};

use crate::{
    drivers::{
        buttons::{Button, Buttons, Event},
        flash,
        matrix::{self, Framebuffer, Matrix},
    },
    util::rand::Rng,
};

const HEAD_BRIGHTNESS: u8 = 64;
const SNAKE_BRIGHTNESS: u8 = 32;
const MAULTASCH_BRIGHTNESS: u8 = 96;

const MULTIPLIER_MIN_LENGTH: u8 = 2 * (Framebuffer::WIDTH + Framebuffer::HEIGHT - 2) as u8;
const INITIAL_DURATION: Duration = Duration::from_millis(250);

const _: () = assert!(Framebuffer::WIDTH * Framebuffer::HEIGHT <= u8::MAX as usize);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    pub fn cw(&self) -> Self {
        match self {
            Self::Up => Self::Right,
            Self::Right => Self::Down,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
        }
    }

    pub fn ccw(&self) -> Self {
        match self {
            Self::Left => Self::Down,
            Self::Down => Self::Right,
            Self::Right => Self::Up,
            Self::Up => Self::Left,
        }
    }

    pub fn delta(&self) -> (i32, i32) {
        match self {
            Self::Up => (0, -1),
            Self::Right => (1, 0),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum FieldState {
    Empty,
    Snake(Direction),
    SnakeHead,
    Maultasch,
}

impl FieldState {
    pub fn unwrap_dir(self) -> Direction {
        if let Self::Snake(dir) = self {
            dir
        } else {
            panic!("called `FieldState::unwrap_dir() on a value that is not `Snake`")
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct FbCoordinate {
    pub x: u8,
    pub y: u8,
}

impl FbCoordinate {
    pub fn random(rng: &mut Rng) -> Self {
        Self {
            x: rng.rand8() % Framebuffer::WIDTH as u8,
            y: rng.rand8() % Framebuffer::HEIGHT as u8,
        }
    }

    pub fn go_dir(&self, dir: Direction) -> Self {
        let x = self.x as i32;
        let y = self.y as i32;
        let (dx, dy) = dir.delta();

        Self {
            x: Self::wraparound(x + dx, Framebuffer::WIDTH),
            y: Self::wraparound(y + dy, Framebuffer::HEIGHT),
        }
    }

    pub fn set_fb(&self, fb: &Framebuffer, val: u8) {
        fb.store(self.x as usize, self.y as usize, val);
    }

    fn wraparound(val: i32, max: usize) -> u8 {
        if val >= max as i32 {
            0
        } else if val < 0 {
            (max - 1) as u8
        } else {
            val as u8
        }
    }
}

#[derive(Debug)]
struct Field<'a> {
    field: [[FieldState; Framebuffer::WIDTH]; Framebuffer::HEIGHT],
    fb: &'a Framebuffer,
}

impl<'a> Field<'a> {
    pub fn new(fb: &'a Framebuffer) -> Self {
        fb.clear_all();
        Self {
            field: [[FieldState::Empty; Framebuffer::WIDTH]; Framebuffer::HEIGHT],
            fb,
        }
    }

    pub fn get(&self, c: FbCoordinate) -> FieldState {
        self.field[c.y as usize][c.x as usize]
    }

    pub fn set(&mut self, c: FbCoordinate, state: FieldState) {
        self.field[c.y as usize][c.x as usize] = state;
        c.set_fb(
            self.fb,
            match state {
                FieldState::Empty => 0,
                FieldState::Snake(_) => SNAKE_BRIGHTNESS,
                FieldState::SnakeHead => HEAD_BRIGHTNESS,
                FieldState::Maultasch => MAULTASCH_BRIGHTNESS,
            },
        );
    }

    pub fn gen_maultasch(&mut self, rng: &mut Rng) {
        loop {
            let pos = FbCoordinate::random(rng);

            if self.get(pos) == FieldState::Empty {
                self.set(pos, FieldState::Maultasch);

                break;
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct GameResult {
    pub has_won: bool,
    pub score: u32,
}

#[derive(Debug)]
struct Game<'a> {
    rng: Rng,
    field: Field<'a>,
    dir: Direction,
    dir_change: i32,
    length: u8,
    length_increase: u8,
    score: u32,
    multiplier: u32,
    head: FbCoordinate,
    tail: FbCoordinate,
}

impl<'a> Game<'a> {
    pub fn new(fb: &'a Framebuffer) -> Self {
        let mut g = Self {
            rng: Rng::new(),
            field: Field::new(fb),
            dir: Direction::Right,
            dir_change: 0,
            length: 1,
            length_increase: 1,
            score: 0,
            multiplier: 1,
            head: FbCoordinate {
                x: 0,
                y: (Framebuffer::HEIGHT / 2) as u8,
            },
            tail: FbCoordinate {
                x: 0,
                y: (Framebuffer::HEIGHT / 2) as u8,
            },
        };

        g.field.set(g.head, FieldState::Snake(g.dir));
        g.field.gen_maultasch(&mut g.rng);

        g
    }

    pub fn advance(&mut self) -> Option<GameResult> {
        if self.retreat_tail() {
            Some(GameResult { has_won: true, score: self.score })
        } else if self.advance_head() {
            Some(GameResult { has_won: false, score: self.score })
        } else {
            None
        }
    }

    pub fn turn_cw(&mut self) {
        self.dir_change += 1;
    }

    pub fn turn_ccw(&mut self) {
        self.dir_change -= 1;
    }

    fn retreat_tail(&mut self) -> bool {
        if self.length_increase > 0 {
            self.length_increase -= 1;
            self.length += 1;
        } else {
            let dir = self.field.get(self.tail).unwrap_dir();
            self.field.set(self.tail, FieldState::Empty);
            self.tail = self.tail.go_dir(dir);
        }

        self.length >= (Framebuffer::WIDTH * Framebuffer::HEIGHT) as u8
    }

    fn advance_head(&mut self) -> bool {
        if self.dir_change > 0 {
            self.dir_change -= 1;
            self.dir = self.dir.cw();
        } else if self.dir_change < 0 {
            self.dir_change += 1;
            self.dir = self.dir.ccw();
        }

        self.field.set(self.head, FieldState::Snake(self.dir));
        self.head = self.head.go_dir(self.dir);

        match self.field.get(self.head) {
            FieldState::Empty => (),
            FieldState::Snake(_) | FieldState::SnakeHead => return true,
            FieldState::Maultasch => {
                self.score += if self.length > MULTIPLIER_MIN_LENGTH {
                    self.multiplier
                } else {
                    1
                };

                self.length_increase += 2;
                self.field.gen_maultasch(&mut self.rng);
            }
        }

        self.field.set(self.head, FieldState::SnakeHead);

        false
    }
}

pub async fn run() {
    let fb = Matrix::fb();
    let mut clock = Ticker::every(INITIAL_DURATION);
    let mut g = Game::new(fb);
    let mut paused = false;

    let res = loop {
        match select(Buttons::event(), clock.next()).await {
            Either::First(Event { pressed: true, button }) => match button {
                Button::Start => {
                    clock.reset();
                    paused ^= true;
                }
                Button::Select => {
                    if paused {
                        break GameResult { has_won: false, score: g.score };
                    } else {
                        let new_ticks = clock.duration().as_ticks() / 2;
                        if new_ticks >= matrix::FRAME_DURATION.as_ticks() {
                            g.multiplier <<= 1;

                            clock.set_duration(Duration::from_ticks(new_ticks));
                        }
                    }
                }
                Button::L => {
                    if !paused {
                        g.turn_ccw();
                    }
                }
                Button::R => {
                    if !paused {
                        g.turn_cw();
                    }
                }
            },
            Either::First(Event { pressed: false, button: _ }) => (),
            Either::Second(()) => {
                if !paused && let Some(r) = g.advance() {
                    break r;
                }
            }
        }
    };

    let hs = flash::new_high_score(super::Game::Snake.high_score_index(), res.score).await;

    super::show_score(res.has_won, res.score, hs).await;
}
