use bevy::prelude::*;

use crate::input::{
    GamepadBinding, LocalInputSource, MenuInputMap, MenuInputMapAction,
};

#[derive(Default, Clone)]
pub struct MenuInput {
    pub movement: Vec2,
    pub accept: bool,
    pub back: bool,
    pub start: bool,
}

#[derive(Resource, Default)]
pub struct MenuInputState {
    pub inputs: Vec<(LocalInputSource, MenuInput)>,
}

impl MenuInputState {
    pub fn get(&self, source: &LocalInputSource) -> MenuInput {
        self.inputs.iter().find(|(candidate, _)| candidate == source).map(|(_, input)| input.clone()).unwrap_or_default()
    }

    pub fn aggregate(&self) -> MenuInput {
        self.inputs.iter().fold(MenuInput::default(), |mut result, (_, input)| {
            result.movement += input.movement;
            result.accept |= input.accept;
            result.back |= input.back;
            result.start |= input.start;
            result
        })
    }
}

impl MenuInput {
    pub fn accumulate(&mut self, other: &Self) {
        self.movement += other.movement;
        self.accept |= other.accept;
        self.back |= other.back;
        self.start |= other.start;
        self.movement = self.movement.clamp_length_max(1.0);
    }
}

fn keyboard_input(key: &ButtonInput<KeyCode>, map: &MenuInputMap) -> MenuInput {
    let mut input = MenuInput::default();
    for binding in &map.keyboard {
        let pressed = key.pressed(binding.key);
        match binding.action {
            MenuInputMapAction::MovementXDir(sign) => {
                input.movement.x += if pressed { sign as f32 } else { 0.0 };
            }
            MenuInputMapAction::MovementYDir(sign) => {
                input.movement.y += if pressed { sign as f32 } else { 0.0 };
            }
            MenuInputMapAction::Accept => input.accept |= pressed,
            MenuInputMapAction::Back => input.back |= pressed,
            MenuInputMapAction::Start => input.start |= pressed,
        }
    }
    input.accept = map.keyboard.iter().any(|binding| matches!(binding.action, MenuInputMapAction::Accept) && key.pressed(binding.key));
    input.back = map.keyboard.iter().any(|binding| matches!(binding.action, MenuInputMapAction::Back) && key.pressed(binding.key));
    input.start = map.keyboard.iter().any(|binding| matches!(binding.action, MenuInputMapAction::Start) && key.pressed(binding.key));
    input.movement = input.movement.normalize_or_zero();
    input
}

fn gamepad_input(pad: &Gamepad, map: &MenuInputMap, deadzone: f32) -> MenuInput {
    let mut input = MenuInput::default();
    for binding in &map.gamepad {
        let (value, just_pressed) = match binding.binding {
            GamepadBinding::Button(button) => (if pad.pressed(button) { 1.0 } else { 0.0 }, pad.just_pressed(button)),
            GamepadBinding::Axis(axis, sign) => (pad.get(axis).unwrap_or_default() * sign as f32, false),
        };
        match binding.action {
            MenuInputMapAction::MovementXDir(sign) => {
                input.movement.x += value * sign as f32;
            }
            MenuInputMapAction::MovementYDir(sign) => {
                input.movement.y += value * sign as f32;
            }
            MenuInputMapAction::Accept => { input.accept |= value > 0.5; input.accept |= just_pressed; }
            MenuInputMapAction::Back => { input.back |= value > 0.5; input.back |= just_pressed; }
            MenuInputMapAction::Start => { input.start |= value > 0.5; input.start |= just_pressed; }
        }
    }
    if input.movement.x.abs() < deadzone { input.movement.x = 0.0; }
    if input.movement.y.abs() < deadzone { input.movement.y = 0.0; }
    input.movement = input.movement.clamp_length_max(1.0);
    input
}

pub fn sample_menu_inputs(
    key: Res<ButtonInput<KeyCode>>,
    map: Res<MenuInputMap>,
    pads: Query<(Entity, &Gamepad)>,
    mut state: ResMut<MenuInputState>,
) {
    state.inputs.clear();
    let deadzone = 0.275;
    state.inputs.push((LocalInputSource::Keyboard, keyboard_input(&key, &map)));
    for (entity, pad) in pads.iter() {
        state.inputs.push((LocalInputSource::Gamepad(entity), gamepad_input(pad, &map, deadzone)));
    }
}
