use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul, Neg, Sub};

use crate::math::int::{FGWide, FGi32};

#[derive(Clone, Copy, PartialEq, Default, Debug, Reflect, Serialize, Deserialize, Hash)]
#[reflect(opaque)]
pub struct FGVec3 {
    pub x: FGi32,
    pub y: FGi32,
    pub z: FGi32,
}

impl FGVec3 {
    pub const ZERO: Self = Self {
        x: FGi32::ZERO,
        y: FGi32::ZERO,
        z: FGi32::ZERO,
    };

    pub const fn new(x: FGi32, y: FGi32, z: FGi32) -> Self {
        Self { x, y, z }
    }

    pub fn lit(x: &str, y: &str, z: &str) -> Self {
        Self {
            x: FGi32::lit(x),
            y: FGi32::lit(y),
            z: FGi32::lit(z),
        }
    }

    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x.to_num(), self.y.to_num(), self.z.to_num())
    }

    pub fn dot(self, rhs: Self) -> FGWide {
        FGWide::from_num(self.x) * FGWide::from_num(rhs.x)
            + FGWide::from_num(self.y) * FGWide::from_num(rhs.y)
            + FGWide::from_num(self.z) * FGWide::from_num(rhs.z)
    }

    pub fn length_squared(self) -> FGWide {
        self.dot(self)
    }

    pub fn normalized(self) -> Option<Self> {
        let length_squared = self.length_squared();
        if length_squared == FGWide::ZERO {
            return None;
        }
        let length = length_squared.sqrt();
        Some(Self::new(
            FGi32::from_num(FGWide::from_num(self.x) / length),
            FGi32::from_num(FGWide::from_num(self.y) / length),
            FGi32::from_num(FGWide::from_num(self.z) / length),
        ))
    }
}

impl Add for FGVec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for FGVec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Neg for FGVec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl Mul<FGi32> for FGVec3 {
    type Output = Self;

    fn mul(self, rhs: FGi32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl From<(&str, &str, &str)> for FGVec3 {
    fn from((x, y, z): (&str, &str, &str)) -> Self {
        Self::lit(x, y, z)
    }
}
