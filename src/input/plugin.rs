use bevy::prelude::*;
use crate::fighter::*;
use crate::input::player::{ControlledBy, Controls, FighterInput};
use crate::input::{
    logic::{postprocess_input, preprocess_keyboard_input}, player::{BaseInputMap, InputMapAction, KeyboardInputMapElement},
};

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
            ],
        })
        .add_systems(FixedPreUpdate, (preprocess_keyboard_input, postprocess_input).chain());
    }
}
