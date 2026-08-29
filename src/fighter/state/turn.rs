use super::{FighterStateContext, FighterStateImpl};
use crate::fighter::animation::AnimKind;
use crate::fighter::state::fall::FallState;
use crate::fighter::state::ground::{self, GroundedMotionResult, GroundedStateCommon};
use crate::fighter::state::{dash, jump, wait, walk, FighterState};

#[derive(Debug, Clone, Hash)]
pub struct TurnState {
    grounded_common: GroundedStateCommon,
}

impl FighterStateImpl for TurnState {
    const NAME: &'static str = "Turn";
    fn check_interrupt(&self, state_context: &mut FighterStateContext) -> Option<FighterState> {
        // State chain that results in the turn completing
        if let Some(state) = jump::check_input(state_context, &self.grounded_common) {
            Some(state)
        } else if state_context.is_current_animation_finished() {
            if let Some(state) = dash::check_input(state_context, &self.grounded_common) {
                Some(state)
            } else if let Some(state) = walk::check_input(state_context, &self.grounded_common) {
                Some(state)
            } else if let Some(state) = wait::check_input(state_context, &self.grounded_common) {
                Some(state)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn on_enter(&mut self, state_context: &mut super::FighterStateContext) {
        state_context.play_animation(AnimKind::Turn, false);
        *state_context.facing_direction = state_context.facing_direction.reverse();
    }

    fn update(&mut self, state_context: &mut super::FighterStateContext) {
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
}

pub fn check_input(
    state_context: &FighterStateContext,
    grounded_common: &GroundedStateCommon,
) -> Option<FighterState> {
    let input_frame = state_context.input.get_last_frame();
    let smash_input_frame_threshold = state_context
        .game_settings
        .input_common
        .smash_input_frame_threshold;
    if !input_frame.movement.x.is_zero()
        && input_frame.movement.x.signum() != state_context.facing_direction.to_sign()
        && state_context.input.get_frames_in_smash_move_deadzone() > smash_input_frame_threshold
    {
        return Some(FighterState::Turn(TurnState {
            grounded_common: grounded_common.clone(),
        }));
    }
    None
}
