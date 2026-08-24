use bevy::prelude::*;

use super::{FighterStateImpl, FighterStateContext, ground, walk};
use crate::fighter::state::FighterState;
use crate::fighter::state::fall::FallState;
use crate::fighter::state::ground::{GroundedMotionResult, GroundedStateCommon};
use crate::fighter::{FighterAttributes, FighterVelocity};
use crate::game_settings::GameSettings;
use crate::stage::line::StageLineID;

#[derive(Debug, Clone, Hash)]
pub struct WaitState {
    pub grounded_common: GroundedStateCommon
}

impl FighterStateImpl for WaitState {
    const NAME: &'static str = "Wait";
    fn check_interrupt(
        &self,
        state_context: &FighterStateContext,
    ) -> Option<FighterState> {
        walk::check_input(state_context, &self.grounded_common)
    }
    
    fn on_enter(&mut self, _state_context: &mut super::FighterStateContext) {}
    
    fn update(&mut self, state_context: &mut super::FighterStateContext) {
        let friction = if state_context.velocity.x.abs() > state_context.fighter_attribs.max_walk_vel {
            state_context.fighter_attribs.ground_friction
                * state_context.game_settings
                    .figher_common
                    .ground_friction_over_walk_speed_multiplier
        } else {
            state_context.fighter_attribs.ground_friction
        };

        let ground_velocity = state_context.velocity.x;

        state_context.velocity.x += ground::apply_grounded_friction(friction, ground_velocity);

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

    if input_frame.movement.x.abs() < state_context.game_settings.input_common.stick_deadzone {
        return Some(FighterState::Wait(WaitState { grounded_common: ground_state_common.clone() }));
    }
    None
}

