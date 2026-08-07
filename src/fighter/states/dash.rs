use crate::{
    fighter::{
        FighterAttributes, FighterVelocity,
        state::{FighterState, FighterStateContext, FighterStateTransition},
        states::grounded,
    },
    game_settings::GameSettings,
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
            super::run::check_input(state_context)
                .or_else(|| super::walk::check_input(state_context))
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

pub fn dash_update(
    dash_query: Query<(
        &mut DashState,
        &FighterAttributes,
        &FighterInput,
        &mut FighterVelocity,
    )>,
    game_settings: Res<GameSettings>,
) {
    for (mut dash, attributes, input, mut velocity) in dash_query {
        dash.frames_in_dash += 1;

        if let Some(last_frame) = input.get_last_frame() {
            let (accel, target_vel) = attributes.get_accel_and_target_dashrun(last_frame);
            let accel = grounded::compute_ground_accel(
                accel,
                target_vel,
                velocity.x,
                attributes,
                game_settings.as_ref(),
            );
            velocity.x += accel;
        }
    }
}

pub fn check_input(state_context: &FighterStateContext) -> Option<FighterStateTransition> {
    check_smash_input_with_dir(state_context).map(|_| FighterStateTransition::Dash)
}

pub fn on_insert_dash(
    add: On<Insert, DashState>,
    query: Query<
        (
            Entity,
            &mut FighterInput,
            &FighterAttributes,
            &mut FighterVelocity,
        ),
        With<DashState>,
    >,
) {
    for (entity, mut input, attribs, mut velocity) in query {
        if entity == add.entity {
            if let Some(last_frame) = input.get_last_frame() {
                velocity.x = attribs.dash_initial_velocity * last_frame.movement.x.signum();
            }
            input.clear_buffer();
        }
    }
}
