//! The fighter state machine: the [`FighterState`] trait every state
//! implements, the context they inspect, and the generic system that runs the
//! interrupt check. One state per submodule.

use bevy::ecs::query::QueryData;
use bevy::prelude::*;

use crate::fighter::animation::{AnimKind, FighterAnimationFrame, FighterAnimationPlayerLink};
use crate::fighter::baked_animation::{BakedFighterAnimations, FixedMat4};
use crate::fighter::ecb::FighterPreviousECB;
use crate::fighter::manifest::FighterManifest;
use crate::fighter::visual::FighterAnimations;
use crate::fighter::{
    Fighter, FighterAttributes, FighterECB, FighterFacingDirection, FighterPreviousTranslation,
    FighterTranslation, FighterVelocity,
};
use crate::game_settings::GameSettings;
use crate::input::FighterInput;
use crate::stage::line::StageCollision;

pub mod air;
pub mod air_dodge;
pub mod dash;
pub mod fall;
pub mod ground;
pub mod jump;
pub mod land;
pub mod run;
pub mod turn;
pub mod wait;
pub mod walk;

macro_rules! fighter_states {
    ($($variant:ident => $ty:ty),* $(,)?) => {
        #[derive(Component, Clone, Debug, Hash)]
        pub enum FighterState { $($variant($ty)),* }

        impl FighterState {
            pub fn name(&self) -> &'static str {
                match self { $(Self::$variant(_) => <$ty>::NAME),* }
            }
            pub fn check_interrupt(&self, ctx: &mut FighterStateContext) -> Option<FighterState> {
                match self { $(Self::$variant(s) => s.check_interrupt(ctx)),* }
            }

            pub fn update(&mut self, ctx: &mut FighterStateContext) {
                match self { $(Self::$variant(s) => s.update(ctx)),* }
            }
            pub fn on_enter(&mut self, ctx: &mut FighterStateContext) {
                match self { $(Self::$variant(s) => s.on_enter(ctx)),* }
            }

            pub fn check_collision_interrupt(&mut self, ctx: &mut FighterStateContext) -> Option<FighterState> {
                match self { $(Self::$variant(s) => s.check_collision_interrupt(ctx)),* }
            }
        }

        $(impl From<$ty> for FighterState {
            fn from(s: $ty) -> Self { Self::$variant(s) }
        })*
    };
}

pub trait FighterStateImpl: Sized {
    const NAME: &'static str;
    fn check_interrupt(&self, state_context: &mut FighterStateContext) -> Option<FighterState>;

    fn on_enter(&mut self, _state_context: &mut FighterStateContext) {}
    fn update(&mut self, _state_context: &mut FighterStateContext);

    fn check_collision_interrupt(
        &mut self,
        _state_context: &mut FighterStateContext,
    ) -> Option<FighterState> {
        None
    }
}

pub struct FighterStateContext<'a> {
    pub translation: &'a mut FighterTranslation,
    pub prev_translation: &'a mut FighterPreviousTranslation,
    pub input: &'a mut FighterInput,
    pub velocity: &'a mut FighterVelocity,
    pub ecb: &'a mut FighterECB,
    pub prev_ecb: &'a mut FighterPreviousECB,
    pub facing_direction: &'a mut FighterFacingDirection,
    pub game_settings: &'a GameSettings,
    pub stage_collision: &'a StageCollision,
    pub animations: &'a FighterAnimations,
    pub animation_player: &'a mut AnimationPlayer,
    pub animation_transitions: &'a mut AnimationTransitions,
    pub animation_frame: &'a mut FighterAnimationFrame,
    pub baked_animations: &'a Assets<BakedFighterAnimations>,
    pub fighter_manifest: &'a FighterManifest,
}

impl FighterStateContext<'_> {
    pub fn play_animation(&mut self, kind: AnimKind, repeat: bool) {
        let active = self.animation_transitions.play(
            self.animation_player,
            self.animations.clips[&kind],
            std::time::Duration::ZERO,
        );
        if repeat {
            active.repeat();
        }
        self.animation_frame.reset(kind, repeat);
    }

    pub fn get_current_animation_frame(&self) -> u32 {
        self.animation_frame.frame
    }

    pub fn is_current_animation_finished(&self) -> bool {
        let anims_data = self
            .baked_animations
            .get(&self.animations.baked)
            .expect("Animation should exist");
        anims_data
            .frame_count(self.animation_frame.kind)
            .expect("Current animation should have baked frames")
            < self.animation_frame.frame
    }

    pub fn sample_bone(&self, bone: &str) -> Option<FixedMat4> {
        self.animations
            .sample_bone(self.baked_animations, *self.animation_frame, bone)
    }
}

fighter_states! {
    Wait => wait::WaitState,
    Walk => walk::WalkState,
    Dash => dash::DashState,
    Run => run::RunState,
    Fall => fall::FallState,
    JumpSquat => jump::JumpSquatState,
    Jump => jump::JumpState,
    Land => land::LandingState,
    AirDodge => air_dodge::AirDodgeState,
    Turn => turn::TurnState
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct FighterFrameQuery {
    pub translation: &'static mut FighterTranslation,
    pub prev_translation: &'static mut FighterPreviousTranslation,
    pub input: &'static mut FighterInput,
    pub velocity: &'static mut FighterVelocity,
    pub ecb: &'static mut FighterECB,
    pub prev_ecb: &'static mut FighterPreviousECB,
    pub facing_direction: &'static mut FighterFacingDirection,
    pub animation_frame: &'static mut FighterAnimationFrame,
    pub fighter: &'static Fighter,
}

impl<'w, 's> FighterFrameQueryItem<'w, 's> {
    fn context<'a>(
        &'a mut self,
        game_settings: &'a GameSettings,
        stage_collision: &'a StageCollision,
        animations: &'a FighterAnimations,
        animation_player: &'a mut AnimationPlayer,
        animation_transitions: &'a mut AnimationTransitions,
        baked_animations: &'a Assets<BakedFighterAnimations>,
        manifest: &'a FighterManifest,
    ) -> FighterStateContext<'a> {
        FighterStateContext {
            translation: &mut self.translation,
            prev_translation: &mut self.prev_translation,
            input: &mut self.input,
            velocity: &mut self.velocity,
            ecb: &mut self.ecb,
            prev_ecb: &mut self.prev_ecb,
            facing_direction: &mut self.facing_direction,
            game_settings,
            stage_collision,
            animations,
            animation_player,
            animation_transitions,
            animation_frame: &mut self.animation_frame,
            baked_animations,
            fighter_manifest: manifest,
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct StatefulFighterQuery {
    pub entity: Entity,
    pub state: &'static mut FighterState,
    pub frame: FighterFrameQuery,
    pub animations: &'static FighterAnimations,
    pub animation_player_link: &'static FighterAnimationPlayerLink,
    pub fighter: &'static Fighter,
}

#[derive(Component)]
pub struct StateNameDebug(pub &'static str);

impl std::fmt::Display for StateNameDebug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn state_update_system(
    mut query: Query<StatefulFighterQuery>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    stage_collision: Res<StageCollision>,
    game_settings: Res<GameSettings>,
    baked_animations: Res<Assets<BakedFighterAnimations>>,
    manifests: Res<Assets<FighterManifest>>,
) {
    for fighter in &mut query {
        let StatefulFighterQueryItem {
            mut state,
            mut frame,
            animations,
            animation_player_link,
            ..
        } = fighter;
        let Ok((mut animation_player, mut animation_transitions)) =
            animation_players.get_mut(animation_player_link.player())
        else {
            continue;
        };

        let mut update_ctx = frame.context(
            &game_settings,
            &stage_collision,
            animations,
            &mut animation_player,
            &mut animation_transitions,
            &baked_animations,
            &manifests
                .get(&(fighter.fighter.manifest))
                .expect("Fighter manifest should be valid"),
        );
        state.update(&mut update_ctx);
    }
}

pub fn state_interrupt_system(
    mut query: Query<StatefulFighterQuery>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    stage_collision: Res<StageCollision>,
    game_settings: Res<GameSettings>,
    baked_animations: Res<Assets<BakedFighterAnimations>>,
    fighter_manifests: Res<Assets<FighterManifest>>,
    mut commands: Commands,
) {
    for fighter in &mut query {
        let StatefulFighterQueryItem {
            entity,
            state,
            mut frame,
            animations,
            animation_player_link,
            fighter,
        } = fighter;
        let Ok((mut animation_player, mut animation_transitions)) =
            animation_players.get_mut(animation_player_link.player())
        else {
            continue;
        };
        let mut state_context = frame.context(
            &game_settings,
            &stage_collision,
            animations,
            &mut animation_player,
            &mut animation_transitions,
            &baked_animations,
            fighter_manifests
                .get(&fighter.manifest)
                .expect("Fighter manifest should be valid"),
        );
        if let Some(new_state) = state.check_interrupt(&mut state_context) {
            let mut new_state = new_state;
            let old_state_name = state.name();
            let new_state_name = new_state.name();
            new_state.on_enter(&mut state_context);
            commands
                .entity(entity)
                .insert((new_state, StateNameDebug(new_state_name)));
            info!("State {} -> {}", old_state_name, new_state_name);
        }
    }
}

pub fn state_collision_interrupt_system(
    mut query: Query<StatefulFighterQuery>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    stage_collision: Res<StageCollision>,
    game_settings: Res<GameSettings>,
    baked_animations: Res<Assets<BakedFighterAnimations>>,
    fighter_manifests: Res<Assets<FighterManifest>>,
    mut commands: Commands,
) {
    for fighter in &mut query {
        let StatefulFighterQueryItem {
            entity,
            mut state,
            mut frame,
            animations,
            animation_player_link,
            fighter,
        } = fighter;
        let Ok((mut animation_player, mut animation_transitions)) =
            animation_players.get_mut(animation_player_link.player())
        else {
            continue;
        };
        let mut state_context = frame.context(
            &game_settings,
            &stage_collision,
            animations,
            &mut animation_player,
            &mut animation_transitions,
            &baked_animations,
            fighter_manifests
                .get(&fighter.manifest)
                .expect("Fighter manifest should be valid"),
        );
        if let Some(new_state) = state.check_collision_interrupt(&mut state_context) {
            let mut new_state = new_state;
            let old_state_name = state.name();
            let new_state_name = new_state.name();
            new_state.on_enter(&mut state_context);
            commands
                .entity(entity)
                .insert((new_state, StateNameDebug(new_state_name)));
            info!("State {} -> {}", old_state_name, new_state_name);
        }
    }
}
