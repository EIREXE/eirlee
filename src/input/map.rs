//! What a physical button means. Device-agnostic: a binding names an
//! [`InputMapAction`], and each device backend (see [`super::keyboard`])
//! decides how to read it.

use bevy::prelude::*;

#[derive(Debug)]
pub enum InputActionState {
    JustPressed,
    Pressed,
    JustReleased,
    Released,
}

#[derive(Debug)]
pub enum InputMapAction {
    MovementXDir(i32),
    MovementYDir(i32),
    Jump,
}

#[derive(Debug)]
pub struct KeyboardInputMapElement {
    pub key: KeyCode,
    pub action: InputMapAction,
}

#[derive(Resource)]
pub struct BaseInputMap {
    pub keyboard: Vec<KeyboardInputMapElement>,
}
