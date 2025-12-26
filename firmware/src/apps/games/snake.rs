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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum FieldState {
    Empty,
    Snake(Direction),
    Maultasch,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Coordinate {
    x: usize,
    y: usize,
}

impl Coordinate {
    fn set_fb(&self, fb: &Framebuffer, val: u8) {
        fb.store(self.x, self.y, val);
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Field([[FieldState; Framebuffer::WIDTH]; Framebuffer::HEIGHT]);

impl Field {
    fn get(&self, c: Coordinate) -> FieldState {
        self.0[c.y][c.x]
    }

    fn set(&mut self, c: Coordinate, state: FieldState) {
        self.0[c.y][c.x] = state;
        Matrix::fb().store(
            c.x,
            c.y,
            match state {
                FieldState::Empty => 0,
                FieldState::Snake(_) => HEAD_BRIGHTNESS,
                FieldState::Maultasch => MAULTASCH_BRIGHTNESS,
            },
        );
    }

    fn gen_maultasch(&mut self) {}
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Game {
    field: Field,
    dir: Direction,
    dir_change: i32,
    length: u8,
    length_increase: u8,
    score: u8,
    head: Coordinate,
    tail: Coordinate,
}

impl Game {
    fn new(fb: &Framebuffer) -> Game {
        let mut g = Game {
            field: Field([[FieldState::Empty; Framebuffer::WIDTH]; Framebuffer::HEIGHT]),
            dir: Direction::Right,
            dir_change: 0,
            length: 1,
            length_increase: 1,
            score: 0,
            head: Coordinate {
                x: 0,
                y: Framebuffer::HEIGHT / 2,
            },
            tail: Coordinate {
                x: 0,
                y: Framebuffer::HEIGHT / 2,
            },
        };

        fb.clear_all();
        g.head.set_fb(fb, HEAD_BRIGHTNESS);
        g.field.set(g.head, FieldState::Snake(g.dir));
        g.field.gen_maultasch();

        g
    }
}

pub async fn run() {
    let fb = Matrix::fb();
    let mut g = Game::new(fb);
}
