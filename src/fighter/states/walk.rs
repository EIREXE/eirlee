use crate::{
    fighter::{
        FighterAttributes, FighterVelocity, state::{FighterState, FighterStateContext, FighterStateTransition}, states::{dash, grounded, wait},
    }, game_settings::GameSettings, input::player::FighterInput,
};
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct WalkState;

impl FighterState for WalkState {
    fn check_interrupt(
        &self,
        state_context: &FighterStateContext,
    ) -> Option<FighterStateTransition> {
        wait::check_input(state_context).or_else(|| dash::check_input(state_context))
    }
}

pub fn walk_update(
    fighters: Query<(
        &mut FighterVelocity,
        &FighterAttributes,
        &FighterInput,
    ), With<WalkState>>,
    game_settings: Res<GameSettings>,
) {
    for (mut velocity, attributes, input) in fighters {
        if let Some(last_frame) = input.get_last_frame() {
            let (accel, target_vel) = attributes.get_accel_and_target_walk(last_frame);

            let accel = grounded::compute_ground_accel(accel, target_vel, velocity.x, attributes, &game_settings);

            velocity.x += accel;
        }
    }
}

pub fn check_input(state_context: &FighterStateContext) -> Option<FighterStateTransition> {
    if let Some(input_frame) = state_context.input.get_last_frame() {
        if input_frame.movement.x.abs() >= state_context.game_settings.input_common.stick_deadzone {
            return Some(FighterStateTransition::Walk);
        }
    }
    None
}
