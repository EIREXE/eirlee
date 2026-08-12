use bevy::prelude::*;

use crate::input::FighterInputFrame;

/// Per-fighter tuning values. These are character data, not global rules —
/// anything that applies to every fighter belongs in
/// [`crate::game_settings::FighterSettingsCommon`] instead.
#[derive(Component, Clone, Copy)]
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

impl FighterAttributes {
    pub fn get_accel_and_target_dashrun(&self, input: &FighterInputFrame) -> (f32, f32) {
        let accel = input.movement.x * self.stick_dash_accel;
        let accel = accel + input.movement.x.signum() * self.base_dash_accel;
        let target_vel = input.movement.x * self.max_dash_vel;

        (accel, target_vel)
    }

    pub fn get_accel_and_target_walk(&self, input: &FighterInputFrame) -> (f32, f32) {
        let accel = input.movement.x * self.stick_walk_accel;
        let accel = accel + input.movement.x.signum() * self.base_walk_accel;
        let target_vel = input.movement.x * self.max_walk_vel;

        (accel, target_vel)
    }
}
