//! The stage: its collision geometry, the scene that gets spawned, and how
//! both are drawn.

use bevy::prelude::*;

pub mod debug;
pub mod line;
pub mod scene;
pub mod manifest;

pub use line::{StagePlaneIntersectResult, StagePoly};

use crate::stage::line::StageCollision;

/// Owns the stage geometry: which scene gets spawned and how it is drawn.
#[derive(Default)]
pub struct StagePlugin;

impl Plugin for StagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            debug::debug_draw_scene.run_if(resource_exists::<line::StageCollision>),
        );
    }
}

pub fn spawn_stage(commands: &mut Commands, match_root: Entity, visual: Handle<WorldAsset>, collision: StageCollision) {
    commands.entity(match_root).insert(WorldAssetRoot(visual));
    scene::spawn_stage_support(commands, match_root, collision);
}
