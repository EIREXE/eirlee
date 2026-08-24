use bevy::prelude::*;

use super::{FighterStateImpl, FighterStateContext, FighterStateTransition, ground, wait, walk};
use crate::fighter::state::FighterState;
use crate::fighter::state::fall::FallState;
use crate::fighter::state::ground::{GroundedMotionResult, GroundedStateCommon};
use crate::fighter::{FighterAttributes, FighterVelocity};
use crate::game_settings::GameSettings;
use crate::input::FighterInput;

#[derive(Component, Debug, Clone, Hash)]
pub struct RunState {
    pub grounded_common: GroundedStateCommon
}

impl FighterStateImpl for RunState {
    const NAME: &'static str = "Run";
    fn check_interrupt(
        &self,
        state_context: &FighterStateContext,
    ) -> Option<FighterState> {
        let input_frame = state_context.input.get_last_frame();
        if input_frame.movement.x.abs() >= state_context.game_settings.input_common.run_stick_threshold {
            None
        } else {
            walk::check_input(state_context, &self.grounded_common).or_else(|| wait::check_input(state_context, &self.grounded_common))
        }
    }
    
    fn on_enter(&mut self, _state_context: &mut super::FighterStateContext) {}
    
    fn update(&mut self, state_context: &mut super::FighterStateContext) {
        let last_frame = state_context.input.get_last_frame();
        let (accel, target_vel) = state_context.fighter_attribs.get_accel_and_target_dashrun(&last_frame);
        let accel = ground::compute_ground_accel(
            accel,
            target_vel,
            state_context.velocity.x,
            state_context.fighter_attribs,
            state_context.game_settings,
        );
        state_context.velocity.x += accel;

        ground::apply_grounded_motion(
            state_context.velocity,
            state_context.translation,
            state_context.prev_translation,
        );
    }

    fn check_collision_interrupt(
        &mut self,
        state_context: &mut FighterStateContext,
    ) -> Option<FighterState> {
        if let GroundedMotionResult::InAir = ground::collide_with_stage_grounded(
            state_context,
            &mut self.grounded_common,
            false
        ) {
            Some(FighterState::Fall(FallState))
        } else {
            None
        }
    }
}

pub fn check_input(state_context: &FighterStateContext, ground_state_common: &GroundedStateCommon) -> Option<FighterState> {
    let input_frame = state_context.input.get_last_frame();
    if input_frame.movement.x.abs() >= state_context.game_settings.input_common.run_stick_threshold
    {
        return Some(FighterState::Run(RunState { grounded_common: ground_state_common.clone() }));
    }
    None
}