//! Everything fighter-shaped: the components that describe a fighter, the
//! state machine that drives it, and the plugin that schedules it all.

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use bevy_ggrs::prelude::*;
use serde::{Deserialize, Serialize};

pub mod animation;
pub mod attributes;
pub mod baked_animation;
pub mod collision;
pub mod debug;
pub mod ecb;
pub mod manifest;
pub mod motion;
pub mod state;
pub mod visual;

// The fighter components are re-exported so the rest of the crate can say
// `fighter::FighterVelocity` without caring which file it lives in. Named
// explicitly rather than globbed, so moving a type between submodules can't
// silently change what this module exports.
pub use attributes::FighterAttributes;
pub use ecb::FighterECB;
pub use manifest::FighterId;
pub use motion::{FighterPreviousTranslation, FighterTranslation, FighterVelocity, Grounded};

use crate::{
    fighter::{
        animation::animation_init,
        manifest::FighterManifest,
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

#[derive(Component, Copy, Clone, Hash, Debug, PartialEq, Reflect)]
pub enum FighterFacingDirection {
    Left,
    Right,
}

impl FighterFacingDirection {
    fn reverse(&self) -> Self {
        match self {
            FighterFacingDirection::Left => FighterFacingDirection::Right,
            FighterFacingDirection::Right => FighterFacingDirection::Left,
        }
    }
}

#[derive(Copy, Clone, Debug, Reflect, Serialize, Deserialize)]
#[reflect(opaque)]
pub struct FighterCameraProfile {
    pub vertical_origin_offset: FGi32,
    pub forward_extent: FGi32,
    pub backward_extent: FGi32,
    pub upward_extent: FGi32,
    pub downward_extent: FGi32,
    /// Reserved for future offscreen indicators and close-up camera modes.
    pub visibility_radius: FGi32,
}

impl FighterCameraProfile {
    pub fn is_valid(&self) -> bool {
        self.forward_extent >= FGi32::ZERO
            && self.backward_extent >= FGi32::ZERO
            && self.upward_extent >= FGi32::ZERO
            && self.downward_extent >= FGi32::ZERO
            && self.visibility_radius >= FGi32::ZERO
    }
}

impl FighterFacingDirection {
    pub fn to_sign(&self) -> FGi32 {
        match self {
            FighterFacingDirection::Left => FGi32::NEG_ONE,
            FighterFacingDirection::Right => FGi32::ONE,
        }
    }
}

#[derive(Component)]
#[require(Transform)]
pub struct FighterVisual;

#[derive(Component)]
pub struct Fighter {
    id: FighterId,
    pub manifest: Handle<FighterManifest>,
}

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
                animation::advance_fighter_animation_frames.before(state_interrupt_system),
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
        .rollback_component_with_copy::<animation::FighterAnimationFrame>()
        .checksum_component_with_hash::<animation::FighterAnimationFrame>()
        .rollback_component_with_copy::<Grounded>()
        .rollback_component_with_copy::<FighterFacingDirection>()
        .rollback_resource_with_reflect::<StageCollision>()
        // Debug views.
        .add_systems(FixedPostUpdate, debug::debug_draw_ecb)
        .add_systems(EguiPrimaryContextPass, debug::fighter_debug)
        .add_systems(FixedPostUpdate, debug::animation_debug)
        .add_systems(EguiPrimaryContextPass, debug::update_config)
        .rollback_component_with_clone::<FighterState>()
        .checksum_component_with_hash::<FighterState>()
        .checksum_component_with_hash::<FighterFacingDirection>()
        .register_type::<FighterFacingDirection>();
    }
}

pub fn spawn_fighter(
    commands: &mut Commands,
    player_handle: usize,
    spawn_position: FGVec2,
    manifest: (Handle<FighterManifest>, &FighterManifest),
    _attributes: FighterAttributes,
    animations: FighterAnimations,
    visual_root: WorldAssetRoot,
) {
    commands.spawn((
        Name(format!("Fighter {player_handle}").into()),
        Player {
            handle: player_handle,
        },
        Fighter {
            id: manifest.1.id,
            manifest: manifest.0,
        },
        FighterFacingDirection::Right,
        FighterECB {
            vertical_half: FGi32::lit("5.0"),
            horizontal_half: FGi32::lit("2.5"),
        },
        FighterVelocity::default(),
        FighterTranslation(spawn_position),
        visual_root,
        animations,
        FighterState::Fall(FallState),
        animation::FighterAnimationFrame::new(animation::AnimKind::Wait, true),
        FighterVisual,
        FighterInput::default(),
        crate::camera::FighterCameraExtents::default(),
    ));
}
