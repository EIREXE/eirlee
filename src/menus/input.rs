use std::collections::HashMap;

use bevy::{input::gamepad::GamepadInput, prelude::*};

use crate::input::{GamepadBinding, LocalInputSource, MenuInputMap, MenuInputMapAction};

#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub enum MenuInputActionState {
    #[default]
    Released,
    Pressed,
    JustReleased,
    JustPressed,
}

#[derive(Default, Clone)]
pub struct MenuJoystickState(HashMap<GamepadInput, f32>);

impl MenuJoystickState {
    pub fn get(&self, input: impl Into<GamepadInput>) -> f32 {
        self.0.get(&input.into()).map(|d| *d).unwrap_or_default()
    }

    pub fn from_gamepad(gamepad: &Gamepad) -> Self {
        let d = gamepad
            .analog()
            .all_axes_and_values()
            .map(|(axis, val)| (*axis, val))
            .collect::<HashMap<_, _>>();
        Self(d)
    }
}

impl MenuInputActionState {
    pub fn is_pressed(&self) -> bool {
        matches!(self, MenuInputActionState::JustPressed)
            || matches!(self, MenuInputActionState::Pressed)
    }

    pub fn is_just_pressed(&self) -> bool {
        matches!(self, MenuInputActionState::JustPressed)
    }

    pub fn accumulate(&mut self, other: &Self) {
        if *self < *other {
            *self = *other;
        }
    }
}

#[derive(Default, Clone)]
pub struct MenuInput {
    pub movement: Vec2,
    pub movement_digital_up: MenuInputActionState,
    pub movement_digital_down: MenuInputActionState,
    pub movement_digital_left: MenuInputActionState,
    pub movement_digital_right: MenuInputActionState,
    pub accept: MenuInputActionState,
    pub back: MenuInputActionState,
    pub start: MenuInputActionState,
}

#[derive(Resource, Default)]
pub struct MenuInputState {
    pub inputs: Vec<(LocalInputSource, MenuInput)>,
    pub prev_frame_joystick_state: Vec<(LocalInputSource, MenuJoystickState)>,
}

impl MenuInputState {
    pub fn get(&self, source: &LocalInputSource) -> MenuInput {
        self.inputs
            .iter()
            .find(|(candidate, _)| candidate == source)
            .map(|(_, input)| input.clone())
            .unwrap_or_default()
    }

    pub fn aggregate(&self) -> MenuInput {
        self.inputs
            .iter()
            .fold(MenuInput::default(), |mut result, (_, input)| {
                result.accumulate(input);
                result
            })
    }
}

impl MenuInput {
    pub fn accumulate(&mut self, other: &Self) {
        self.movement += other.movement;
        self.accept.accumulate(&other.accept);
        self.back.accumulate(&other.back);
        self.start.accumulate(&other.start);
        self.movement_digital_up.accumulate(&other.movement_digital_up);
        self.movement_digital_down.accumulate(&other.movement_digital_down);
        self.movement_digital_left.accumulate(&other.movement_digital_left);
        self.movement_digital_right.accumulate(&other.movement_digital_right);
        self.movement = self.movement.clamp_length_max(1.0);
    }
}

fn keyboard_input(key: &ButtonInput<KeyCode>, map: &MenuInputMap) -> MenuInput {
    let mut input = MenuInput::default();

    let key_to_state = |key_code| {
        if key.just_pressed(key_code) {
            MenuInputActionState::JustPressed
        } else if key.pressed(key_code) {
            MenuInputActionState::Pressed
        } else if key.just_released(key_code) {
            MenuInputActionState::JustReleased
        } else {
            MenuInputActionState::Released
        }
    };

    for binding in &map.keyboard {
        match binding.action {
            MenuInputMapAction::MovementXDir(sign) => {
                let state = key_to_state(binding.key);
                input.movement.x += if state.is_pressed() { sign as f32 } else { 0.0 };

                if sign.is_positive() {
                    input.movement_digital_right.accumulate(&state);
                } else if sign.is_negative() {
                    input.movement_digital_left.accumulate(&state);
                }
            }
            MenuInputMapAction::MovementYDir(sign) => {
                let state = key_to_state(binding.key);
                input.movement.y += if state.is_pressed() { sign as f32 } else { 0.0 };

                if sign.is_positive() {
                    input.movement_digital_up.accumulate(&state);
                } else if sign.is_negative() {
                    input.movement_digital_down.accumulate(&state);
                }
            }
            MenuInputMapAction::Accept => input.accept.accumulate(&key_to_state(binding.key)),
            MenuInputMapAction::Back => input.back.accumulate(&key_to_state(binding.key)),
            MenuInputMapAction::Start => input.start.accumulate(&key_to_state(binding.key)),
        }
    }

    input.movement = input.movement.normalize_or_zero();
    input
}

fn gamepad_input(
    pad: &Gamepad,
    map: &MenuInputMap,
    deadzone: f32,
    prev_frame: &MenuJoystickState,
) -> MenuInput {
    let mut input = MenuInput::default();

    let button_to_state = |button| {
        if pad.just_pressed(button) {
            MenuInputActionState::JustPressed
        } else if pad.pressed(button) {
            MenuInputActionState::Pressed
        } else if pad.just_released(button) {
            MenuInputActionState::JustReleased
        } else {
            MenuInputActionState::Released
        }
    };

    for binding in &map.gamepad {
        let (value, status) = match binding.binding {
            GamepadBinding::Button(button) => {
                let state = button_to_state(button);

                (if state.is_pressed() { 1.0 } else { 0.0 }, state)
            }
            GamepadBinding::Axis(axis, sign) => {
                let prev_frame = prev_frame.get(axis);
                let curr_frame = pad.get(axis).unwrap_or_default();

                let prev_frame_in_deadzone = prev_frame.abs() >= deadzone;
                let curr_frame_in_deadzone = curr_frame.abs() >= deadzone;

                let status = if prev_frame_in_deadzone && curr_frame_in_deadzone {
                    if prev_frame.signum() != curr_frame.signum() {
                        MenuInputActionState::JustPressed
                    } else {
                        MenuInputActionState::Pressed
                    }
                } else if curr_frame_in_deadzone && !prev_frame_in_deadzone {
                    MenuInputActionState::JustPressed
                } else if !curr_frame_in_deadzone && prev_frame_in_deadzone {
                    MenuInputActionState::JustReleased
                } else {
                    MenuInputActionState::Released
                };

                (curr_frame * sign as f32, status)
            }
        };
        match binding.action {
            MenuInputMapAction::MovementXDir(sign) => {
                input.movement.x += value * sign as f32;

                if sign > 0 {
                    input.movement_digital_right.accumulate(&status);
                } else {
                    input.movement_digital_left.accumulate(&status);
                }
            }
            MenuInputMapAction::MovementYDir(sign) => {
                input.movement.y += value * sign as f32;
                if sign > 0 {
                    input.movement_digital_up.accumulate(&status);
                } else {
                    input.movement_digital_down.accumulate(&status);
                }
            }
            MenuInputMapAction::Accept => {
                input.accept.accumulate(&status);
            }
            MenuInputMapAction::Back => {
                input.accept.accumulate(&status);
            }
            MenuInputMapAction::Start => {
                input.accept.accumulate(&status);
            }
        }
    }
    if input.movement.x.abs() < deadzone {
        input.movement.x = 0.0;
    }
    if input.movement.y.abs() < deadzone {
        input.movement.y = 0.0;
    }
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
    state
        .inputs
        .push((LocalInputSource::Keyboard, keyboard_input(&key, &map)));
    
    let mut prev_joystick_states: Vec<(LocalInputSource, MenuJoystickState)> = vec![];
    std::mem::swap(&mut prev_joystick_states, &mut state.prev_frame_joystick_state);

    for (entity, pad) in pads.iter() {
        let local_source = LocalInputSource::Gamepad(entity);
        let prev_frame_inputs = state
            .prev_frame_joystick_state
            .iter()
            .find(|(source, _)| *source == local_source)
            .map(|(_, gp)| gp.clone())
            .unwrap_or(MenuJoystickState::default());

        state.inputs.push((
            local_source,
            gamepad_input(pad, &map, deadzone, &prev_frame_inputs),
        ));

        state
            .prev_frame_joystick_state
            .push((local_source, MenuJoystickState::from_gamepad(pad)));
    }
}
