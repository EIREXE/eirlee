use bevy::prelude::*;
use crate::fighter::{state::{FighterState, FighterStateContext, FighterStateTransition}, states::walk};

#[derive(Component, Debug)]
pub struct WaitState;

impl FighterState for WaitState {
    fn check_interrupt(
        &self,
        state_context: &FighterStateContext,
    ) -> Option<FighterStateTransition> {
        walk::check_input(state_context)
    }
}

pub fn check_input(state_context: &FighterStateContext) -> Option<FighterStateTransition> {
    if let Some(input_frame) = state_context.input.get_last_frame() {
        if input_frame.movement.x.abs() < state_context.game_settings.input_common.stick_deadzone {
            return Some(FighterStateTransition::Wait);
        }
    }
    None
}