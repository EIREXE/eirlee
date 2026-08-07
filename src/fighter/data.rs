use bevy::prelude::*;

#[derive(Component)]
pub struct FighterAttributes {
    pub base_walk_accel: f32,
    pub stick_walk_accel: f32,
    pub max_walk_vel: f32,

    pub dash_duration: u32,
    pub dash_initial_velocity: f32,
    pub base_dash_accel: f32,
    pub stick_dash_accel: f32,
    pub max_dash_vel: f32,

    pub ground_friction: f32,
}