use bevy::{
    color::palettes::css::{HOT_PINK, YELLOW},
    prelude::*,
};

pub mod plugin;
pub mod data;
pub mod state;
pub mod states;
pub use plugin::*;
pub use data::*;

use crate::stage::StagePlane;

#[derive(Component)]
pub struct Grounded;

#[derive(Component, Deref, DerefMut, Default, Debug)]
pub struct FighterVelocity(pub Vec2);

#[derive(Component, Deref, DerefMut, Default)]
pub struct FighterPreviousTranslation(pub Vec2);

#[derive(Component, Deref, DerefMut, Default)]
#[require(FighterPreviousTranslation)]
pub struct FighterTranslation(pub Vec2);

#[derive(Component, Debug)]
#[require(Transform)]
pub struct FighterECB {
    pub vertical_half: f32,
    pub horizontal_half: f32,
}

impl FighterECB {
    pub fn to_3d_lineloop(&self, trf: Transform) -> [Vec3; 4] {
        let rel_up = trf.transform_point(Vec3::new(0.0, self.vertical_half, 0.0));
        let rel_down = trf.transform_point(Vec3::new(0.0, -self.vertical_half, 0.0));
        let rel_left = trf.transform_point(Vec3::new(-self.horizontal_half, 0.0, 0.0));
        let rel_right = trf.transform_point(Vec3::new(self.horizontal_half, 0.0, 0.0));
        [rel_up, rel_right, rel_down, rel_left]
    }
}

pub fn debug_draw_ecb(ecbs: Query<(&FighterECB, &FighterTranslation)>, mut gizmos: Gizmos) {
    for (ecb, trf) in ecbs {
        let transformed = ecb.to_3d_lineloop(Transform::from_xyz(trf.x, trf.y, 0.0));
        gizmos.lineloop(transformed, YELLOW);
    }
}

pub fn integrate_gravity(
    _fighters: Query<&mut FighterVelocity>,
    _time: Res<Time<Fixed>>,
) {
}

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

pub fn collide_fighter_with_scene(
    fighters: Query<(&FighterECB, &mut FighterVelocity, &mut FighterTranslation)>,
    stage_planes: Query<&StagePlane>,
    mut gizmos: Gizmos,
) {
    for (ecb, mut velocity, mut translation) in fighters {
        for plane in stage_planes {
            let dist_sq = ecb.vertical_half * ecb.vertical_half;
            let ray = Ray2d::new(translation.0, Dir2::NEG_Y);

            let intersection_result = plane
                .intersect_ray(ray)
                .filter(|result| result.position.distance_squared(translation.0) <= dist_sq);

            if let Some(result) = intersection_result {
                gizmos.sphere(
                    Isometry3d::from(Vec3::new(result.position.x, result.position.y, 0.0)),
                    0.1f32,
                    HOT_PINK,
                );
                velocity.0 = plane.project(velocity.0);
                translation.0 = result.position + Vec2::new(0.0, ecb.vertical_half);
            }
        }
    }
}

pub fn fighter_movement() {}

