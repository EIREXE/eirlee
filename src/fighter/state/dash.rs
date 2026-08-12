use std::hash::{Hash, Hasher};

use bevy::prelude::*;

use super::{FighterState, FighterStateContext, FighterStateTransition, ground, run, wait, walk};
use crate::fighter::{FighterAttributes, FighterVelocity};
use crate::game_settings::GameSettings;
use crate::input::{
    FighterCommands::{SmashMoveLeft, SmashMoveRight},
    FighterInput,
};

pub struct DashStateCreator {}

#[derive(Component, Debug, Clone, Copy)]
pub struct DashState {
    frames_in_dash: u32,
    direction: f32,
}

pub fn hash_dash_state(dash_state: &DashState) -> u64 {
    let mut hasher = bevy_ggrs::checksum_hasher();
    dash_state.direction.to_bits().hash(&mut hasher);
    dash_state.frames_in_dash.hash(&mut hasher);
    hasher.finish()
}

impl DashState {
    pub fn new(direction: f32) -> Self {
        Self {
            frames_in_dash: 0,
            direction,
        }
    }
}

impl FighterState for DashState {
    fn check_interrupt(
        &self,
        state_context: &FighterStateContext,
    ) -> Option<FighterStateTransition> {
        if self.frames_in_dash > state_context.fighter_attribs.dash_duration {
            // Dash finish shenanigans
            run::check_input(state_context)
                .or_else(|| walk::check_input(state_context))
                .or_else(|| wait::check_input(state_context))
        } else {
            check_smash_input_with_dir(state_context)
                .filter(|dir| *dir != self.direction)
                .and_then(|dir| Some(FighterStateTransition::Dash(dir)))
        }
    }
}

pub fn start(state_context: &mut FighterStateContext) {
    state_context.input.clear_buffer();
}

fn check_smash_input_with_dir(state_context: &FighterStateContext) -> Option<f32> {
    if state_context.input.has_command(SmashMoveLeft) {
        return Some(-1.0);
    } else if state_context.input.has_command(SmashMoveRight) {
        return Some(1.0);
    } else {
        None
    }
}

pub fn check_input(state_context: &FighterStateContext) -> Option<FighterStateTransition> {
    check_smash_input_with_dir(state_context).map(|dir| FighterStateTransition::Dash(dir))
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

        let last_frame = input.get_last_frame();
        let (accel, target_vel) = attributes.get_accel_and_target_dashrun(&last_frame);
        let accel = ground::compute_ground_accel(
            accel,
            target_vel,
            velocity.x,
            attributes,
            game_settings.as_ref(),
        );
        velocity.x += accel;
    }
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
            velocity.x = attribs.dash_initial_velocity * input.get_last_frame().movement.x.signum();
            input.clear_buffer();
        }
    }
}
