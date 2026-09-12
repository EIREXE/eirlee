//! Input, from physical device to the buffered per-fighter state that the
//! state machine reads.

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_ggrs::prelude::*;
use bevy_ggrs::{LocalInputs, LocalPlayers};

pub mod buffer;
pub mod control;
pub mod frame;
pub mod gamecube;
pub mod gamepad;
pub mod keyboard;
pub mod map;

pub use buffer::{FighterCommands, FighterInput};
pub use frame::FighterInputFrame;
pub use map::{
    BaseInputMap, GamepadBinding, GamepadInputMapElement, InputActionState, InputMapAction,
    KeyboardInputMapElement, MenuGamepadInputMapElement, MenuInputMap, MenuInputMapAction,
    MenuKeyboardInputMapElement,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalInputSource {
    Keyboard,
    Gamepad(Entity),
}

/// Maps an input source to a player index
#[derive(Resource, Clone, Debug, Default, Deref, DerefMut)]
pub struct LocalInputAssignments(pub Vec<(usize, LocalInputSource)>);

impl LocalInputAssignments {
    pub fn get_free_player_index(&self) -> Option<usize> {
        for i in 0..16 {
            if let None = self.iter().find(|(idx, _)| *idx == i) {
                return Some(i);
            }
        }
        None
    }

    pub fn get_from_slot(&self, slot: usize) -> Option<&LocalInputSource> {
        self.0
            .iter()
            .find(|(c_slot, _)| slot == *c_slot)
            .map(|(_, source)| source)
    }
}

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
                // attack
                KeyboardInputMapElement {
                    key: KeyCode::KeyJ,
                    action: InputMapAction::Attack,
                },
                // Menu
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
                // attack
                GamepadInputMapElement {
                    binding: GamepadBinding::Button(GamepadButton::East),
                    action: InputMapAction::Attack,
                },
            ],
        })
        .insert_resource(MenuInputMap {
            keyboard: vec![
                MenuKeyboardInputMapElement {
                    key: KeyCode::ArrowUp,
                    action: MenuInputMapAction::MovementYDir(1),
                },
                MenuKeyboardInputMapElement {
                    key: KeyCode::ArrowDown,
                    action: MenuInputMapAction::MovementYDir(-1),
                },
                MenuKeyboardInputMapElement {
                    key: KeyCode::ArrowLeft,
                    action: MenuInputMapAction::MovementXDir(-1),
                },
                MenuKeyboardInputMapElement {
                    key: KeyCode::ArrowRight,
                    action: MenuInputMapAction::MovementXDir(1),
                },
                MenuKeyboardInputMapElement {
                    key: KeyCode::Enter,
                    action: MenuInputMapAction::Accept,
                },
                MenuKeyboardInputMapElement {
                    key: KeyCode::Escape,
                    action: MenuInputMapAction::Back,
                },
                MenuKeyboardInputMapElement {
                    key: KeyCode::Space,
                    action: MenuInputMapAction::Start,
                },
                MenuKeyboardInputMapElement {
                    key: KeyCode::KeyW,
                    action: MenuInputMapAction::MovementYDir(1),
                },
                MenuKeyboardInputMapElement {
                    key: KeyCode::KeyS,
                    action: MenuInputMapAction::MovementYDir(-1),
                },
                MenuKeyboardInputMapElement {
                    key: KeyCode::KeyA,
                    action: MenuInputMapAction::MovementXDir(-1),
                },
                MenuKeyboardInputMapElement {
                    key: KeyCode::KeyD,
                    action: MenuInputMapAction::MovementXDir(1),
                },
            ],
            gamepad: vec![
                MenuGamepadInputMapElement {
                    binding: GamepadBinding::Button(GamepadButton::DPadUp),
                    action: MenuInputMapAction::MovementYDir(1),
                },
                MenuGamepadInputMapElement {
                    binding: GamepadBinding::Button(GamepadButton::DPadDown),
                    action: MenuInputMapAction::MovementYDir(-1),
                },
                MenuGamepadInputMapElement {
                    binding: GamepadBinding::Button(GamepadButton::DPadLeft),
                    action: MenuInputMapAction::MovementXDir(-1),
                },
                MenuGamepadInputMapElement {
                    binding: GamepadBinding::Button(GamepadButton::DPadRight),
                    action: MenuInputMapAction::MovementXDir(1),
                },
                MenuGamepadInputMapElement {
                    binding: GamepadBinding::Axis(GamepadAxis::LeftStickX, 1),
                    action: MenuInputMapAction::MovementXDir(1),
                },
                MenuGamepadInputMapElement {
                    binding: GamepadBinding::Axis(GamepadAxis::LeftStickY, 1),
                    action: MenuInputMapAction::MovementYDir(1),
                },
                MenuGamepadInputMapElement {
                    binding: GamepadBinding::Button(GamepadButton::South),
                    action: MenuInputMapAction::Accept,
                },
                MenuGamepadInputMapElement {
                    binding: GamepadBinding::Button(GamepadButton::West),
                    action: MenuInputMapAction::Back,
                },
                MenuGamepadInputMapElement {
                    binding: GamepadBinding::Button(GamepadButton::Start),
                    action: MenuInputMapAction::Start,
                },
            ],
        })
        .init_resource::<LocalInputAssignments>()
        .add_plugins(gamecube::GamecubeAdapterPlugin)
        .add_systems(ReadInputs, read_local_inputs)
        .add_systems(
            GgrsSchedule,
            buffer::postprocess_input.in_set(GameplaySet::Input),
        )
        .rollback_component_with_clone::<FighterInput>()
        .checksum_component_with_hash::<FighterInput>();
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
    assignments: Res<LocalInputAssignments>,
    local_players: Res<LocalPlayers>,
    game_settings: Res<GameSettings>,
) {
    let mut local_inputs = HashMap::new();

    for handle in &local_players.0 {
        let assigned = assignments
            .0
            .iter()
            .find(|(player_handle, _)| player_handle == handle);
        let gamepad = match assigned {
            Some((_, LocalInputSource::Gamepad(entity))) => gamepads.get(*entity).ok(),
            Some((_, LocalInputSource::Keyboard)) => None,
            None => gc_ports
                .entities
                .get(*handle)
                .copied()
                .flatten()
                .and_then(|entity| gamepads.get(entity).ok()),
        };

        let mut input_frame = match assigned {
            Some((_, LocalInputSource::Keyboard)) => keyboard::sample_keyboard(&key, &input_map),
            Some((_, LocalInputSource::Gamepad(_))) => gamepad
                .map(|pad| gamepad::sample_gamepad(pad, &input_map))
                .unwrap_or_default(),
            None => match gamepad {
                Some(pad) => gamepad::sample_gamepad(pad, &input_map),
                None => keyboard::sample_keyboard(&key, &input_map),
            },
        };

        // preprocess frame
        preprocess_input_frame(&mut input_frame, &game_settings);

        local_inputs.insert(*handle, input_frame);
    }
    commands.insert_resource(LocalInputs::<GGRSCfg>(local_inputs));
}
