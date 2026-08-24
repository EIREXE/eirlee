//! The fighter state machine: the [`FighterState`] trait every state
//! implements, the context they inspect, and the generic system that runs the
//! interrupt check. One state per submodule.

use bevy::ecs::component::Mutable;
use bevy::ecs::system::ScheduleSystem;
use bevy::prelude::*;
use bevy_ggrs::{GgrsSchedule, RollbackApp};

use crate::fighter::ecb::FighterPreviousECB;
use crate::fighter::{FighterAttributes, FighterECB, FighterPreviousTranslation, FighterTranslation, FighterVelocity};
use crate::game_settings::GameSettings;
use crate::input::FighterInput;
use crate::math::int::FGi32;
use crate::schedule::GameplaySet;
use crate::stage::StagePoly;
use crate::stage::line::StageCollision;

pub mod dash;
pub mod ground;
pub mod run;
pub mod wait;
pub mod walk;
pub mod fall;
pub mod air;
pub mod jump;

macro_rules! fighter_states {
    ($($variant:ident => $ty:ty),* $(,)?) => {
        #[derive(Component, Clone, Debug, Hash)]
        pub enum FighterState { $($variant($ty)),* }

        impl FighterState {
            pub fn name(&self) -> &'static str {
                match self { $(Self::$variant(_) => <$ty>::NAME),* }
            }
            pub fn check_interrupt(&self, ctx: &FighterStateContext) -> Option<FighterState> {
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
    fn check_interrupt(
        &self,
        state_context: &FighterStateContext,
    ) -> Option<FighterState>;

    fn on_enter(&mut self, _state_context: &mut FighterStateContext) {}
    fn update(&mut self, _state_context: &mut FighterStateContext);

    fn check_collision_interrupt(
        &mut self,
        _state_context: &mut FighterStateContext,
    ) -> Option<FighterState> {
        None
    }
}

pub struct FighterStateContext<'a, 'w, 's> {
    pub translation: &'a mut FighterTranslation,
    pub prev_translation: &'a mut FighterPreviousTranslation,
    pub input: &'a mut FighterInput,
    pub velocity: &'a mut FighterVelocity,
    pub ecb: &'a mut FighterECB,
    pub prev_ecb: &'a mut FighterPreviousECB,
    pub fighter_attribs: &'a FighterAttributes,
    pub game_settings: &'a GameSettings,
    pub entity: Entity,
    pub stage_collision: &'a StageCollision,
    pub gizmos: Option<&'a mut Gizmos<'w, 's>>,
}

fighter_states! {
    Wait => wait::WaitState,
    Walk => walk::WalkState,
    Dash => dash::DashState,
    Run => run::RunState,
    Fall => fall::FallState,
    JumpSquat => jump::JumpSquatState,
    Jump => jump::JumpState
}

#[derive(Component)]
pub struct StateNameDebug(pub &'static str);

impl std::fmt::Display for StateNameDebug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn state_update_system(
    query: Query<(Entity, &mut FighterState, &mut FighterTranslation, &mut FighterPreviousTranslation, &mut FighterInput, &mut FighterVelocity, &mut FighterECB, &mut FighterPreviousECB, &FighterAttributes)>,
    stage_collision: Res<StageCollision>,
    game_settings: Res<GameSettings>,
    mut gizmos: Gizmos,
) {
    for (ent, mut state, mut translation,  mut prev_translation, mut input, mut velocity, mut ecb, mut prev_ecb, attribs) in query {
        let mut update_ctx = FighterStateContext {
            translation: &mut translation,
            prev_translation: &mut prev_translation,
            input: &mut input,
            velocity: &mut velocity,
            ecb: &mut ecb,
            prev_ecb: &mut prev_ecb,
            fighter_attribs: &attribs,
            game_settings: &game_settings,
            entity: ent,
            stage_collision: &stage_collision,
            gizmos: Some(&mut gizmos),
        };
        state.update(&mut update_ctx);
    }
}

pub fn state_interrupt_system (
    query: Query<(Entity, &FighterState, &mut FighterTranslation, &mut FighterPreviousTranslation, &mut FighterInput, &mut FighterVelocity, &mut FighterECB, &mut FighterPreviousECB, &FighterAttributes)>,
    stage_collision: Res<StageCollision>,
    game_settings: Res<GameSettings>,
    mut commands: Commands,
) {
    for (ent, state, mut translation, mut prev_translation, mut input, mut velocity, mut ecb, mut prev_ecb, attribs) in query {
        let mut state_context = FighterStateContext {
            translation: &mut translation,
            prev_translation: &mut prev_translation,
            input: &mut input,
            velocity: &mut velocity,
            ecb: &mut ecb,
            prev_ecb: &mut prev_ecb,
            fighter_attribs: &attribs,
            game_settings: &game_settings,
            entity: ent,
            stage_collision: &stage_collision,
            gizmos: None,
        };
        if let Some(new_state) = state.check_interrupt(&state_context) {
            let mut new_state = new_state;
            let old_state_name = state.name();
            let new_state_name = new_state.name();
            new_state.on_enter(&mut state_context);
            commands.entity(ent).insert((
                new_state,
                StateNameDebug(new_state_name)
            ));
            info!("State {} -> {}", old_state_name, new_state_name);
        }
    }
}

pub fn state_collision_interrupt_system(
    query: Query<(Entity, &mut FighterState, &mut FighterTranslation, &mut FighterPreviousTranslation, &mut FighterInput, &mut FighterVelocity, &mut FighterECB, &mut FighterPreviousECB, &FighterAttributes)>,
    stage_collision: Res<StageCollision>,
    game_settings: Res<GameSettings>,
    mut commands: Commands,
    mut gizmos: Gizmos,
) {
    for (ent, mut state, mut translation, mut prev_translation, mut input, mut velocity, mut ecb, mut prev_ecb, attribs) in query {
        let mut state_context = FighterStateContext {
            translation: &mut translation,
            prev_translation: &mut prev_translation,
            input: &mut input,
            velocity: &mut velocity,
            ecb: &mut ecb,
            prev_ecb: &mut prev_ecb,
            fighter_attribs: &attribs,
            game_settings: &game_settings,
            entity: ent,
            stage_collision: &stage_collision,
            gizmos: Some(&mut gizmos),
        };
        if let Some(new_state) = state.check_collision_interrupt(&mut state_context) {
            let mut new_state = new_state;
            let old_state_name = state.name();
            let new_state_name = new_state.name();
            new_state.on_enter(&mut state_context);
            commands.entity(ent).insert((
                new_state,
                StateNameDebug(new_state_name)
            ));
            info!("State {} -> {}", old_state_name, new_state_name);
        }
    }
}
