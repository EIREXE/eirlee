//! Code-defined deterministic stage collision and development lighting.

use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{camera::MatchCamera, stage::line::StageCollision};

pub fn spawn_stage_support(
    commands: &mut Commands,
    match_root: Entity,
    collision: StageCollision,
    camera_profile: crate::stage::manifest::StageCameraProfile,
) {
    commands.insert_resource(collision);
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.0),
            ..default()
        },
        ChildOf(match_root),
    ));
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: camera_profile.vertical_fov_degrees.to_radians(),
            ..default()
        }),
        MatchCamera::default(),
        camera_profile,
        Transform::from_xyz(0.0, 10.5, 150.0),
        /*FreeCamera {
            sensitivity: 0.2,
            friction: 25.0,
            walk_speed: 3.0,
            run_speed: 9.0,
            ..default()
        },*/
        ChildOf(match_root),
    ));
}
