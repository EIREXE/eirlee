use bevy::{prelude::*};

use crate::{fighter::{FighterAttributes, states}, game_settings::GameSettings, input::player::FighterInput};

pub enum FighterStateTransition {
    Wait,
    Walk,
    Dash,
    Run,
}

pub trait FighterState : Component + Sized {
    fn remove(&self,
        state_context: &FighterStateContext,
        commands: &mut Commands) {
        commands.entity(state_context.entity).remove::<Self>();
    }
    fn check_interrupt(
        &self,
        state_context: &FighterStateContext,
    ) -> Option<FighterStateTransition>;
}

pub struct FighterStateContext<'a> {
    pub input: &'a mut FighterInput,
    pub fighter_attribs: &'a FighterAttributes,
    pub game_settings: &'a Res<'a, GameSettings>,
    pub entity: Entity,
}

fn create_state(transition: FighterStateTransition, state_context: FighterStateContext, commands: &mut Commands) {
    let mut ent_cmd = commands.entity(state_context.entity);
    match transition {
        FighterStateTransition::Wait => ent_cmd.insert(states::wait::WaitState),
        FighterStateTransition::Walk => ent_cmd.insert(states::walk::WalkState),
        FighterStateTransition::Dash => ent_cmd.insert(states::dash::DashState::default()),
        FighterStateTransition::Run => ent_cmd.insert(states::run::RunState),
    };
}

fn get_state_debug_name(transition: &FighterStateTransition) -> &'static str {
    match transition {
        FighterStateTransition::Wait => "wait",
        FighterStateTransition::Walk => "walk",
        FighterStateTransition::Dash => "dash",
        FighterStateTransition::Run => "run",
    }
}

#[derive(Component)]
pub struct StateNameDebug(pub &'static str);

impl std::fmt::Display for StateNameDebug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn state_interrupt_system<T: FighterState + Component + std::fmt::Debug>(
    query: Query<(Entity, &T, &mut FighterInput, &FighterAttributes)>,
    game_settings: Res<GameSettings>,
    mut commands: Commands,
) {
    for (ent, state, mut input, attribs) in query {
        info!("{:?}", state);
        let state_context = FighterStateContext {
            input: &mut input,
            fighter_attribs: &attribs,
            game_settings: &game_settings,
            entity: ent,
        };
        if let Some(new_state) = state.check_interrupt(&state_context) {
            commands.entity(ent).remove::<T>();
            commands.entity(ent).insert(StateNameDebug(get_state_debug_name(&new_state)));
            create_state(new_state, state_context, &mut commands);
        }
    }
}
