use bevy::prelude::*;

use super::{FighterStateContext, FighterStateImpl, ground, walk};
use crate::fighter::state::ground::GroundedStateCommon;
use crate::fighter::state::land::LandingState;
use crate::fighter::state::wait::WaitState;
use crate::fighter::state::{FighterState, air};
use crate::fighter::{FighterAttributes, FighterVelocity, collision, motion};
use crate::game_settings::GameSettings;
use crate::stage::line::StageCollision;

#[derive(Debug, Clone, Copy, Hash)]
pub struct FallState;

impl FighterStateImpl for FallState {
    const NAME: &'static str = "Fall";
    fn check_interrupt(&self, state_context: &FighterStateContext) -> Option<FighterState> {
        None
    }

    fn on_enter(&mut self, _state_context: &mut super::FighterStateContext) {}

    fn update(&mut self, state_context: &mut super::FighterStateContext) {
        air::integrate_gravity(state_context.velocity, state_context.fighter_attribs);
        air::apply_air_motion(
            state_context.velocity,
            state_context.translation,
            state_context.prev_translation,
        );
    }

    fn check_collision_interrupt(
        &mut self,
        state_context: &mut FighterStateContext,
    ) -> Option<FighterState> {
        if let Some(res) = collision::air_collide_with_stage(state_context) {
            // Adjust translation
            state_context.translation.0 = res.hit_position - state_context.ecb.get_bottom_point();
            let grounded_common = GroundedStateCommon {
                current_line_id: res.line_id,
            };
            Some(FighterState::Land(LandingState { grounded_common, duration_counter: 0 }))
        } else {
            None
        }
    }
}

pub fn check_input(state_context: &FighterStateContext) -> Option<FighterState> {
    let input_frame = state_context.input.get_last_frame();

    if input_frame.movement.x.abs() < state_context.game_settings.input_common.stick_deadzone {
        return Some(FighterState::Fall(FallState));
    }
    None
}
