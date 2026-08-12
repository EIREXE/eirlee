//! The scene content that gets spawned at startup.

use bevy::prelude::*;

use crate::stage::StageLine;

/// set up a simple 3D scene
pub fn test_scene() -> impl SceneList {
    bsn_list! [
        (
            #Cube
            Mesh3d(asset_value(Cuboid::new(5.0, 1.0, 1.0)))
            MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(124, 144, 255)))
            Transform::from_xyz(0.0, 0.5, 0.0)
        ),
        (
            #StagePlane
            template_value(StageLine::new(Vec2::new(-2.5, 1.0), Vec2::new(2.5, 1.0)).unwrap())
        ),
        (
            PointLight {
                shadow_maps_enabled: true,
            }
            Transform::from_xyz(4.0, 8.0, 4.0)
        ),
        (
            Camera3d
            template_value(Transform::from_xyz(0.0, 1.5, 9.0))
            /*FreeCamera {
                sensitivity: 0.2,
                friction: 25.0,
                walk_speed: 3.0,
                run_speed: 9.0,
            }*/
        )
    ]
}
