use crate::drivers::buttons::{Button, Buttons};
use crate::drivers::matrix::{Framebuffer, Matrix};
use crate::util::games::{MAX_VEC, Vec2d};
use crate::util::rand;
use core::ops::Add;
use embassy_time::Timer;

#[derive(Clone, Copy)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Add<Direction> for Vec2d {
    type Output = Self;

    fn add(mut self, rhs: Direction) -> Self::Output {
        match rhs {
            Direction::Left => self.x -= 1,
            Direction::Right => self.x += 1,
            Direction::Up => self.y -= 1,
            Direction::Down => self.y += 1,
        }
        self
    }
}

impl Vec2d {
    fn output_brightness(&self, brightness: u8) {
        Matrix::fb().store(self.x as usize, self.y as usize, brightness);
    }
}

struct Snake {
    body: [Vec2d; Framebuffer::HEIGHT * Framebuffer::WIDTH],
    len: usize,
    direction: Direction,
}

impl Snake {
    fn new() -> Self {
        let body = [Vec2d::from_xy(0, Framebuffer::HEIGHT as i8 / 2);
            Framebuffer::HEIGHT * Framebuffer::WIDTH];
        let len = 2;

        let direction = Direction::Right;
        Self {
            body,
            len,
            direction,
        }
    }

    fn eat_mauldasch(&mut self, mauldäschle: &mut Mauldasch) {
        for i in 0..self.len {
            self.body[i + 1] = self.body[i];
        }
        self.len += 1;
        self.body[0] = mauldäschle.pos.clone();
        mauldäschle.generate_new(self);
    }

    fn go_forward(&mut self, mauldäschle: &mut Mauldasch) -> GameState {
        if self.body[0..self.len].contains(&(self.body[0].wrapping_add(self.direction, MAX_VEC))) {
            if self.len == Framebuffer::HEIGHT * Framebuffer::WIDTH {
                GameState::Won
            } else {
                GameState::Lost
            }
        } else if self.body[0].wrapping_add(self.direction, MAX_VEC) == mauldäschle.pos {
            self.eat_mauldasch(mauldäschle);
            GameState::Playing {
                increment_score: true,
            }
        } else {
            for i in (0..self.len).rev() {
                self.body[i + 1] = self.body[i];
            }
            self.body[0] = self.body[0].wrapping_add(self.direction, MAX_VEC);
            GameState::Playing {
                increment_score: false,
            }
        }
    }

    fn change_direction(&mut self, dir_ch: Direction) {
        self.direction = match dir_ch {
            Direction::Right => match self.direction {
                Direction::Up => Direction::Right,
                Direction::Right => Direction::Down,
                Direction::Down => Direction::Left,
                Direction::Left => Direction::Up,
            },
            Direction::Left => match self.direction {
                Direction::Up => Direction::Left,
                Direction::Right => Direction::Up,
                Direction::Down => Direction::Right,
                Direction::Left => Direction::Down,
            },
            _ => unimplemented!(),
        }
    }

    fn output_brightness(&self, brightness: u8) {
        for i in 0..self.len {
            self.body[i].output_brightness(brightness);
        }
    }
}

struct Mauldasch {
    pos: Vec2d,
    noisegen: rand::WhiteNoiseGenerator,
}

impl Mauldasch {
    fn new(snake: &Snake) -> Self {
        let mut noisegen = rand::WhiteNoiseGenerator::new();
        let pos = Vec2d::from_xy(0, 0);
        let mut mauldäschle = Self { noisegen, pos };
        mauldäschle.generate_new(snake);
        mauldäschle
    }

    fn generate_new(&mut self, snake: &Snake) {
        while {
            self.pos = Vec2d::from_xy(
                (self.noisegen.rand8() % Framebuffer::WIDTH as u8) as i8,
                (self.noisegen.rand8() % Framebuffer::HEIGHT as u8) as i8,
            );
            snake.body[0..snake.len].contains(&self.pos)
        } {}
    }

    fn output_brightness(&self, brightness: u8) {
        self.pos.output_brightness(brightness)
    }
}

enum GameState {
    Playing { increment_score: bool },
    Won,
    Lost,
}

pub struct SnakeGame {
    score: u8,
    state: GameState,
    snake: Snake,
    mauldäschle: Mauldasch,
}

impl SnakeGame {
    pub fn new() -> Self {
        let score = 0;
        let state = GameState::Playing {
            increment_score: false,
        };
        let snake = Snake::new();
        let mauldäschle = Mauldasch::new(&snake);
        Self {
            score,
            state,
            snake,
            mauldäschle,
        }
    }

    pub async fn run(mut self) {
        while let GameState::Playing { increment_score } = self.state {
            Matrix::fb().store_all(0);
            if increment_score {
                self.score += 1;
            }
            self.snake.output_brightness(128);
            self.mauldäschle.output_brightness(255);
            self.state = self.snake.go_forward(&mut self.mauldäschle);

            // TODO: how do I get a pressed button without blocking the whole program?
            let buttons = Buttons::event().await;
            let l_pressed = (buttons.button as usize == 2 && buttons.pressed);
            let r_pressed = (buttons.button as usize == 1 && buttons.pressed);

            if l_pressed && !r_pressed {
                self.snake.change_direction(Direction::Left);
            } else if r_pressed && !l_pressed {
                self.snake.change_direction(Direction::Right);
            }

            Timer::after_millis(1000).await;
        }
        Matrix::fb().store_all(0);
    }
}
