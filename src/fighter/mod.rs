//! Everything fighter-shaped: the components that describe a fighter, the
//! state machine that drives it, and the plugin that schedules it all.

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use bevy_ggrs::prelude::*;
use serde::{Deserialize, Serialize};

pub mod animation;
pub mod attack;
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
        attack::FighterAttackScriptAssets,
        manifest::FighterManifest,
        state::{FighterState, fall::FallState},
        visual::FighterAnimations,
    },
    input::FighterInput,
    math::{int::FGi32, vec::FGVec2},
    player::Player,
    schedule::GameplaySet,
    scripting::FighterAttackScript,
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
    pub manifest: Handle<FighterManifest>,
}

impl Plugin for FighterPlugin {
    fn build(&self, app: &mut App) {
        app
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
                Update,
                (
                    motion::apply_fighter_translation_to_visuals,
                    animation::apply_animation,
                )
                    .chain()
                    .run_if(in_state(crate::AppState::InMatch)),
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
            .add_systems(
                FixedPostUpdate,
                debug::debug_draw_ecb.run_if(crate::debug_tools::ecb_enabled),
            )
            .add_systems(
                EguiPrimaryContextPass,
                debug::fighter_debug.run_if(crate::debug_tools::fighter_info_enabled),
            )
            .add_systems(
                FixedPostUpdate,
                debug::animation_debug.run_if(crate::debug_tools::animation_bones_enabled),
            )
            .add_systems(
                FixedPostUpdate,
                debug::attack_debug.run_if(crate::debug_tools::attack_hitboxes_enabled),
            )
            .rollback_component_with_clone::<FighterState>()
            .rollback_component_with_clone::<FighterHitboxes>()
            .checksum_component_with_hash::<FighterState>()
            .checksum_component_with_hash::<FighterFacingDirection>()
            .register_type::<FighterFacingDirection>();
    }
}

#[derive(Component, Clone)]
pub struct FighterHitboxes {
    pub attack_script: Option<Handle<FighterAttackScript>>,
    pub active_hitboxes: Vec<usize>,
}

impl FighterHitboxes {
    pub fn clear(&mut self) {
        self.attack_script = None;
        self.active_hitboxes.clear();
    }
}

pub fn spawn_fighter(
    commands: &mut Commands,
    player_handle: usize,
    spawn_position: FGVec2,
    manifest: Handle<FighterManifest>,
    animations: FighterAnimations,
    attack_scripts: FighterAttackScriptAssets,
    visual_root: WorldAssetRoot,
) {
    commands.spawn((
        Name(format!("Fighter {player_handle}").into()),
        Player {
            handle: player_handle,
        },
        Fighter { manifest: manifest },
        FighterFacingDirection::Right,
        (
            FighterECB {
                vertical_half: FGi32::lit("5.0"),
                horizontal_half: FGi32::lit("2.5"),
            },
            FighterHitboxes {
                attack_script: None,
                active_hitboxes: vec![],
            },
        ),
        (
            FighterVelocity::default(),
            FighterTranslation(spawn_position),
            FighterPreviousTranslation(spawn_position),
        ),
        visual_root,
        animations,
        attack_scripts,
        FighterState::Fall(FallState),
        animation::FighterAnimationFrame::new(animation::AnimKind::Wait, true),
        baked_animation::FighterBoneMatrices::default(),
        FighterVisual,
        FighterInput::default(),
        crate::camera::FighterCameraExtents::default(),
    ));
}
