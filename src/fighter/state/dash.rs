use std::hash::{Hash, Hasher};
use std::time::Duration;

use bevy::prelude::*;

use super::{FighterStateContext, FighterStateImpl, ground, run, wait, walk};
use crate::fighter::animation::AnimKind;
use crate::fighter::state::FighterState;
use crate::fighter::state::fall::FallState;
use crate::fighter::state::ground::{GroundedMotionResult, GroundedStateCommon};
use crate::fighter::FighterFacingDirection;
use crate::input::FighterCommands::{SmashMoveLeft, SmashMoveRight};

#[derive(Debug, Clone)]
pub struct DashState {
    frames_in_dash: u32,
    direction: FighterFacingDirection,
    ground_common: GroundedStateCommon,
}

impl std::hash::Hash for DashState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.direction.hash(state);
        self.frames_in_dash.hash(state);
    }
}

impl DashState {
    pub fn new(direction: FighterFacingDirection, ground_common: GroundedStateCommon) -> Self {
        Self {
            frames_in_dash: 0,
            direction,
            ground_common,
        }
    }
}

impl FighterStateImpl for DashState {
    const NAME: &'static str = "Dash";
    fn check_interrupt(&self, ctx: &FighterStateContext) -> Option<FighterState> {
        let ground = &self.ground_common;
        let attributes = ctx.fighter_attribs;

        // A reverse dash can interrupt before IASA
        if let Some(direction) =
            check_smash_input_with_dir(ctx).filter(|direction| *direction != self.direction)
        {
            return Some(FighterState::Dash(DashState::new(
                direction,
                ground.clone(),
            )));
        }

        if let Some(state) = ground::grounded_movement_common_interrupts(ctx, ground) {
            return Some(state);
        }

        if self.frames_in_dash > attributes.dash_duration {
            return run::check_input(ctx, ground)
                .or_else(|| walk::check_input(ctx, ground))
                .or_else(|| wait::check_input(ctx, ground));
        }

        if self.frames_in_dash > attributes.dash_acceleration_duration {
            return run::check_input(ctx, ground);
        }

        None
    }

    fn on_enter(&mut self, state_context: &mut FighterStateContext) {
        *state_context.facing_direction = self.direction;
        state_context.input.clear_buffer();
        state_context.velocity.x =
            state_context.fighter_attribs.dash_initial_velocity * self.direction.to_sign();

        state_context.animation_transitions.play(
            &mut state_context.animation_player,
            state_context.animations.clips[&AnimKind::Dash],
            Duration::ZERO,
        );

        state_context.input.clear_buffer();
    }

    fn update(&mut self, state_context: &mut FighterStateContext) {
        self.frames_in_dash += 1;

        if  self.frames_in_dash < state_context.fighter_attribs.dash_acceleration_duration {

        }

        let last_frame = state_context.input.get_last_frame();
        let (accel, target_vel) = state_context
            .fighter_attribs
            .get_accel_and_target_dashrun(&last_frame, self.direction.to_sign());
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
        if let GroundedMotionResult::InAir =
            ground::collide_with_stage_grounded(state_context, &mut self.ground_common, true)
        {
            Some(FighterState::Fall(FallState))
        } else {
            None
        }
    }
}

fn check_smash_input_with_dir(state_context: &FighterStateContext) -> Option<FighterFacingDirection> {
    if state_context.input.has_command(SmashMoveLeft) {
        return Some(FighterFacingDirection::Left);
    } else if state_context.input.has_command(SmashMoveRight) {
        return Some(FighterFacingDirection::Right);
    } else {
        None
    }
}

pub fn check_input(
    state_context: &FighterStateContext,
    ground_common: &GroundedStateCommon,
) -> Option<FighterState> {
    check_smash_input_with_dir(state_context)
        .map(|dir| FighterState::Dash(DashState::new(dir, ground_common.clone())))
}
