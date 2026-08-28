use std::time::Duration;

use bevy::prelude::*;

use crate::{
    fighter::{
        animation::AnimKind,
        collision,
        state::{
            FighterState, FighterStateContext, FighterStateImpl, air,
            air_dodge::{self, AirDodgeState},
            fall::FallState,
            ground::{self, GroundedMotionResult, GroundedStateCommon},
            land::LandingState,
        },
    },
    input::FighterCommands,
    math::{int::FGi32, vec::FGVec2},
};
#[derive(Debug, Hash, Clone, Copy)]
pub enum JumpType {
    ShortHop,
    LongJump,
    DoubleJump,
}

#[derive(Debug, Clone, Hash)]
pub struct JumpState {
    jump_type: JumpType,
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

    fn check_interrupt(&self, state_context: &mut FighterStateContext) -> Option<FighterState> {
        // Special handling for wavedash from jump squat
        if state_context.input.has_command(FighterCommands::Shield) {
            // If aiming sideways and in the deadzone, do a perfect wavedash
            // TODO: Improve this so we don't have to go through the airborne state first
            let mut wavedash_dir = state_context
                .input
                .get_last_frame()
                .movement
                .normalize_or_zero();
            let movement = state_context.input.get_last_frame().movement;
            if movement.y.is_zero() && !movement.x.is_zero() {
                wavedash_dir.y =
                    -state_context.game_settings.input_common.stick_deadzone - FGi32::lit("-0.01");
                wavedash_dir.x = movement.x.signum();
            }

            if wavedash_dir == FGVec2::ZERO {
                wavedash_dir = FGVec2::lit("0.0", "-1.0");
            }

            Some(FighterState::AirDodge(AirDodgeState {
                duration_counter: 0,
                direction: wavedash_dir.normalize_or_zero(),
            }))
        } else if self.duration_counter
            >= state_context.fighter_manifest.attributes.jumpsquat_duration
        {
            info!("Jump {}", state_context.input.get_last_frame().jump);
            let jump_type = if state_context.input.get_last_frame().jump {
                JumpType::LongJump
            } else {
                JumpType::ShortHop
            };
            Some(FighterState::Jump(JumpState { jump_type }))
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
        state_context.play_animation(AnimKind::JumpSquat, false);
        state_context.input.clear_command(FighterCommands::Jump);
    }
}

impl FighterStateImpl for JumpState {
    const NAME: &'static str = "Jump";
    fn check_interrupt(&self, state_context: &mut FighterStateContext) -> Option<FighterState> {
        if state_context.input.has_command(FighterCommands::Jump) {
            Some(FighterState::Jump(JumpState {
                jump_type: JumpType::DoubleJump,
            }))
        } else {
            air_dodge::check_input(state_context)
        }
    }

    fn update(&mut self, state_context: &mut FighterStateContext) {
        air::integrate_gravity(
            state_context.velocity,
            &state_context.fighter_manifest.attributes,
        );
        air::apply_air_drift(
            state_context.velocity,
            &state_context.fighter_manifest.attributes,
            state_context.input,
        );
        air::apply_air_motion(
            state_context.velocity,
            state_context.translation,
            state_context.prev_translation,
        );
    }

    fn on_enter(&mut self, state_context: &mut FighterStateContext) {
        if let JumpType::DoubleJump = self.jump_type {
            state_context.input.clear_command(FighterCommands::Jump);
        }
        let movement_stick_x = state_context.input.get_last_frame().movement.x;
        match self.jump_type {
            JumpType::ShortHop | JumpType::LongJump => {
                if movement_stick_x.is_zero()
                    || movement_stick_x.signum() == state_context.facing_direction.to_sign()
                {
                    state_context.play_animation(AnimKind::JumpForward, false);
                } else {
                    state_context.play_animation(AnimKind::JumpBack, false);
                }
            }
            JumpType::DoubleJump => {
                state_context.play_animation(AnimKind::DoubleJump, false);
            }
        }

        let vertical_vel = match self.jump_type {
            JumpType::ShortHop => {
                state_context
                    .fighter_manifest
                    .attributes
                    .short_hop_vertical_velocity
            }
            JumpType::LongJump => {
                state_context
                    .fighter_manifest
                    .attributes
                    .full_jump_vertical_velocity
            }
            JumpType::DoubleJump => {
                state_context
                    .fighter_manifest
                    .attributes
                    .full_jump_vertical_velocity
                    * state_context
                        .fighter_manifest
                        .attributes
                        .air_jump_multiplier
            }
        };

        match self.jump_type {
            JumpType::ShortHop | JumpType::LongJump => {
                state_context.velocity.x += state_context
                    .fighter_manifest
                    .attributes
                    .jump_horizontal_velocity
                    * movement_stick_x;
            }
            JumpType::DoubleJump => {
                state_context.velocity.x = state_context
                    .fighter_manifest
                    .attributes
                    .air_jump_horizontal_velocity
                    * movement_stick_x;
            }
        };

        state_context.velocity.y = vertical_vel;
        state_context.velocity.x += state_context
            .fighter_manifest
            .attributes
            .jump_horizontal_velocity
            * movement_stick_x;
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
            Some(FighterState::Land(LandingState {
                grounded_common,
                duration_counter: 0,
            }))
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

pub fn check_input_double_jump(state_context: &FighterStateContext) -> Option<FighterState> {
    state_context
        .input
        .has_command(FighterCommands::Jump)
        .then(|| {
            FighterState::Jump(JumpState {
                jump_type: JumpType::DoubleJump,
            })
        })
}
