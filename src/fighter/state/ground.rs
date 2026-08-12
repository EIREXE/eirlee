//! Ground movement math shared by every grounded state. Deliberately free of
//! ECS types so it stays cheap to reason about and to unit-test — it has to be
//! bit-for-bit deterministic for rollback.

use crate::fighter::FighterAttributes;
use crate::game_settings::GameSettings;

pub fn apply_grounded_friction(friction: f32, ground_vel: f32) -> f32 {
    if friction.abs() > ground_vel.abs() {
        -ground_vel
    } else {
        (-ground_vel.signum()) * friction.abs()
    }
}

pub fn move_accelerate() {}

pub fn compute_ground_accel(
    accel: f32,
    target_vel: f32,
    gr_vel: f32,
    attribs: &FighterAttributes,
    game_settings: &GameSettings,
) -> f32 {
    if target_vel == 0.0 {
        apply_grounded_friction(attribs.ground_friction, gr_vel)
    } else {
        let mut accel = accel;
        let ground_max_horizontal_velocity =
            game_settings.figher_common.ground_max_horizontal_velocity;
        if !(gr_vel * accel < 0.0) {
            // accelerating, not reversing
            if accel > 0.0 {
                if gr_vel + accel > target_vel {
                    accel = -attribs.ground_friction;
                    if gr_vel + accel < target_vel {
                        accel = target_vel - gr_vel;
                    }
                    if gr_vel + accel > ground_max_horizontal_velocity {
                        accel = ground_max_horizontal_velocity - gr_vel;
                    }
                }
            } else if gr_vel + accel < target_vel {
                accel = attribs.ground_friction;
                if gr_vel + accel > target_vel {
                    accel = target_vel - gr_vel;
                }
                if gr_vel + accel < -ground_max_horizontal_velocity {
                    accel = -ground_max_horizontal_velocity - gr_vel;
                }
            }
        }
        accel
    }
}
