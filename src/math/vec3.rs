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
    pub fn lit(x: &str, y: &str, z: &str) -> Self {
        Self {
            x: FGi32::lit(x),
            y: FGi32::lit(y),
            z: FGi32::lit(z),
        }
    }
}