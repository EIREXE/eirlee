use bevy::math::Vec2;
use std::ops::{Add, Div, Mul, Sub};

use super::int::FGi32;

#[derive(Clone, Copy, PartialEq)]
pub struct FGVec2 {
    pub x: FGi32,
    pub y: FGi32,
}

impl FGVec2 {
    pub const ZERO: Self = Self::new(FGi32::ZERO, FGi32::ZERO);

    #[inline(always)]
    pub const fn new(x: FGi32, y: FGi32) -> Self {
        Self { x, y }
    }

    #[inline(always)]
    pub fn from_i32(x: i32, y: i32) -> Self {
        Self {
            x: FGi32::from_num(x),
            y: FGi32::from_num(y),
        }
    }

    #[inline]
    pub fn length_squared(&self) -> FGi32 {
        self.x * self.x + self.y * self.y
    }

    #[inline]
    pub fn length(&self) -> FGi32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    #[inline]
    pub fn normalized_or_zero(&self) -> Self {
        let len = self.length();
        if len.is_zero() {
            Self::ZERO
        } else {
            self / len
        }
    }

    #[inline]
    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.x.to_num(), self.y.to_num())
    }
}

impl Mul for FGVec2 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl Add for FGVec2 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for FGVec2 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Div for FGVec2 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl Mul<FGi32> for FGVec2 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: FGi32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<FGi32> for FGVec2 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: FGi32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl Div<FGi32> for &FGVec2 {
    type Output = FGVec2;

    #[inline]
    fn div(self, rhs: FGi32) -> Self::Output {
        (*self).div(rhs)
    }
}
