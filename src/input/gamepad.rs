use bevy::math::Vec2;
use bevy::prelude::*;

use crate::input::map::GamepadBinding;
use crate::input::{BaseInputMap, FighterInputFrame, InputMapAction};
use crate::math::int::FGi32;
use crate::math::vec::FGVec2;

pub fn sample_gamepad(pad: &Gamepad, input_map: &BaseInputMap) -> FighterInputFrame {
    let mut input_frame = FighterInputFrame::default();
    let mut movement = Vec2::ZERO;

    for gamepad_element in &input_map.gamepad {
        match &gamepad_element.binding {
            GamepadBinding::Button(button) => {
                let pressed = pad.pressed(*button);
                match gamepad_element.action {
                    InputMapAction::Jump => input_frame.jump |= pressed,
                    InputMapAction::Shield => input_frame.shield |= pressed,
                    InputMapAction::Attack => input_frame.attack |= pressed,
                    // Digital buttons driving movement isn't used by the
                    // default bindings (the stick covers that), but stay
                    // consistent with the keyboard backend if one is added.
                    InputMapAction::MovementXDir(sign) => if pressed { movement.x += sign as f32 },
                    InputMapAction::MovementYDir(sign) => if pressed { movement.y += sign as f32 },
                }
            }
            // gamepad does not have a separate axis for left and right, so sign here should be 1
            GamepadBinding::Axis(axis, sign) => {
                let value = pad.get(*axis).unwrap_or(0.0) * (*sign as f32);
                match gamepad_element.action {
                    InputMapAction::MovementXDir(_) => movement.x += value,
                    InputMapAction::MovementYDir(_) => movement.y += value,
                    _ => {}
                    }
            }
        }
    }

    input_frame.movement = FGVec2::new(FGi32::from_num(movement.x), FGi32::from_num(movement.y));

    input_frame
}
