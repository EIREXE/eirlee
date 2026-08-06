use crate::{
    fighter::state::{FighterState, FighterStateContext, FighterStateTransition},
    input::player::FighterInput,
};
use bevy::prelude::*;

pub struct DashStateCreator {}

#[derive(Component, Debug)]
pub struct DashState {
    frames_in_dash: u32,
}

impl Default for DashState {
    fn default() -> Self {
        Self { frames_in_dash: 0 }
    }
}

impl FighterState for DashState {
    fn check_interrupt(
        &self,
        state_context: &FighterStateContext,
    ) -> Option<FighterStateTransition> {
        if self.frames_in_dash > state_context.fighter_attribs.dash_duration {
            // Dash finish shenanigans
            super::walk::check_input(state_context)
                .or_else(|| super::wait::check_input(state_context))
        } else if let Some(last_frame) = state_context.input.get_last_frame() {
            check_smash_input_with_dir(state_context)
                .filter(|dir| *dir != last_frame.movement.x.signum())
                .and_then(|_| super::dash::check_input(state_context))
        } else {
            None
        }
    }
}

pub fn start(state_context: &mut FighterStateContext) {
    state_context.input.clear_buffer();
}

fn check_smash_input_with_dir(state_context: &FighterStateContext) -> Option<f32> {
    state_context.input.has_smash_x_movement(
        state_context
            .game_settings
            .input_common
            .smash_input_reset_axis_threshold,
        state_context
            .game_settings
            .input_common
            .smash_input_axis_threshold,
        state_context
            .game_settings
            .input_common
            .smash_input_frame_threshold,
    )
}

pub fn dash_update(dash_query: Query<&mut DashState>) {
    for mut dash in dash_query {
        dash.frames_in_dash += 1;
    }
}

pub fn check_input(state_context: &FighterStateContext) -> Option<FighterStateTransition> {
    check_smash_input_with_dir(state_context).map(|_| FighterStateTransition::Dash)
}

pub fn on_insert_dash(
    add: On<Insert, DashState>,
    query: Query<(Entity, &mut FighterInput), With<DashState>>,
) {
    for (entity, mut input) in query {
        if entity == add.entity {
            input.clear_buffer();
        }
    }
}
