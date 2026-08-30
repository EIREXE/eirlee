use std::collections::HashMap;

use bevy::{asset::Handle, ecs::component::Component, reflect::Reflect};
use serde::{Deserialize, Serialize};

use crate::{fighter::animation::AnimKind, math::{int::FGi32, vec3::FGVec3}, scripting::FighterAttackScript};
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FighterDamage(FGi32);

impl FighterDamage {
    pub fn lit(text: &str) -> Self {
        Self(FGi32::lit(text))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum KnockbackType {
    Normal,
    Fixed
}

impl KnockbackType {
    pub fn make_normal() -> Self {
        Self::Normal
    }

    pub fn make_fixed() -> Self {
        Self::Fixed
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Knockback(FGi32);

impl Knockback {
    pub fn lit(val: &str) -> Self {
        Self(FGi32::lit(val))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttackAngle {
    Normal(FGi32),
    Sakurai
}

impl Default for AttackAngle {
    fn default() -> Self {
        AttackAngle::Normal(FGi32::lit("0"))
    }
}

impl AttackAngle {

    pub fn make_normal(angle: FGi32) -> Self {
        Self::Normal(angle)
    }

    pub fn make_sakurai() -> Self {
        Self::Sakurai
    }
}

#[derive(Serialize, Deserialize, Reflect, PartialEq, Eq, Hash, Clone, Debug)]
pub enum AttackKind {
    NAir,
    Jab
}

impl AttackKind {
    pub fn get_animation(&self) -> AnimKind {
        match self {
            AttackKind::NAir => AnimKind::AttackJab1,
            AttackKind::Jab => AnimKind::AttackJab1,
        }
    }
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(opaque)]
pub struct AttackHitbox {
    pub id: u32,
    pub bone: String,
    pub damage: FighterDamage,
    pub offset: FGVec3,
    pub radius: FGi32,
    pub angle: AttackAngle,
    pub knockback_type: KnockbackType,
    pub knockback: Knockback,
    pub knockback_growth: FGi32,
    pub start_frame: u32,
    pub end_frame: u32
}

pub fn update_active_hitbox_list(script: &FighterAttackScript, list: &mut Vec<usize>, frame: u32) {
    *list = script.hitboxes.iter().enumerate().filter(|(_, hb)| {
        hb.start_frame <= frame && hb.end_frame > frame
    }).map(|(i, _)| i).collect()
}

#[derive(Component)]
pub struct FighterAttackScriptAssets {
    pub scripts: HashMap<AttackKind, Handle<FighterAttackScript>>
}