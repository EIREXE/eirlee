use bevy::prelude::*;

use super::{FighterState, FighterStateContext, FighterStateTransition, dash, ground, wait};
use crate::fighter::{FighterAttributes, FighterVelocity};
use crate::game_settings::GameSettings;
use crate::input::FighterInput;

#[derive(Component, Debug, Clone, Copy)]
pub struct WalkState;

impl FighterState for WalkState {
    fn check_interrupt(
        &self,
        state_context: &FighterStateContext,
    ) -> Option<FighterStateTransition> {
        wait::check_input(state_context).or_else(|| dash::check_input(state_context))
    }
}

pub fn check_input(state_context: &FighterStateContext) -> Option<FighterStateTransition> {
    let input_frame = state_context.input.get_last_frame();

    if input_frame.movement.x.abs() >= state_context.game_settings.input_common.stick_deadzone {
        return Some(FighterStateTransition::Walk);
    }

    None
}

pub fn walk_update(
    fighters: Query<(&mut FighterVelocity, &FighterAttributes, &FighterInput), With<WalkState>>,
    game_settings: Res<GameSettings>,
) {
    for (mut velocity, attributes, input) in fighters {
        let last_frame = input.get_last_frame();
        let (accel, target_vel) = attributes.get_accel_and_target_walk(&last_frame);

        let accel =
            ground::compute_ground_accel(accel, target_vel, velocity.x, attributes, &game_settings);

        velocity.x += accel;
    }
}
