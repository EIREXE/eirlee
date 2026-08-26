use std::time::Duration;

use bevy::prelude::*;

use crate::{fighter::{
        animation::AnimKind, collision, state::{
            FighterState, FighterStateContext, FighterStateImpl, air, dash, fall::FallState, ground::{self, GroundedMotionResult, GroundedStateCommon}, land::LandingState, wait::WaitState, walk,
        },
    }, input};

#[derive(Debug, Clone, Hash)]
pub struct AirDodgeState {
    pub duration_counter: u32,
}

impl FighterStateImpl for AirDodgeState {
    const NAME: &'static str = "AirDodge";

    fn check_interrupt(&self, state_context: &FighterStateContext) -> Option<FighterState> {
        if self.duration_counter >= state_context.fighter_attribs.air_dodge_duration {
            Some(FighterState::Fall(FallState))
        } else {
            None
        }
    }

    fn update(&mut self, state_context: &mut FighterStateContext) {
        self.duration_counter += 1;

        state_context.velocity.0 *= state_context.fighter_attribs.air_dodge_decay;

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

    fn on_enter(&mut self, state_context: &mut FighterStateContext) {
        state_context.animation_transitions.play(
            state_context.animation_player,
            state_context.animations.clips[&AnimKind::AirDodge],
            Duration::ZERO,
        );

        state_context.velocity.0 = state_context.input.get_last_frame().movement.normalize_or_zero() * state_context.fighter_attribs.air_dodge_velocity;
    }
}

pub fn check_input(state_context: &FighterStateContext) -> Option<FighterState> {
    state_context.input.has_command(input::FighterCommands::Shield).then(||
        FighterState::AirDodge(AirDodgeState { duration_counter: 0 })
    )
}