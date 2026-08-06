use bevy::prelude::*;

use crate::input::player::InputFrame;

use super::player::{FighterInput, BaseInputMap, InputActionState};

pub fn postprocess_input(input_state: Query<&mut FighterInput>) {
    for mut player_input in input_state {
    }
}
pub fn preprocess_keyboard_input(key: Res<ButtonInput<KeyCode>>, input_map: Res<BaseInputMap>, mut input_state: Query<&mut FighterInput>) {
    // Keyboard input is only for player 0
    for mut keeb_input in &mut input_state.iter_mut().take(1) {
        let mut input_frame = InputFrame::default();
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
                InputActionState::JustPressed | InputActionState::Pressed => 1.0f32,
                InputActionState::JustReleased | InputActionState::Released => 0.0f32,
            };

            match keyboard_elment.action {
                super::player::InputMapAction::MovementXDir(sign) => input_frame.movement.x += (sign as f32) * action_strength,
                super::player::InputMapAction::MovementYDir(sign) => input_frame.movement.y += (sign as f32) * action_strength,
                super::player::InputMapAction::Jump => todo!(),
            }
        }

        input_frame.movement = input_frame.movement.normalize_or_zero();
        input_frame.directional_attack = input_frame.directional_attack.normalize_or_zero();
        keeb_input.push_input_frame(input_frame);
    }

}