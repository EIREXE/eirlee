use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{fighter::attack::AttackHitbox, math::int::FGi32};

pub mod importer;
pub mod move_compiler;

pub enum MoveAngleKind {
    Normal(FGi32),
    Sakurai,
}

#[derive(Asset, Reflect, Serialize, Deserialize)]
pub struct FighterAttackScript {
    pub hitboxes: Vec<AttackHitbox>,
    pub iasa_frame: u32,
    pub wont_autocancel_window: (u32, u32),
}
