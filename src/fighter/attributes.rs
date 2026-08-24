use bevy::prelude::*;

use crate::{input::FighterInputFrame, math::int::FGi32};

/// Per-fighter tuning values. These are character data, not global rules —
/// anything that applies to every fighter belongs in
/// [`crate::game_settings::FighterSettingsCommon`] instead.
#[derive(Component, Clone, Copy)]
pub struct FighterAttributes {
    pub base_walk_accel: FGi32,
    pub stick_walk_accel: FGi32,
    pub max_walk_vel: FGi32,

    pub dash_duration: u32,
    pub dash_initial_velocity: FGi32,
    pub base_dash_accel: FGi32,
    pub stick_dash_accel: FGi32,
    pub max_dash_vel: FGi32,

    pub jumpsquat_duration: u32,
    pub jump_vertical_velocity: FGi32,

    pub ground_friction: FGi32,
    pub terminal_velocity: FGi32,
    pub gravity: FGi32,
}

impl FighterAttributes {
    pub fn get_accel_and_target_dashrun(&self, input: &FighterInputFrame) -> (FGi32, FGi32) {
        let accel = input.movement.x * self.stick_dash_accel;
        let accel = accel + input.movement.x.signum() * self.base_dash_accel;
        let target_vel = input.movement.x * self.max_dash_vel;

        (accel, target_vel)
    }

    pub fn get_accel_and_target_walk(&self, input: &FighterInputFrame) -> (FGi32, FGi32) {
        let accel = input.movement.x * self.stick_walk_accel;
        let accel = accel + input.movement.x.signum() * self.base_walk_accel;
        let target_vel = input.movement.x * self.max_walk_vel;

        (accel, target_vel)
    }
}
