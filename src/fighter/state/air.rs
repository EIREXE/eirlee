use crate::fighter::*;

pub fn integrate_gravity(velocity: &mut FighterVelocity, attributes: &FighterAttributes) {
    velocity.y -= attributes.gravity;
    velocity.y = velocity.y.max(-attributes.terminal_velocity);
}

pub fn apply_air_drift(
    velocity: &mut FighterVelocity,
    attribs: &FighterAttributes,
    input: &FighterInput,
) {
    if input.get_last_frame().movement.x.is_zero() {
        if velocity.x.abs() < attribs.air_friction {
            velocity.x = FGi32::ZERO;
        } else {
            let velocity_sign = velocity.x.signum();
            velocity.x += (-velocity_sign) * attribs.air_friction;
        }
    } else {
        let target_velocity =
            input.get_last_frame().movement.x.signum() * attribs.max_air_horizontal_velocity;
        let acceleration =
            input.get_last_frame().movement.x.signum() * attribs.air_acceleration_base;
        let mut acceleration =
            acceleration + input.get_last_frame().movement.x * attribs.air_acceleration_stick;
        let velocity_x = velocity.x;
        if !(velocity_x * acceleration < FGi32::ZERO) {
            if acceleration > FGi32::ZERO {
                if velocity_x + acceleration > target_velocity {
                    acceleration = -attribs.air_friction;
                    if velocity_x + acceleration < target_velocity {
                        acceleration = target_velocity - velocity_x;
                    }
                    if velocity_x + acceleration > attribs.max_air_horizontal_velocity {
                        acceleration = attribs.max_air_horizontal_velocity - velocity_x;
                    }
                }
            } else if velocity_x + acceleration < target_velocity {
                acceleration = attribs.air_friction;
                if velocity_x + acceleration > target_velocity {
                    acceleration = target_velocity - velocity_x;
                }
                if velocity_x + acceleration < -attribs.max_air_horizontal_velocity {
                    acceleration = -attribs.max_air_horizontal_velocity - velocity_x;
                }
            }
        }

        velocity.x += acceleration;
    }
}

pub fn apply_air_motion(
    velocity: &FighterVelocity,
    translation: &mut FighterTranslation,
    prev_translation: &mut FighterPreviousTranslation,
) {
    // Vertical velocity should be 0 on the ground
    prev_translation.0 = translation.0;
    translation.0 += velocity.0;
}
