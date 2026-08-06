use crate::{
    fighter::{
        FighterAttributes, FighterVelocity,
        state::{FighterState, FighterStateContext, FighterStateTransition},
        states::{dash, wait},
    },
    game_settings::GameSettings,
    input::player::FighterInput,
};
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct WalkState;

impl FighterState for WalkState {
    fn check_interrupt(
        &self,
        state_context: &FighterStateContext,
    ) -> Option<FighterStateTransition> {
        wait::check_input(state_context).or_else(|| dash::check_input(state_context))
    }
}

pub fn walk_update(
    fighters: Query<(
        &mut FighterVelocity,
        &FighterAttributes,
        &FighterInput,
    ), With<WalkState>>,
    game_settings: Res<GameSettings>,
) {
    for (mut velocity, attributes, input) in fighters {
        if let Some(last_frame) = input.get_last_frame() {
            let accel_mul = 1.0;
            let mut accel = last_frame.movement.x * attributes.stick_walk_accel * accel_mul; // stick-proportional
            accel += (last_frame.movement.x.signum() * attributes.base_walk_accel) * accel_mul; // constant
            let target_vel = last_frame.movement.x * attributes.max_walk_vel * accel_mul;

            if target_vel != 0.0 {
                let mult = velocity.x / target_vel;
                if mult > 0.0 && mult < 1.0 {
                    accel *= (1.0 - mult) * game_settings.figher_common.walk_speed_ease; // 0.5 — linear ease-out toward target
                }
            }

            // Friction mode
            if target_vel == 0.0 {
                let mut out_friction = velocity.x.signum() * (-attributes.ground_friction);
                // apply friction
                if attributes.ground_friction.abs() > velocity.x.abs() {
                    out_friction = -velocity.x;
                }

                accel = out_friction;
            } else {
                let ground_max_horizontal_velocity =
                    game_settings.figher_common.ground_max_horizontal_velocity;
                if !(velocity.x * accel < 0.0) {
                    // accelerating, not reversing
                    if accel > 0.0 {
                        if velocity.x + accel > target_vel {
                            accel = -attributes.ground_friction;
                            if velocity.x + accel < target_vel {
                                accel = target_vel - velocity.x;
                            }
                            if velocity.x + accel > ground_max_horizontal_velocity {
                                accel = ground_max_horizontal_velocity - velocity.x;
                            }
                        }
                    } else if velocity.x + accel < target_vel {
                        accel = attributes.ground_friction;
                        if velocity.x + accel > target_vel {
                            accel = target_vel - velocity.x;
                        }
                        if velocity.x + accel < -ground_max_horizontal_velocity {
                            accel = -ground_max_horizontal_velocity - velocity.x;
                        }
                    }
                }
            }

            velocity.x += accel;
        }
    }
}

pub fn check_input(state_context: &FighterStateContext) -> Option<FighterStateTransition> {
    if let Some(input_frame) = state_context.input.get_last_frame() {
        if input_frame.movement.x.abs() >= state_context.game_settings.input_common.stick_deadzone {
            return Some(FighterStateTransition::Walk);
        }
    }
    None
}
