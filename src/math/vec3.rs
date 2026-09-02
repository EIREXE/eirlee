use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::math::int::FGi32;

#[derive(Clone, Copy, PartialEq, Default, Debug, Reflect, Serialize, Deserialize, Hash)]
#[reflect(opaque)]
pub struct FGVec3 {
    pub x: FGi32,
    pub y: FGi32,
    pub z: FGi32,
}

impl FGVec3 {

    pub fn new(x: FGi32, y: FGi32, z: FGi32) -> Self {
        Self {
            x,
            y,
            z
        }
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
}

impl From<(&str, &str, &str)> for FGVec3 {
    fn from((x, y, z): (&str, &str, &str)) -> Self {
        Self::lit(x, y, z)
    }
}