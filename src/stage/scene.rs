//! The scene content that gets spawned at startup.

use std::f32::consts::PI;

use bevy::{camera_controller::free_camera::FreeCamera, prelude::*};

use crate::{math::vec::FGVec2, stage::{StagePoly, line::{StageCollision, StagePolyLineSegmentType}}};

/// set up a simple 3D scene
pub fn test_scene() -> impl SceneList {

    let stage_poly = StagePoly::build(crate::stage::line::StagePolyType::Closed, &[
        (FGVec2::lit("-5.6", "-0.35"), StagePolyLineSegmentType::Floor),
        (FGVec2::lit("-3.92", "0.0"), StagePolyLineSegmentType::Floor),
        (FGVec2::lit("0.0", "0.0"), StagePolyLineSegmentType::Floor),
        (FGVec2::lit("3.92", "0.0"), StagePolyLineSegmentType::Floor),
        (FGVec2::lit("5.6", "-0.35"), StagePolyLineSegmentType::Wall),
        (FGVec2::lit("5.6", "-20.0"), StagePolyLineSegmentType::Ceiling),
        (FGVec2::lit("-5.6", "-20.0"), StagePolyLineSegmentType::Wall)
    ]);

    bsn_list! [
        (
            #StageCollision
            template_value(StageCollision {
                stage_polys: vec!(stage_poly)
            })
        ),
        (
            DirectionalLight {
                shadow_maps_enabled: true,
            }
            Transform {
                translation: Vec3::new(0.0, 2.0, 0.0),
                rotation: Quat::from_rotation_x(-PI / 4.),
            }
        ),
        (
            Camera3d
            template_value(Transform::from_xyz(0.0, 1.5, 9.0))
            FreeCamera {
                sensitivity: 0.2,
                friction: 25.0,
                walk_speed: 3.0,
                run_speed: 9.0,
            }
        )
    ]
}
