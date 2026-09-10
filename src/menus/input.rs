use bevy::prelude::*;

use crate::{game_settings::GameSettings, input::{LocalInputAssignments, LocalInputSource}};

#[derive(Default)]
pub struct MenuInput {
    pub movement: Vec2,
    pub accept: bool
}

impl MenuInput {

    pub fn accumulate(&mut self, other: &Self) {
        self.movement += other.movement;
        self.movement = self.movement.normalize();
        self.accept = self.accept || other.accept;
    }

    pub fn from_gamepad(gamepad: &Gamepad, game_settings: &GameSettings) -> Self {
        let mut x = gamepad.left_stick().x;
        let mut y = gamepad.left_stick().y;

        let deadzone = game_settings.input_common.stick_deadzone.to_num::<f32>();

        if x.abs() < deadzone {
            x = 0.0;
        }

        if y.abs() < deadzone {
            y = 0.0;
        }

        MenuInput {
            movement: Vec2::new(x, y),
            accept: gamepad.pressed(GamepadButton::South)
        }
    }
    pub fn from_keyboard(keeb: &ButtonInput<KeyCode>) -> Self {
        let x = if keeb.pressed(KeyCode::KeyA) { -1.0 } else { 0.0 };
        let x = x + if keeb.pressed(KeyCode::KeyD) { 1.0 } else { 0.0 };
        let y = if keeb.pressed(KeyCode::KeyS) { -1.0 } else { 0.0 };
        let y = y + if keeb.pressed(KeyCode::KeyW) { 1.0 } else { 0.0 };

        let movement = Vec2::new(x, y);
        MenuInput {
            movement: movement,
            accept: keeb.pressed(KeyCode::Enter)
        }
    }

    pub fn from_source(source: &LocalInputSource, key: &ButtonInput<KeyCode>, gamepads: &Query<(Entity, &Gamepad)>, assignments: &LocalInputAssignments, game_settings: &GameSettings) -> Self {
        match source {
            LocalInputSource::Keyboard => Self::from_keyboard(key),
            LocalInputSource::Gamepad(entity) => {
                let gamepad = gamepads.get(*entity);

                if let Ok((_, gamepad)) = gamepad {
                    Self::from_gamepad(gamepad, game_settings)
                } else {
                    Self::default()
                }
            },
        }
    }
}