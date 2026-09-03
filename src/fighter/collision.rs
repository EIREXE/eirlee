//! Resolving an integrated fighter position against the stage.

use crate::fighter::state::FighterStateContext;
use crate::math::segment::FGSegment2d;
use crate::math::vec::FGVec2;
use crate::stage::line::StageLineID;
/*
pub fn collide_fighter_with_scene(
    fighters: Query<(&FighterECB, &mut FighterVelocity, &mut FighterTranslation)>,
    stage_planes: Query<&StagePoly>,
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
*/
#[derive(Clone)]
pub struct AirCollisionWithStageResult {
    pub line_id: StageLineID,
    pub hit_position: FGVec2,
    pub hit_normal: FGVec2,
}

pub fn air_collide_with_stage(
    state_context: &FighterStateContext,
) -> Option<AirCollisionWithStageResult> {
    let ray_segment = FGSegment2d::new(
        state_context.prev_translation.0 + state_context.prev_ecb.get_bottom_point(),
        state_context.translation.0 + state_context.ecb.get_bottom_point(),
    );

    let mut intersect_results = vec![];

    for (poly_idx, poly) in state_context.stage_collision.stage_polys.iter().enumerate() {
        if let Some(intersect_result) = poly.intersect_ray(ray_segment) {
            intersect_results.push((AirCollisionWithStageResult {
                line_id: {
                    StageLineID {
                        polygon: poly_idx,
                        segment: intersect_result.segment_idx,
                    }
                },
                hit_position: intersect_result.position,
                hit_normal: intersect_result.normal,
            }, intersect_result.position.distance_squared(ray_segment.point2())));
        }
    }
    intersect_results.sort_by(| (_, a), (_, b) | {
        a.cmp(b)
    });

    intersect_results.iter().last().map(|(val, _)| {
        val.to_owned().clone()
    })
}
