use bevy::prelude::*;

#[derive(Component)]
pub struct FighterAttributes {
    pub base_walk_accel: f32,
    pub stick_walk_accel: f32,
    pub max_walk_vel: f32,
    pub ground_friction: f32,
    pub dash_duration: u32
}