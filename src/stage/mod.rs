//! The stage: its collision geometry, the scene that gets spawned, and how
//! both are drawn.

use bevy::prelude::*;

pub mod debug;
pub mod line;
pub mod scene;

pub use line::{StagePoly, StagePlaneIntersectResult};

/// Owns the stage geometry: which scene gets spawned and how it is drawn.
#[derive(Default)]
pub struct StagePlugin;

impl Plugin for StagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, scene::test_scene.spawn())
            .add_systems(Update, debug::debug_draw_scene);
    }
}
