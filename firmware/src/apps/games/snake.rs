use crate::drivers::matrix::{Framebuffer, Matrix};

const HEAD_BRIGHTNESS: u8 = 64;
const SNAKE_BRIGHTNESS: u8 = 32;
const MAULTASCH_BRIGHTNESS: u8 = 96;

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
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum FieldState {
    Empty,
    Snake(Direction),
    Maultasch,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct FbCoordinate {
    pub x: u8,
    pub y: u8,
}

impl FbCoordinate {
    pub fn set_fb(&self, fb: &Framebuffer, val: u8) {
        fb.store(self.x as usize, self.y as usize, val);
    }
}

#[derive(Debug)]
struct Field<'a> {
    field: [[FieldState; Framebuffer::WIDTH]; Framebuffer::HEIGHT],
    fb: &'a Framebuffer,
}

impl<'a> Field<'a> {
    fn new(fb: &'a Framebuffer) -> Self {
        fb.clear_all();
        Self {
            field: [[FieldState::Empty; Framebuffer::WIDTH]; Framebuffer::HEIGHT],
            fb,
        }
    }
    fn get(&self, c: FbCoordinate) -> FieldState {
        self.field[c.y as usize][c.x as usize]
    }

    fn set(&mut self, c: FbCoordinate, state: FieldState) {
        self.field[c.y as usize][c.x as usize] = state;
        c.set_fb(
            self.fb,
            match state {
                FieldState::Empty => 0,
                FieldState::Snake(_) => HEAD_BRIGHTNESS,
                FieldState::Maultasch => MAULTASCH_BRIGHTNESS,
            },
        );
    }

    fn gen_maultasch(&mut self) {}
}

#[derive(Debug)]
struct Game<'a> {
    field: Field<'a>,
    dir: Direction,
    dir_change: i32,
    length: u8,
    length_increase: u8,
    score: u8,
    head: FbCoordinate,
    tail: FbCoordinate,
}

impl<'a> Game<'a> {
    fn new(fb: &'a Framebuffer) -> Self {
        let mut g = Game {
            field: Field::new(fb),
            dir: Direction::Right,
            dir_change: 0,
            length: 1,
            length_increase: 1,
            score: 0,
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
        g.field.gen_maultasch();

        g
    }
}

pub async fn run() {
    let fb = Matrix::fb();
    let mut g = Game::new(&fb);
}
