use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Default)]
pub enum InputActionState {
    JustPressed,
    Pressed,
    JustReleased,
    #[default]
    Released,
}

#[derive(Debug)]
pub enum InputMapAction {
    MovementXDir(i32),
    MovementYDir(i32),
    Jump,
    Shield,
}

#[derive(Debug)]
pub struct KeyboardInputMapElement {
    pub key: KeyCode,
    pub action: InputMapAction,
}

#[derive(Debug)]
pub enum GamepadBinding {
    Button(GamepadButton),
    Axis(GamepadAxis, i32),
}

#[derive(Debug)]
pub struct GamepadInputMapElement {
    pub binding: GamepadBinding,
    pub action: InputMapAction,
}

#[derive(Resource)]
pub struct BaseInputMap {
    pub keyboard: Vec<KeyboardInputMapElement>,
    pub gamepad: Vec<GamepadInputMapElement>,
}
