use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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

impl FighterAttackScript {
    pub fn validate(&self) -> Result<(), String> {
        let mut ids = HashSet::with_capacity(self.hitboxes.len());
        for hitbox in &self.hitboxes {
            hitbox.validate()?;
            if !ids.insert(hitbox.id) {
                return Err(format!("duplicate hitbox id {}", hitbox.id));
            }
        }
        if self.wont_autocancel_window.0 > self.wont_autocancel_window.1 {
            return Err("autocancel window end precedes its start".into());
        }
        Ok(())
    }
}
