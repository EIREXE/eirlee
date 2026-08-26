use std::time::Duration;

use bevy::prelude::*;

use crate::{
    fighter::{animation::AnimKind, state::{
        FighterState, FighterStateContext, FighterStateImpl,
        fall::FallState,
        ground::{self, GroundedMotionResult, GroundedStateCommon},
    }}, input::FighterCommands,
};

pub enum JumpType {
    ShortHop,
    LongJump,
    DoubleJump,
}

#[derive(Debug, Clone, Hash)]
pub struct JumpState {}

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
            Some(FighterState::Jump(JumpState {}))
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
    
    fn play_animation(
        &self,
        transitions: &mut AnimationTransitions,
        player: &mut AnimationPlayer,
        anims: &crate::fighter::visual::FighterAnimations,
    ) {
        transitions.play(player, anims.clips[&AnimKind::JumpSquat], Duration::ZERO);
    }
    
    fn on_enter(&mut self, _state_context: &mut FighterStateContext) {}
}

impl FighterStateImpl for JumpState {
    const NAME: &'static str = "Jump";
    fn check_interrupt(&self, state_context: &FighterStateContext) -> Option<FighterState> {
        None
    }

    fn update(&mut self, _state_context: &mut FighterStateContext) {}
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
