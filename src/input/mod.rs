//! Input, from physical device to the buffered per-fighter state that the
//! state machine reads.
//!
//! The pipeline is: [`keyboard`] samples devices into [`FighterInputFrame`]s
//! that GGRS ships over the wire, then [`buffer::postprocess_input`] turns the
//! rolled-back frames into [`FighterInput`].

use bevy::prelude::*;
use bevy_ggrs::prelude::*;

pub mod buffer;
pub mod control;
pub mod frame;
pub mod keyboard;
pub mod map;

pub use buffer::{FighterCommands, FighterInput};
pub use frame::FighterInputFrame;
pub use map::{BaseInputMap, InputActionState, InputMapAction, KeyboardInputMapElement};

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
                // jump
                KeyboardInputMapElement {
                    key: KeyCode::Space,
                    action: InputMapAction::Jump,
                },
            ],
        })
        .add_systems(ReadInputs, keyboard::preprocess_keyboard_input)
        .add_systems(
            GgrsSchedule,
            buffer::postprocess_input.in_set(GameplaySet::Input),
        )
        .rollback_component_with_clone::<FighterInput>();
    }
}
