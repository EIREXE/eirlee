//! Keyboard backend: samples the keys into the local input frames GGRS sends
//! over the wire.

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_ggrs::{LocalInputs, LocalPlayers};

use crate::input::{BaseInputMap, FighterInputFrame, InputActionState, InputMapAction};
use crate::math::int::FGi32;
use crate::netcode::GGRSCfg;

pub fn preprocess_keyboard_input(
    mut commands: Commands,
    key: Res<ButtonInput<KeyCode>>,
    input_map: Res<BaseInputMap>,
    local_players: Res<LocalPlayers>,
) {
    // Keyboard input is only for player 0

    let mut local_inputs = HashMap::new();

    for handle in &local_players.0 {
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
            }
        }

        input_frame.movement = input_frame.movement.normalize_or_zero();
        input_frame.directional_attack = input_frame.directional_attack.normalize_or_zero();
        local_inputs.insert(*handle, input_frame);
    }
    commands.insert_resource(LocalInputs::<GGRSCfg>(local_inputs));
}
