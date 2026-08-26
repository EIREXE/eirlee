//! Everything fighter-shaped: the components that describe a fighter, the
//! state machine that drives it, and the plugin that schedules it all.

use bevy::{ecs::name::HashedStr, prelude::*};
use bevy_egui::EguiPrimaryContextPass;
use bevy_ggrs::prelude::*;

pub mod animation;
pub mod attributes;
pub mod collision;
pub mod debug;
pub mod ecb;
pub mod motion;
pub mod state;
pub mod visual;

// The fighter components are re-exported so the rest of the crate can say
// `fighter::FighterVelocity` without caring which file it lives in. Named
// explicitly rather than globbed, so moving a type between submodules can't
// silently change what this module exports.
pub use attributes::FighterAttributes;
pub use ecb::FighterECB;
pub use motion::{FighterPreviousTranslation, FighterTranslation, FighterVelocity, Grounded};

use crate::{
    fighter::{
        animation::animation_init,
        state::{FighterState, fall::FallState},
        visual::FighterAnimations,
    },
    input::FighterInput,
    math::{int::FGi32, vec::FGVec2},
    player::Player,
    schedule::GameplaySet,
    stage::line::StageCollision,
};
use state::state_interrupt_system;

/// Owns fighter simulation, the rollback registrations for fighter
/// components, and the fighter debug overlays.
#[derive(Default)]
pub struct FighterPlugin;

#[derive(Component, Copy, Clone, Hash)]
pub enum FighterFacingDirection {
    Left,
    Right,
}

#[derive(Component)]
#[require(Transform)]
pub struct FighterVisual;

impl Plugin for FighterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (animation_init, animation::setup_fighter_animation_player),
        )
        // Every state gets a chance to interrupt itself, in priority order.
        .add_systems(
            GgrsSchedule,
            (
                ecb::snapshot_fighter_ecb.before(state_interrupt_system),
                state::state_interrupt_system,
            )
                .in_set(GameplaySet::Interrupt),
        )
        .add_systems(
            GgrsSchedule,
            (state::state_update_system,).in_set(GameplaySet::StateUpdate),
        )
        /*.add_systems(
            GgrsSchedule,
            (motion::integrate_gravity, motion::apply_grounded_motion)
                .chain()
                .in_set(GameplaySet::Physics),
        )*/
        .add_systems(
            GgrsSchedule,
            state::state_collision_interrupt_system.in_set(GameplaySet::Collision),
        )
        .add_systems(
            GgrsSchedule,
            (
                animation::apply_fighter_translation_to_visuals,
                animation::apply_animation,
            )
                .chain()
                .in_set(GameplaySet::Animation),
        )
        // Rollback registration lives next to the systems that write these
        // components; a component simulated here but missing from this list
        // is a desync waiting to happen.
        .rollback_component_with_copy::<FighterECB>()
        .rollback_component_with_copy::<ecb::FighterPreviousECB>()
        .rollback_component_with_copy::<FighterVelocity>()
        .rollback_component_with_copy::<FighterTranslation>()
        .rollback_component_with_copy::<FighterPreviousTranslation>()
        .rollback_component_with_copy::<FighterAttributes>()
        .rollback_component_with_copy::<Grounded>()
        .rollback_resource_with_reflect::<StageCollision>()
        // Debug views.
        .add_systems(FixedPostUpdate, debug::debug_draw_ecb)
        .add_systems(EguiPrimaryContextPass, debug::fighter_debug)
        .add_systems(EguiPrimaryContextPass, debug::update_config)
        .rollback_component_with_clone::<FighterState>()
        .checksum_component_with_hash::<FighterState>();
    }
}

pub fn spawn_fighter(
    commands: &mut Commands,
    animations: FighterAnimations,
    visual_root: WorldAssetRoot,
) {
    commands.spawn((
        Name("Fighter".into()),
        Player { handle: 0 },
        FighterFacingDirection::Right,
        FighterECB {
            vertical_half: FGi32::lit("5.0"),
            horizontal_half: FGi32::lit("2.5"),
        },
        FighterVelocity::default(),
        FighterTranslation(FGVec2::lit("5.0", "12.5")),
        FighterAttributes {
            base_walk_accel: FGi32::lit("0.1"),
            stick_walk_accel: FGi32::lit("0.16"),
            max_walk_vel: FGi32::lit("0.7"),
            ground_friction: FGi32::lit("0.09"),
            dash_acceleration_duration: 13,
            dash_duration: 24,
            base_dash_accel: FGi32::lit("0.02"),
            stick_dash_accel: FGi32::lit("0.065"),
            max_dash_vel: FGi32::lit("1.1"),
            jump_vertical_velocity: FGi32::lit("2.5"),
            dash_initial_velocity: FGi32::lit("1.4"),
            terminal_velocity: FGi32::lit("2.8"),
            gravity: FGi32::lit("0.23"),
            jumpsquat_duration: 4,
        },
        visual_root,
        animations,
        FighterState::Fall(FallState),
        FighterVisual,
        FighterInput::default(),
    ));
}
