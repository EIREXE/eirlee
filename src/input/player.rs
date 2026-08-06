use std::collections::VecDeque;

use bevy::prelude::*;
use arrayvec::ArrayVec;

#[derive(Default, Clone)]
pub struct InputFrame {
    pub movement: Vec2,
    pub directional_attack: Vec2
}

#[derive(Component, Default)]
pub struct FighterInput {
    input_buffer: VecDeque<InputFrame>
}

impl FighterInput {
    const INPUT_BUFFER_SIZE: usize = 5; 
    pub fn get_last_frame(&self) -> Option<&InputFrame> {
        self.input_buffer.back()
    }

    pub fn push_input_frame(&mut self, frame: InputFrame) {
        while self.input_buffer.len() >= Self::INPUT_BUFFER_SIZE {
            self.input_buffer.pop_front();
        }
        self.input_buffer.push_back(frame);
    }

    pub fn clear_buffer(&mut self) {
        while self.input_buffer.len() > 1 {
            self.input_buffer.pop_front();
        }
    }

    pub fn has_smash_x_movement(&self, reset_deadzone: f32, threshold: f32, frame_threshold: u32) -> Option<f32> {
        let iter = self.input_buffer.iter().zip(self.input_buffer.iter().skip(1));
        
        let mut frames_in_deadzone = 0xFE;
        for (prev, current) in iter {
            let x_abs = current.movement.x.abs();
            if x_abs < reset_deadzone {
                frames_in_deadzone = 0xFE;
            } else {
                let prev_x_abs = prev.movement.x.abs();
                if prev_x_abs < reset_deadzone || prev.movement.x.signum() != current.movement.x.signum() {
                    frames_in_deadzone = 0;
                } else {
                    frames_in_deadzone += 1;
                }
            }

            if x_abs >= threshold && frames_in_deadzone <= frame_threshold {
                return Some(x_abs.signum());
            }
        }

        None
    }
}

#[derive(Debug)]
pub enum InputActionState {
    JustPressed,
    Pressed,
    JustReleased,
    Released
}

#[derive(Debug)]
pub enum InputMapAction {
    MovementXDir(i32),
    MovementYDir(i32),
    Jump
}
#[derive(Debug)]
pub struct KeyboardInputMapElement {
    pub key: KeyCode,
    pub action: InputMapAction
}

#[derive(Resource)]
pub struct BaseInputMap {
    pub keyboard: Vec<KeyboardInputMapElement>
}

#[derive(Component, Debug)]
#[relationship(relationship_target = ControlledBy)]
pub struct Controls(pub Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = Controls)]
pub struct ControlledBy(Entity);