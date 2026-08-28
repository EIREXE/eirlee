use std::time::Duration;

use bevy::prelude::*;

use crate::fighter::{
    animation::AnimKind,
    state::{
        FighterState, FighterStateContext, FighterStateImpl, dash,
        fall::FallState,
        ground::{
            self, GroundedMotionResult, GroundedStateCommon,
            grounded_movement_standstill_common_interrupts,
        },
        wait::WaitState,
        walk,
    },
};

#[derive(Debug, Clone, Hash)]
pub struct LandingState {
    pub grounded_common: GroundedStateCommon,
    pub duration_counter: u32,
}

impl LandingState {
    pub fn create(grounded_common: GroundedStateCommon) -> Self {
        Self {
            grounded_common,
            duration_counter: 0,
        }
    }
}

impl FighterStateImpl for LandingState {
    const NAME: &'static str = "Landing";

    fn check_interrupt(&self, state_context: &mut FighterStateContext) -> Option<FighterState> {
        if state_context.is_current_animation_finished() {
            grounded_movement_standstill_common_interrupts(state_context, &self.grounded_common)
        } else if self.duration_counter >= state_context.fighter_manifest.attributes.landing_iasa {
            dash::check_input(state_context, &self.grounded_common).or_else(|| {
                ground::grounded_movement_common_interrupts(state_context, &self.grounded_common)
                    .or_else(|| walk::check_input(state_context, &self.grounded_common))
            })
        } else {
            None
        }
    }

    fn update(&mut self, state_context: &mut FighterStateContext) {
        self.duration_counter += 1;

        let friction = if state_context.velocity.x.abs()
            > state_context.fighter_manifest.attributes.max_walk_vel
        {
            state_context.fighter_manifest.attributes.ground_friction
                * state_context
                    .game_settings
                    .fighter_common
                    .ground_friction_over_walk_speed_multiplier
        } else {
            state_context.fighter_manifest.attributes.ground_friction
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
        if let GroundedMotionResult::InAir =
            ground::collide_with_stage_grounded(state_context, &mut self.grounded_common, false)
        {
            Some(FighterState::Fall(FallState))
        } else {
            None
        }
    }

    fn on_enter(&mut self, state_context: &mut FighterStateContext) {
        state_context.play_animation(AnimKind::Landing, false);
    }
}
