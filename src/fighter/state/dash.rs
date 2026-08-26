use std::hash::{Hash, Hasher};
use std::time::Duration;

use bevy::prelude::*;

use super::{FighterStateImpl, FighterStateContext, ground, run, wait, walk};
use crate::fighter::animation::AnimKind;
use crate::fighter::state::fall::FallState;
use crate::fighter::state::ground::{GroundedMotionResult, GroundedStateCommon};
use crate::fighter::state::wait::WaitState;
use crate::fighter::state::{FighterState};
use crate::fighter::{FighterAttributes, FighterVelocity, collision};
use crate::game_settings::GameSettings;
use crate::input::{
    FighterCommands::{SmashMoveLeft, SmashMoveRight},
    FighterInput,
};
use crate::math::int::FGi32;

#[derive(Debug, Clone)]
pub struct DashState {
    frames_in_dash: u32,
    direction: FGi32,
    ground_common: GroundedStateCommon
}

impl std::hash::Hash for DashState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.direction.to_bits().hash(state);
        self.frames_in_dash.hash(state);
    }
}

impl DashState {
    pub fn new(direction: FGi32, ground_common: GroundedStateCommon) -> Self {
        Self {
            frames_in_dash: 0,
            direction,
            ground_common
        }
    }
}

impl FighterStateImpl for DashState {
    const NAME: &'static str = "Dash";
    fn check_interrupt(
        &self,
        state_context: &FighterStateContext,
    ) -> Option<FighterState> {
        if self.frames_in_dash > state_context.fighter_attribs.dash_duration {
            run::check_input(state_context, &self.ground_common)
                .or_else(|| walk::check_input(state_context, &self.ground_common))
                .or_else(|| wait::check_input(state_context, &self.ground_common))
        } else if self.frames_in_dash > state_context.fighter_attribs.dash_acceleration_duration {
            // Dash finish shenanigans
            run::check_input(state_context, &self.ground_common)
                .or_else(|| walk::check_input(state_context, &self.ground_common))
        } else {
            check_smash_input_with_dir(state_context)
                .filter(|dir| *dir != self.direction)
                .and_then(|dir| Some(FighterState::Dash(DashState::new(dir, self.ground_common.clone()))))
        }
    }
    
    fn on_enter(&mut self, state_context: &mut FighterStateContext) {
        state_context.input.clear_buffer();
        state_context.velocity.x = state_context.fighter_attribs.dash_initial_velocity * self.direction;

        state_context.animation_transitions.play(
            &mut state_context.animation_player,
            state_context.animations.clips[&AnimKind::Dash],
            Duration::ZERO,
        );
    }
    
    fn update(&mut self, state_context: &mut FighterStateContext) {
        self.frames_in_dash += 1;

        info!("{}", self.frames_in_dash);

        let last_frame = state_context.input.get_last_frame();
        let (accel, target_vel) = state_context.fighter_attribs.get_accel_and_target_dashrun(&last_frame, self.direction);
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
            &mut self.ground_common,
            true
        ) {
            Some(FighterState::Fall(FallState))
        } else {
            None
        }
    }
}

fn check_smash_input_with_dir(state_context: &FighterStateContext) -> Option<FGi32> {
    if state_context.input.has_command(SmashMoveLeft) {
        return Some(FGi32::NEG_ONE);
    } else if state_context.input.has_command(SmashMoveRight) {
        return Some(FGi32::ONE);
    } else {
        None
    }
}

pub fn check_input(state_context: &FighterStateContext, ground_common: &GroundedStateCommon) -> Option<FighterState> {
    check_smash_input_with_dir(state_context).map(|dir| FighterState::Dash(DashState::new(dir, ground_common.clone())))
}
