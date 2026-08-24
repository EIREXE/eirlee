use bevy::prelude::*;

use crate::fighter::state::{FighterState, FighterStateImpl};

pub struct JumpState {
    pub counter: u32
}

pub struct JumpSquatState {
    pub counter: u32
}


impl FighterStateImpl for JumpSquatState {
    const NAME: &'static str = "JumpSquat";
    fn check_interrupt(
        &self,
        state_context: &super::FighterStateContext,
    ) -> Option<FighterState> {
        None
    }

    fn update(&mut self, _state_context: &mut super::FighterStateContext) {
        todo!()
    }
    
}

impl FighterStateImpl for JumpState {
    const NAME: &'static str = "Jump";
    fn check_interrupt(
        &self,
        state_context: &super::FighterStateContext,
    ) -> Option<FighterState> {
        None
    }

    fn update(&mut self, _state_context: &mut super::FighterStateContext) {
        todo!()
    }
}