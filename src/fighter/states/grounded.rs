use bevy::log::info;

use crate::{fighter::FighterAttributes, game_settings::GameSettings, input::player::InputFrame};

pub fn apply_grounded_friction(friction: f32, ground_vel: f32) -> f32 {
    if friction.abs() > ground_vel.abs() {
        -ground_vel
    } else {
        (-ground_vel.signum()) * friction.abs()
    }
}

pub fn move_accelerate() {

}

pub fn compute_ground_accel(accel: f32, target_vel: f32, gr_vel: f32, attribs: &FighterAttributes, game_settings: &GameSettings) -> f32 {
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


impl FighterAttributes {
    pub fn get_accel_and_target_dashrun(&self, input: &InputFrame) -> (f32, f32) {
        let accel = input.movement.x * self.stick_dash_accel;
        let accel = accel + input.movement.x.signum() * self.base_dash_accel;
        let target_vel = input.movement.x * self.max_dash_vel;

        info!("accel {:?} target_vel {:?}", accel, target_vel);

        (accel, target_vel)
    }

    pub fn get_accel_and_target_walk(&self, input: &InputFrame) -> (f32, f32) {
        let accel = input.movement.x * self.stick_walk_accel;
        let accel = accel + input.movement.x.signum() * self.base_walk_accel;
        let target_vel = input.movement.x * self.max_walk_vel;

        (accel, target_vel)
    }
}
