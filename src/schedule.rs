//! The gameplay frame pipeline.
//!
//! Every rollback-simulated system belongs to exactly one of these sets, and
//! the sets run in the order declared here. Plugins add their systems with
//! `.in_set(GameplaySet::X)` instead of being threaded into one central
//! `.chain()`, so a module can schedule its own work without any other module
//! knowing about it.

use bevy::prelude::*;
use bevy_ggrs::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameplaySet {
    /// Turn this frame's netcode inputs into per-fighter input state.
    Input,
    /// Decide whether the current fighter state hands off to another one.
    Interrupt,
    /// Run the fighter state that survived the interrupt pass.
    StateUpdate,
    /// Integrate velocities into positions.
    Physics,
    /// Resolve the integrated positions against the stage.
    Collision,
}

/// Declares the order of [`GameplaySet`] inside [`GgrsSchedule`]. Added by
/// `GamePlugin` before any plugin that populates the sets.
pub struct GameplaySchedulePlugin;

impl Plugin for GameplaySchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            GgrsSchedule,
            (
                GameplaySet::Input,
                GameplaySet::Interrupt,
                GameplaySet::StateUpdate,
                GameplaySet::Physics,
                GameplaySet::Collision,
            )
                .chain(),
        );
    }
}
