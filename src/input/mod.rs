//! Input, from physical device to the buffered per-fighter state that the
//! state machine reads.

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use bevy_ggrs::prelude::*;
use bevy_ggrs::{LocalInputs, LocalPlayers};

pub mod buffer;
pub mod control;
pub mod frame;
pub mod gamecube;
pub mod gamepad;
pub mod keyboard;
pub mod map;
pub mod debug;

pub use buffer::{FighterCommands, FighterInput};
pub use frame::FighterInputFrame;
pub use map::{
    BaseInputMap, GamepadBinding, GamepadInputMapElement, InputActionState, InputMapAction,
    KeyboardInputMapElement,
};

use crate::game_settings::GameSettings;
use crate::math::int::FGi32;
use crate::netcode::GGRSCfg;
use crate::schedule::GameplaySet;

/// Owns the input map, the local-input collection that feeds GGRS, and the
/// per-fighter input state derived from the rolled-back inputs.
#[derive(Default)]
pub struct FighterInputPlugin;

impl Plugin for FighterInputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BaseInputMap {
            keyboard: vec![
                // right
                KeyboardInputMapElement {
                    key: KeyCode::KeyD,
                    action: InputMapAction::MovementXDir(1),
                },
                // left
                KeyboardInputMapElement {
                    key: KeyCode::KeyA,
                    action: InputMapAction::MovementXDir(-1),
                },
                // up
                KeyboardInputMapElement {
                    key: KeyCode::KeyW,
                    action: InputMapAction::MovementYDir(1),
                },
                // down
                KeyboardInputMapElement {
                    key: KeyCode::KeyS,
                    action: InputMapAction::MovementYDir(-1),
                },
                // jump
                KeyboardInputMapElement {
                    key: KeyCode::Space,
                    action: InputMapAction::Jump,
                },
                // shield/airdodge
                KeyboardInputMapElement {
                    key: KeyCode::ShiftLeft,
                    action: InputMapAction::Shield,
                },
            ],
            gamepad: vec![
                // left stick
                GamepadInputMapElement {
                    binding: GamepadBinding::Axis(GamepadAxis::LeftStickX, 1),
                    action: InputMapAction::MovementXDir(1),
                },
                GamepadInputMapElement {
                    binding: GamepadBinding::Axis(GamepadAxis::LeftStickY, 1),
                    action: InputMapAction::MovementYDir(1),
                },
                // jump
                GamepadInputMapElement {
                    binding: GamepadBinding::Button(GamepadButton::North),
                    action: InputMapAction::Jump,
                },
                // shield/airdodge -- both shoulder buttons, Melee-style
                GamepadInputMapElement {
                    binding: GamepadBinding::Button(GamepadButton::LeftTrigger),
                    action: InputMapAction::Shield,
                },
                GamepadInputMapElement {
                    binding: GamepadBinding::Button(GamepadButton::RightTrigger),
                    action: InputMapAction::Shield,
                },
            ],
        })
        .add_plugins(gamecube::GamecubeAdapterPlugin)
        .add_systems(ReadInputs, read_local_inputs)
        .add_systems(EguiPrimaryContextPass, debug::input_debug)
        .add_systems(
            GgrsSchedule,
            buffer::postprocess_input.in_set(GameplaySet::Input),
        )
        .rollback_component_with_clone::<FighterInput>();
    }
}

fn preprocess_input_frame(frame: &mut FighterInputFrame, game_settings: &GameSettings) {
    if frame.movement.x.abs() < game_settings.input_common.stick_deadzone {
        frame.movement.x = FGi32::ZERO;
    }
    if frame.movement.y.abs() < game_settings.input_common.stick_deadzone {
        frame.movement.y = FGi32::ZERO;
    }
}

fn read_local_inputs(
    mut commands: Commands,
    key: Res<ButtonInput<KeyCode>>,
    input_map: Res<BaseInputMap>,
    gc_ports: Res<gamecube::GcPorts>,
    gamepads: Query<&Gamepad>,
    local_players: Res<LocalPlayers>,
    game_settings: Res<GameSettings>
) {
    let mut local_inputs = HashMap::new();

    for handle in &local_players.0 {
        let gamepad = gc_ports
            .entities
            .get(*handle)
            .copied()
            .flatten()
            .and_then(|entity| gamepads.get(entity).ok());

        let mut input_frame = match gamepad {
            Some(pad) => gamepad::sample_gamepad(pad, &input_map),
            None => keyboard::sample_keyboard(&key, &input_map),
        };

        // preprocess frame
        preprocess_input_frame(&mut input_frame, &game_settings);

        local_inputs.insert(*handle, input_frame);
    }
    commands.insert_resource(LocalInputs::<GGRSCfg>(local_inputs));
}
