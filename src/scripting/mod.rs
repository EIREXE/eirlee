use bevy::ecs::name::HashedStr;

use crate::math::int::FGi32;

pub mod move_compiler;

pub enum MoveAngleKind {
    Normal(FGi32),
    Sakurai
}

pub struct MoveEventHurtbox {
    pub priority: u32,
    pub bone: HashedStr,
    pub radius: FGi32,
    pub damage: FGi32,
    pub knockback_scaling: FGi32,
    pub angle: MoveAngleKind,
}

pub enum MoveEvent {
    Hurtbox(MoveEventHurtbox)
}