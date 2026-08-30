//! Rollback networking: session setup and the engine-level rollback
//! registrations. Gameplay components are registered by the plugin that owns
//! them, not here.

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use bevy_ggrs::prelude::*;

pub mod debug;
pub mod session;

/// The GGRS configuration for this game — notably, what a frame of input is.
pub type GGRSCfg = GgrsConfig<crate::input::FighterInputFrame>;

#[derive(Default)]
pub struct FighterNetcodePlugin;

impl Plugin for FighterNetcodePlugin {
    fn build(&self, app: &mut App) {
        let plugin = GgrsPlugin::<GGRSCfg>::default();
        app.add_plugins(plugin)
            // Engine-level components that every rolled-back entity may carry.
            // Gameplay components are registered by the plugin that owns them.
            .rollback_component_with_clone::<Transform>()
            .rollback_component_with_clone::<Name>()
            .add_systems(
                EguiPrimaryContextPass,
                debug::network_debug.run_if(crate::debug_tools::network_enabled),
            )
            .add_systems(Update, session::print_events_system)
            .insert_resource(RollbackFrameRate(60));
    }
}
