//! Where a fighter is and how it moves. States write [`FighterVelocity`];
//! this module is what turns that into a position.

use bevy::prelude::*;

use crate::fighter::FighterAttributes;

#[derive(Component, Clone, Copy)]
pub struct Grounded;

#[derive(Component, Deref, DerefMut, Default, Debug, Clone, Copy)]
pub struct FighterVelocity(pub Vec2);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
pub struct FighterPreviousTranslation(pub Vec2);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
#[require(FighterPreviousTranslation)]
pub struct FighterTranslation(pub Vec2);

pub fn apply_air_motion(
    fighters: Query<(
        &FighterVelocity,
        &mut FighterTranslation,
        &mut FighterPreviousTranslation,
        &mut Transform,
    )>,
    _time: Res<Time<Fixed>>,
) {
    for (velocity, mut translation, mut prev_translation, mut transform) in fighters {
        prev_translation.0 = translation.0;
        translation.0 += velocity.0;
    }
}

pub fn copy_fighter_transform_to_visuals(
    fighters: Query<(&FighterTranslation, &mut Transform)>,
) {
    for (translation, mut transform) in fighters {
        transform.translation = Vec3::new(translation.x, translation.y, 0.0);
    }
}

pub fn fighter_movement() {}
