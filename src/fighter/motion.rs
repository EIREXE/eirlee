//! Where a fighter is and how it moves. States write [`FighterVelocity`];
//! this module is what turns that into a position.

use bevy::prelude::*;
use bevy_ggrs::GgrsFrameTiming;

use crate::debug_tools::PresentationMotionMode;
use crate::{
    fighter::{FighterECB, FighterFacingDirection, baked_animation::FixedMat4},
    math::{int::FGi32, vec::FGVec2, vec3::FGVec3},
};

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

/// Samples deterministic fighter positions for presentation without changing
/// gameplay state or rollback timing.
pub fn sample_presentation_translation(
    previous: FighterPreviousTranslation,
    current: FighterTranslation,
    overstep_fraction: f32,
    mode: PresentationMotionMode,
) -> Vec2 {
    let previous = Vec2::new(previous.x.to_num(), previous.y.to_num());
    let current = Vec2::new(current.x.to_num(), current.y.to_num());
    match mode {
        PresentationMotionMode::Interpolation => previous.lerp(current, overstep_fraction),
        PresentationMotionMode::Extrapolation => current + (current - previous) * overstep_fraction,
    }
}

pub fn apply_fighter_translation_to_visuals(
    query: Query<(
        &FighterTranslation,
        &FighterPreviousTranslation,
        &FighterECB,
        &mut Transform,
    )>,
    timing: Res<GgrsFrameTiming>,
    debug_settings: Res<crate::debug_tools::DebugSettings>,
) {
    let overstep_fraction = timing.overstep_fraction();
    for (translation, previous_translation, ecb, mut transform) in query {
        let translation = sample_presentation_translation(
            *previous_translation,
            *translation,
            overstep_fraction,
            debug_settings.motion_sampling,
        );
        let bottom = ecb.get_bottom_point();
        transform.translation = Vec3::new(
            translation.x + bottom.x.to_num::<f32>(),
            translation.y + bottom.y.to_num::<f32>(),
            0.0,
        );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presentation_translation_interpolates_between_fixed_samples() {
        let position = sample_presentation_translation(
            FighterPreviousTranslation(FGVec2::new(FGi32::lit("2"), FGi32::lit("4"))),
            FighterTranslation(FGVec2::new(FGi32::lit("6"), FGi32::lit("12"))),
            0.25,
            PresentationMotionMode::Interpolation,
        );

        assert_eq!(position, Vec2::new(3.0, 6.0));
    }

    #[test]
    fn presentation_translation_extrapolates_from_current_sample() {
        let position = sample_presentation_translation(
            FighterPreviousTranslation(FGVec2::new(FGi32::lit("2"), FGi32::lit("4"))),
            FighterTranslation(FGVec2::new(FGi32::lit("6"), FGi32::lit("12"))),
            0.25,
            PresentationMotionMode::Extrapolation,
        );

        assert_eq!(position, Vec2::new(7.0, 14.0));
    }
}
