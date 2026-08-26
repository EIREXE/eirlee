use std::time::Duration;

use bevy::prelude::*;

use crate::{
    fighter::{
        animation::AnimKind, collision, state::{
            FighterState, FighterStateContext, FighterStateImpl, air, air_dodge, fall::FallState, ground::{self, GroundedMotionResult, GroundedStateCommon}, land::LandingState,
        },
    }, input::FighterCommands,
};
#[derive(Debug, Hash, Clone, Copy)]
pub enum JumpType {
    ShortHop,
    LongJump,
    DoubleJump,
}

#[derive(Debug, Clone, Hash)]
pub struct JumpState {
    jump_type: JumpType
}

#[derive(Debug, Clone, Hash)]
pub struct JumpSquatState {
    pub grounded_common: GroundedStateCommon,
    pub duration_counter: u32,
}

impl JumpSquatState {
    pub fn create(grounded_common: GroundedStateCommon) -> Self {
        Self {
            grounded_common,
            duration_counter: 0,
        }
    }
}

impl FighterStateImpl for JumpSquatState {
    const NAME: &'static str = "JumpSquat";

    fn check_interrupt(&self, state_context: &FighterStateContext) -> Option<FighterState> {
        if self.duration_counter >= state_context.fighter_attribs.jumpsquat_duration {
            info!("Jump {}", state_context.input.get_last_frame().jump);
            let jump_type = if state_context.input.get_last_frame().jump {JumpType::LongJump} else {JumpType::ShortHop};
            Some(FighterState::Jump(JumpState { jump_type}))
        } else {
            None
        }
    }

    fn update(&mut self, state_context: &mut FighterStateContext) {
        self.duration_counter += 1;

        let friction =
            if state_context.velocity.x.abs() > state_context.fighter_attribs.max_walk_vel {
                state_context.fighter_attribs.ground_friction
                    * state_context
                        .game_settings
                        .fighter_common
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
        if let GroundedMotionResult::InAir =
            ground::collide_with_stage_grounded(state_context, &mut self.grounded_common, false)
        {
            Some(FighterState::Fall(FallState))
        } else {
            None
        }
    }

    fn on_enter(&mut self, state_context: &mut FighterStateContext) {
        state_context.animation_transitions.play(
            state_context.animation_player,
            state_context.animations.clips[&AnimKind::JumpSquat],
            Duration::ZERO,
        );
    }
}

impl FighterStateImpl for JumpState {
    const NAME: &'static str = "Jump";
    fn check_interrupt(&self, state_context: &FighterStateContext) -> Option<FighterState> {
        air_dodge::check_input(state_context)
    }

    fn update(&mut self, state_context: &mut FighterStateContext) {
        info!("TRF1 {}", state_context.translation.0);
        air::integrate_gravity(state_context.velocity, state_context.fighter_attribs);
        air::apply_air_drift(state_context.velocity, state_context.fighter_attribs, state_context.input);
        air::apply_air_motion(
            state_context.velocity,
            state_context.translation,
            state_context.prev_translation,
        );
        info!("TRF12 {}", state_context.translation.0);
    }

    fn on_enter(&mut self, state_context: &mut FighterStateContext) {
        
        let movement_stick_x = state_context.input.get_last_frame().movement.x;
        if movement_stick_x.is_zero() || movement_stick_x.signum() == state_context.facing_direction.to_sign() {
            state_context.animation_transitions.play(
                state_context.animation_player,
                state_context.animations.clips[&AnimKind::JumpForward],
                Duration::ZERO,
            );
        } else {
            state_context.animation_transitions.play(
                state_context.animation_player,
                state_context.animations.clips[&AnimKind::JumpBack],
                Duration::ZERO,
            );
        }

        let vertical_vel = match self.jump_type {
            JumpType::ShortHop => state_context.fighter_attribs.short_hop_vertical_velocity,
            JumpType::LongJump => state_context.fighter_attribs.full_jump_vertical_velocity,
            JumpType::DoubleJump => todo!(),
        };

        state_context.velocity.y = vertical_vel;
        state_context.velocity.x += state_context.fighter_attribs.jump_horizontal_velocity * movement_stick_x;
        state_context.input.clear_buffer();
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

pub fn check_input(
    state_context: &FighterStateContext,
    ground_common: &GroundedStateCommon,
) -> Option<FighterState> {
    state_context
        .input
        .has_command(FighterCommands::Jump)
        .then(|| FighterState::JumpSquat(JumpSquatState::create(ground_common.clone())))
}
