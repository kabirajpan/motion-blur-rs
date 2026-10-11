//! 2D Vector mathematics for optical flow velocity fields.

use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

/// 2D floating-point vector representing velocity (u, v) or coordinates (x, y).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    #[inline(always)]
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline(always)]
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    #[inline(always)]
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    #[inline(always)]
    pub fn dot(&self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    #[inline(always)]
    pub fn clamp_length(&self, max_length: f32) -> Self {
        let len_sq = self.length_squared();
        if len_sq > max_length * max_length && len_sq > 1e-7 {
            let scale = max_length / len_sq.sqrt();
            Self {
                x: self.x * scale,
                y: self.y * scale,
            }
        } else {
            *self
        }
    }
}

impl Add for Vec2 {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl AddAssign for Vec2 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vec2 {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        Self { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl SubAssign for Vec2 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: f32) -> Self {
        Self { x: self.x * rhs, y: self.y * rhs }
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;
    #[inline(always)]
    fn div(self, rhs: f32) -> Self {
        Self { x: self.x / rhs, y: self.y / rhs }
    }
}
