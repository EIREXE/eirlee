use crate::{
    fighter::{FighterAttributes, FighterVelocity, state::{FighterStateContext, FighterStateTransition}, states::grounded}, game_settings::GameSettings, input::player::FighterInput, state::FighterState,
};
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct RunState;

pub fn check_input(state_context: &FighterStateContext) -> Option<FighterStateTransition> {
    if let Some(input_frame) = state_context.input.get_last_frame() {
        if input_frame.movement.x.abs() >= state_context.game_settings.input_common.run_stick_threshold {
            return Some(FighterStateTransition::Run);
        }
    }
    None
}

impl FighterState for RunState {
    fn check_interrupt(
        &self,
        state_context: &crate::fighter::state::FighterStateContext,
    ) -> Option<crate::fighter::state::FighterStateTransition> {
        use super::*;

        if let Some(_) = check_input(state_context) {
            None
        } else {
            walk::check_input(state_context).or_else(|| wait::check_input(state_context))            
        }
    }
}

pub fn run_update(
    fighters: Query<(&mut FighterVelocity, &FighterAttributes, &FighterInput), With<RunState>>,
    game_settings: Res<GameSettings>,
) {
    for ( mut velocity, attributes, input) in fighters {
        if let Some(last_frame) = input.get_last_frame() {
            let (accel, target_vel) = attributes.get_accel_and_target_dashrun(last_frame);
            let accel =  grounded::compute_ground_accel(accel, target_vel, velocity.x, attributes, game_settings.as_ref());
            velocity.x += accel;
        }
    }
}
