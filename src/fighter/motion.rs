//! Where a fighter is and how it moves. States write [`FighterVelocity`];
//! this module is what turns that into a position.

use bevy::prelude::*;
use fixed::types::I16F16;

use crate::{fighter::{FighterFacingDirection, baked_animation::FixedMat4}, math::{int::FGi32, vec::FGVec2, vec3::FGVec3}};

#[derive(Component, Clone, Copy)]
pub struct Grounded;

#[derive(Component, Deref, DerefMut, Default, Debug, Clone, Copy)]
pub struct FighterVelocity(pub FGVec2);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
pub struct FighterPreviousTranslation(pub FGVec2);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
#[require(FighterPreviousTranslation)]
pub struct FighterTranslation(pub FGVec2);

impl FighterTranslation {
    pub fn get_3d_transform(&self, facing_dir: &FighterFacingDirection) -> FixedMat4 {
        let mut o = match facing_dir {
            FighterFacingDirection::Left => FixedMat4::IDENTITY.rotate_y(3),
            FighterFacingDirection::Right => FixedMat4::IDENTITY.rotate_y(1),
        };
        o.translate(FGVec3::new(self.0.x, self.0.y, FGi32::ZERO));
        o
    }
}

pub fn apply_air_motion(
    fighters: Query<(
        &FighterVelocity,
        &mut FighterTranslation,
        &mut FighterPreviousTranslation,
        &mut Transform,
    )>,
    _time: Res<Time<Fixed>>,
) {
    for (velocity, mut translation, mut prev_translation, _transform) in fighters {
        prev_translation.0 = translation.0;
        translation.0 += velocity.0;
    }
}

pub fn copy_fighter_transform_to_visuals(fighters: Query<(&FighterTranslation, &mut Transform)>) {
    for (translation, mut transform) in fighters {
        transform.translation = Vec3::new(translation.x.to_num(), translation.y.to_num(), 0.0);
    }
}

pub fn fighter_movement() {}
