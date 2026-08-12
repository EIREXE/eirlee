use bevy::prelude::*;

use super::{FighterState, FighterStateContext, FighterStateTransition, ground, wait, walk};
use crate::fighter::{FighterAttributes, FighterVelocity};
use crate::game_settings::GameSettings;
use crate::input::FighterInput;

#[derive(Component, Debug, Clone, Copy)]
pub struct RunState;

impl FighterState for RunState {
    fn check_interrupt(
        &self,
        state_context: &FighterStateContext,
    ) -> Option<FighterStateTransition> {
        if let Some(_) = check_input(state_context) {
            None
        } else {
            walk::check_input(state_context).or_else(|| wait::check_input(state_context))
        }
    }
}

pub fn check_input(state_context: &FighterStateContext) -> Option<FighterStateTransition> {
    let input_frame = state_context.input.get_last_frame();
    if input_frame.movement.x.abs() >= state_context.game_settings.input_common.run_stick_threshold
    {
        return Some(FighterStateTransition::Run);
    }
    None
}

pub fn run_update(
    fighters: Query<(&mut FighterVelocity, &FighterAttributes, &FighterInput), With<RunState>>,
    game_settings: Res<GameSettings>,
) {
    for (mut velocity, attributes, input) in fighters {
        let last_frame = input.get_last_frame();
        let (accel, target_vel) = attributes.get_accel_and_target_dashrun(&last_frame);
        let accel = ground::compute_ground_accel(
            accel,
            target_vel,
            velocity.x,
            attributes,
            game_settings.as_ref(),
        );
        velocity.x += accel;
    }
}
