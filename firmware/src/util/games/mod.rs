pub mod snake;

use crate::drivers::matrix::{Framebuffer, Matrix};
use core::ops::Add;

const MAX_VEC: Vec2d = Vec2d {
    x: Framebuffer::WIDTH as i8,
    y: Framebuffer::HEIGHT as i8,
};

#[derive(Clone, Copy, PartialEq)]
pub struct Vec2d {
    x: i8,
    y: i8,
}

impl Vec2d {
    pub fn from_xy(x: i8, y: i8) -> Self {
        Self { x, y }
    }

    fn wrap(&mut self, max: Self) {
        if self.x >= 0 {
            self.x %= max.x;
        } else {
            while self.x < 0 {
                self.x += max.x;
            }
        }

        if self.y >= 0 {
            self.y %= max.y;
        } else {
            while self.y < 0 {
                self.y += max.y;
            }
        }
    }

    pub fn wrapping_add<T>(mut self, rhs: T, max: Self) -> Self
    where
        Self: Add<T, Output = Self>,
    {
        self = self + rhs;
        self.wrap(max);
        self
    }
}

impl Add for Vec2d {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
