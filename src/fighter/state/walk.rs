use super::{FighterStateContext, FighterStateImpl, dash, ground, wait};
use crate::fighter::animation::AnimKind;
use crate::fighter::state::FighterState;
use crate::fighter::state::fall::FallState;
use crate::fighter::state::ground::{GroundedMotionResult, GroundedStateCommon};
use crate::fighter::state::turn;

#[derive(Debug, Clone, Hash)]
pub struct WalkState {
    pub grounded_common: GroundedStateCommon,
}

impl FighterStateImpl for WalkState {
    const NAME: &'static str = "Walk";
    fn check_interrupt(&self, state_context: &mut FighterStateContext) -> Option<FighterState> {
        if let Some(state) = turn::check_input(state_context, &self.grounded_common) {
            return Some(state);
        } else if let Some(state) =
            ground::grounded_movement_common_interrupts(state_context, &self.grounded_common)
        {
            return Some(state);
        }
        wait::check_input(state_context, &self.grounded_common)
            .or_else(|| dash::check_input(state_context, &self.grounded_common))
    }

    fn on_enter(&mut self, state_context: &mut FighterStateContext) {
        state_context.play_animation(AnimKind::Walk, true);
    }

    fn update(&mut self, state_context: &mut FighterStateContext) {
        let last_frame = state_context.input.get_last_frame();
        let (accel, target_vel) = state_context
            .fighter_manifest
            .attributes
            .get_accel_and_target_walk(&last_frame);

        let accel = ground::compute_ground_accel(
            accel,
            target_vel,
            state_context.velocity.x,
            &state_context.fighter_manifest.attributes,
            &state_context.game_settings,
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
    ground_state_common: &GroundedStateCommon,
) -> Option<FighterState> {
    let input_frame = state_context.input.get_last_frame();

    if !input_frame.movement.x.is_zero() {
        return Some(FighterState::Walk(WalkState {
            grounded_common: ground_state_common.clone(),
        }));
    }

    None
}
