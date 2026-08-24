//! The scene content that gets spawned at startup.

use std::f32::consts::PI;

use bevy::{camera_controller::free_camera::FreeCamera, prelude::*};

use crate::stage::{StagePoly, line::StagePolyLineSegmentType, line::StageCollision};

/// set up a simple 3D scene
pub fn test_scene() -> impl SceneList {

    let stage_poly = StagePoly::build(crate::stage::line::StagePolyType::Closed, &[
        (Vec2::new(-5.6, -0.35), StagePolyLineSegmentType::Floor),
        (Vec2::new(-3.92, 0.0), StagePolyLineSegmentType::Floor),
        (Vec2::new(0.0, 0.0), StagePolyLineSegmentType::Floor),
        (Vec2::new(3.92, 0.0), StagePolyLineSegmentType::Floor),
        (Vec2::new(5.6, -0.35), StagePolyLineSegmentType::Wall),
        (Vec2::new(5.6, -20.0), StagePolyLineSegmentType::Ceiling),
        (Vec2::new(-5.6, -20.0), StagePolyLineSegmentType::Wall)
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
