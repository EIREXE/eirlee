//! Keyboard backend: samples the keys into a [`FighterInputFrame`].

use bevy::prelude::*;

use crate::input::{BaseInputMap, FighterInputFrame, InputActionState, InputMapAction};
use crate::math::int::FGi32;

pub fn sample_keyboard(key: &ButtonInput<KeyCode>, input_map: &BaseInputMap) -> FighterInputFrame {
    let mut input_frame = FighterInputFrame::default();
    for keyboard_elment in &input_map.keyboard {
        let action_state = if key.just_pressed(keyboard_elment.key) {
            InputActionState::JustPressed
        } else if key.just_released(keyboard_elment.key) {
            InputActionState::JustReleased
        } else if key.pressed(keyboard_elment.key) {
            InputActionState::Pressed
        } else {
            InputActionState::Released
        };

        let action_strength = match action_state {
            InputActionState::JustPressed | InputActionState::Pressed => FGi32::ONE,
            InputActionState::JustReleased | InputActionState::Released => FGi32::ZERO,
        };

        match keyboard_elment.action {
            InputMapAction::MovementXDir(sign) => {
                input_frame.movement.x += (sign) * action_strength
            }
            InputMapAction::MovementYDir(sign) => {
                input_frame.movement.y += (sign) * action_strength
            }
            InputMapAction::Jump => input_frame.jump = action_strength != FGi32::ZERO,
            InputMapAction::Shield => input_frame.shield = action_strength != FGi32::ZERO,
            InputMapAction::Attack => input_frame.attack = action_strength != FGi32::ZERO,
        }
    }

    input_frame.movement = input_frame.movement.normalize_or_zero();
    input_frame.directional_attack = input_frame.directional_attack.normalize_or_zero();
    input_frame
}
