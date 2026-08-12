//! Everything fighter-shaped: the components that describe a fighter, the
//! state machine that drives it, and the plugin that schedules it all.

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use bevy_ggrs::prelude::*;

pub mod attributes;
pub mod collision;
pub mod debug;
pub mod ecb;
pub mod motion;
pub mod state;

// The fighter components are re-exported so the rest of the crate can say
// `fighter::FighterVelocity` without caring which file it lives in. Named
// explicitly rather than globbed, so moving a type between submodules can't
// silently change what this module exports.
pub use attributes::FighterAttributes;
pub use ecb::FighterECB;
pub use motion::{FighterPreviousTranslation, FighterTranslation, FighterVelocity, Grounded};

use crate::schedule::GameplaySet;
use state::state_interrupt_system;

/// Owns fighter simulation, the rollback registrations for fighter
/// components, and the fighter debug overlays.
#[derive(Default)]
pub struct FighterPlugin;

impl Plugin for FighterPlugin {
    fn build(&self, app: &mut App) {
        app
            // Every state gets a chance to interrupt itself, in priority order.
            .add_systems(
                GgrsSchedule,
                (
                    state_interrupt_system::<state::wait::WaitState>,
                    state_interrupt_system::<state::walk::WalkState>,
                    state_interrupt_system::<state::dash::DashState>,
                    state_interrupt_system::<state::run::RunState>,
                )
                    .chain()
                    .in_set(GameplaySet::Interrupt),
            )
            .add_systems(
                GgrsSchedule,
                (
                    state::wait::wait_update,
                    state::walk::walk_update,
                    state::dash::dash_update,
                    state::run::run_update,
                )
                    .chain()
                    .in_set(GameplaySet::StateUpdate),
            )
            .add_systems(
                GgrsSchedule,
                (motion::integrate_gravity, motion::apply_motion)
                    .chain()
                    .in_set(GameplaySet::Physics),
            )
            .add_systems(
                GgrsSchedule,
                collision::collide_fighter_with_scene.in_set(GameplaySet::Collision),
            )
            .add_observer(state::dash::on_insert_dash)
            // Rollback registration lives next to the systems that write these
            // components; a component simulated here but missing from this list
            // is a desync waiting to happen.
            .rollback_component_with_copy::<FighterECB>()
            .rollback_component_with_copy::<FighterVelocity>()
            .rollback_component_with_copy::<FighterTranslation>()
            .rollback_component_with_copy::<FighterPreviousTranslation>()
            .rollback_component_with_copy::<FighterAttributes>()
            .rollback_component_with_copy::<Grounded>()
            .rollback_component_with_copy::<state::wait::WaitState>()
            .rollback_component_with_copy::<state::walk::WalkState>()
            .rollback_component_with_copy::<state::dash::DashState>()
            .rollback_component_with_copy::<state::run::RunState>()
            .checksum_component::<state::dash::DashState>(state::dash::hash_dash_state)
            // Debug views.
            .add_systems(FixedPostUpdate, debug::debug_draw_ecb)
            .add_systems(EguiPrimaryContextPass, debug::fighter_debug);
    }
}
