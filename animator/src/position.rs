use std::ops::{Add, Mul};

use crate::to_float::ToFloat;

/// A position with an x- and a y-value
#[derive(Default, Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl From<Position> for naviz_state::Position {
    fn from(value: Position) -> Self {
        (value.x, value.y)
    }
}

impl From<naviz_parser::config::position::Position> for Position {
    fn from(value: naviz_parser::config::position::Position) -> Self {
        let x = value.0.f32();
        let y = value.1.f32();
        Self { x, y }
    }
}

impl Mul<f32> for Position {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Add<Position> for Position {
    type Output = Self;
    fn add(self, rhs: Position) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
