use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{input::FighterInputFrame, math::int::FGi32};

/// Per-fighter tuning values. These are character data, not global rules —
/// anything that applies to every fighter belongs in
/// [`crate::game_settings::FighterSettingsCommon`] instead.
#[derive(Asset, Component, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(opaque)]
pub struct FighterAttributes {
    pub base_walk_accel: FGi32,
    pub stick_walk_accel: FGi32,
    pub max_walk_vel: FGi32,

    pub dash_acceleration_duration: u32,
    pub dash_duration: u32,
    pub dash_initial_velocity: FGi32,
    pub base_dash_accel: FGi32,
    pub stick_dash_accel: FGi32,
    pub max_dash_vel: FGi32,

    pub jumpsquat_duration: u32,
    pub short_hop_vertical_velocity: FGi32,
    pub full_jump_vertical_velocity: FGi32,
    pub jump_horizontal_velocity: FGi32,
    pub landing_iasa: u32,
    pub landing_duration: u32,

    pub air_acceleration_base: FGi32,
    pub air_acceleration_stick: FGi32,
    pub max_air_horizontal_velocity: FGi32,

    pub ground_friction: FGi32,
    pub terminal_velocity: FGi32,
    pub gravity: FGi32,
    pub air_friction: FGi32,

    pub air_dodge_duration: u32,
    pub air_dodge_velocity: FGi32,
    pub air_dodge_decay: FGi32
}

impl FighterAttributes {
    pub fn get_accel_and_target_dashrun(
        &self,
        input: &FighterInputFrame,
        direction: FGi32,
    ) -> (FGi32, FGi32) {
        let accel = input.movement.x * self.stick_dash_accel;
        let accel = accel + direction * self.base_dash_accel;
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

pub struct FighterAttributesLink {
    attributes: Handle<FighterAttributes>,
}
