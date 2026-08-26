//! Code-defined deterministic stage collision and development lighting.

use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{
    math::vec::FGVec2,
    stage::{
        StagePoly,
        line::{StageCollision, StagePolyLineSegmentType, StagePolyType},
    },
};

pub fn spawn_stage_support(commands: &mut Commands, match_root: Entity) {
    let stage_poly = StagePoly::build(
        StagePolyType::Closed,
        &[
            (
                FGVec2::lit("-56.0", "-3.5"),
                StagePolyLineSegmentType::Floor,
            ),
            (FGVec2::lit("-39.2", "0.0"), StagePolyLineSegmentType::Floor),
            (FGVec2::lit("0.0", "0.0"), StagePolyLineSegmentType::Floor),
            (FGVec2::lit("39.2", "0.0"), StagePolyLineSegmentType::Floor),
            (FGVec2::lit("56.0", "-3.5"), StagePolyLineSegmentType::Wall),
            (
                FGVec2::lit("56.0", "-200.0"),
                StagePolyLineSegmentType::Ceiling,
            ),
            (FGVec2::lit("-56", "-200.0"), StagePolyLineSegmentType::Wall),
        ],
    );

    commands.insert_resource(StageCollision {
        stage_polys: vec![stage_poly.clone()],
    });
    commands.spawn((
        Name::new("Stage Collision"),
        stage_poly,
        ChildOf(match_root),
    ));
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
