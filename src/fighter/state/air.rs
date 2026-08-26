use bevy::prelude::*;

use crate::fighter::*;

pub fn integrate_gravity(velocity: &mut FighterVelocity, attributes: &FighterAttributes) {
    velocity.y -= attributes.gravity;
    velocity.y = velocity.y.max(-attributes.terminal_velocity);
}

pub fn apply_air_motion(
    velocity: &FighterVelocity,
    translation: &mut FighterTranslation,
    prev_translation: &mut FighterPreviousTranslation,
) {
    // Vertical velocity should be 0 on the ground
    prev_translation.0 = translation.0;
    translation.0 += velocity.0;
}
