use bevy::prelude::*;

use super::{FighterState, FighterStateContext, FighterStateTransition, ground, walk};
use crate::fighter::{FighterAttributes, FighterVelocity};
use crate::game_settings::GameSettings;

#[derive(Component, Debug, Clone, Copy)]
pub struct WaitState;

impl FighterState for WaitState {
    fn check_interrupt(
        &self,
        state_context: &FighterStateContext,
    ) -> Option<FighterStateTransition> {
        walk::check_input(state_context)
    }
}

pub fn check_input(state_context: &FighterStateContext) -> Option<FighterStateTransition> {
    let input_frame = state_context.input.get_last_frame();

    if input_frame.movement.x.abs() < state_context.game_settings.input_common.stick_deadzone {
        return Some(FighterStateTransition::Wait);
    }
    None
}

pub fn wait_update(
    query: Query<(&FighterAttributes, &mut FighterVelocity), With<WaitState>>,
    game_settings: Res<GameSettings>,
) {
    for (attribs, mut velocity) in query {
        let friction = if velocity.x.abs() > attribs.max_walk_vel {
            attribs.ground_friction
                * game_settings
                    .figher_common
                    .ground_friction_over_walk_speed_multiplier
        } else {
            attribs.ground_friction
        };
        velocity.x += ground::apply_grounded_friction(friction, velocity.x);
    }
}
