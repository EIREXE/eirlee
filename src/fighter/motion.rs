//! Where a fighter is and how it moves. States write [`FighterVelocity`];
//! this module is what turns that into a position.

use bevy::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct Grounded;

#[derive(Component, Deref, DerefMut, Default, Debug, Clone, Copy)]
pub struct FighterVelocity(pub Vec2);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
pub struct FighterPreviousTranslation(pub Vec2);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
#[require(FighterPreviousTranslation)]
pub struct FighterTranslation(pub Vec2);

pub fn integrate_gravity(_fighters: Query<&mut FighterVelocity>, _time: Res<Time<Fixed>>) {}

pub fn apply_motion(
    fighters: Query<(
        &FighterVelocity,
        &mut FighterTranslation,
        &mut FighterPreviousTranslation,
    )>,
    _time: Res<Time<Fixed>>,
) {
    for (velocity, mut translation, mut prev_translation) in fighters {
        prev_translation.0 = translation.0;
        translation.0 += velocity.0;
    }
}

pub fn fighter_movement() {}
