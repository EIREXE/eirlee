//! Resolving an integrated fighter position against the stage.

use bevy::{color::palettes::css::HOT_PINK, prelude::*};

use crate::fighter::{FighterECB, FighterTranslation, FighterVelocity};
use crate::stage::StageLine;

pub fn collide_fighter_with_scene(
    fighters: Query<(&FighterECB, &mut FighterVelocity, &mut FighterTranslation)>,
    stage_planes: Query<&StageLine>,
    mut gizmos: Gizmos,
) {
    for (ecb, mut velocity, mut translation) in fighters {
        for plane in stage_planes {
            let ray_from = translation.0;
            let ray_to = translation.0 + Vec2::new(0.0, -ecb.vertical_half);

            let intersection_result = plane.intersect_ray(Segment2d::new(ray_from, ray_to));

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
