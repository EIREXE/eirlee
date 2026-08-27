//! Per-fighter input state: the current and previous frame, plus the buffered
//! commands (smash inputs and friends) that states query when deciding whether
//! to interrupt.

use std::ops::{Index, IndexMut};

use bevy::prelude::*;
use bevy_ggrs::{PlayerInputs, Rollback};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::game_settings::GameSettings;
use crate::input::FighterInputFrame;
use crate::netcode::GGRSCfg;
use crate::player::Player;

#[derive(Clone, Copy, EnumIter, Debug)]
pub enum FighterCommands {
    SmashMoveLeft,
    SmashMoveRight,
    Jump,
    Shield,
}

/// Frames of life remaining for each buffered command. Indexed by
/// [`FighterCommands`] so adding a command is a two-line change here.
#[derive(Default, Clone, Copy)]
pub struct FighterCommandLifetimes {
    smash_move_left: u8,
    smash_move_right: u8,
    jump: u8,
    shield: u8,
}

impl Index<FighterCommands> for FighterCommandLifetimes {
    type Output = u8;

    fn index(&self, command: FighterCommands) -> &Self::Output {
        match command {
            FighterCommands::SmashMoveLeft => &self.smash_move_left,
            FighterCommands::SmashMoveRight => &self.smash_move_right,
            FighterCommands::Jump => &self.jump,
            FighterCommands::Shield => &self.shield,
        }
    }
}

impl IndexMut<FighterCommands> for FighterCommandLifetimes {
    fn index_mut(&mut self, command: FighterCommands) -> &mut Self::Output {
        match command {
            FighterCommands::SmashMoveLeft => &mut self.smash_move_left,
            FighterCommands::SmashMoveRight => &mut self.smash_move_right,
            FighterCommands::Jump => &mut self.jump,
            FighterCommands::Shield => &mut self.shield,
        }
    }
}

#[derive(Component, Default, Clone, Copy)]
#[require(Rollback)]
pub struct FighterInput {
    command_lifetimes: FighterCommandLifetimes,
    prev_frame: FighterInputFrame,
    current_frame: FighterInputFrame,
    frames_in_smash_move_deadzone: u8,
}

impl FighterInput {
    pub fn get_last_frame(&self) -> FighterInputFrame {
        self.current_frame
    }

    pub fn push_input_frame(&mut self, frame: FighterInputFrame) {
        self.prev_frame = self.current_frame;
        self.current_frame = frame;
    }

    pub fn clear_buffer(&mut self) {
        self.command_lifetimes = FighterCommandLifetimes::default()
    }

    pub fn clear_command(&mut self, command: FighterCommands) {
        self.command_lifetimes[command] = 0;
    }

    pub fn set_lifetime(&mut self, command: FighterCommands, lifetime: u8) {
        self.command_lifetimes[command] = lifetime;
    }

    pub fn has_command(&self, command: FighterCommands) -> bool {
        self.command_lifetimes[command] > 0
    }

    pub fn reduce_command_lifetime(&mut self, command: FighterCommands) {
        self.command_lifetimes[command] = self.command_lifetimes[command].saturating_sub(1);
    }
}

/// Turns this frame's rolled-back inputs into per-fighter [`FighterInput`]
/// state. Runs in `GameplaySet::Input`, before any state looks at it.
pub fn postprocess_input(
    query: Query<(&mut FighterInput, &Player)>,
    inputs: Res<PlayerInputs<GGRSCfg>>,
    game_settings: Res<GameSettings>,
) {
    for (mut input, player) in query {
        for command in FighterCommands::iter() {
            input.reduce_command_lifetime(command);
        }

        input.push_input_frame(inputs[player.handle].0);

        // Directional smash detection
        let x_abs = input.current_frame.movement.x.abs();

        let reset_deadzone = game_settings.input_common.smash_input_reset_axis_threshold;
        let axis_threshold = game_settings.input_common.smash_input_axis_threshold;
        let frame_threshold = game_settings.input_common.smash_input_frame_threshold;

        if x_abs < reset_deadzone {
            input.frames_in_smash_move_deadzone = 0xFE;
        } else {
            let prev_x_abs = input.prev_frame.movement.x.abs();
            if prev_x_abs < reset_deadzone
                || input.prev_frame.movement.x.signum() != input.current_frame.movement.x.signum()
            {
                input.frames_in_smash_move_deadzone = 0;
            } else {
                input.frames_in_smash_move_deadzone += 1;
            }
        }

        if !input.prev_frame.jump && input.current_frame.jump {
            input.set_lifetime(
                FighterCommands::Jump,
                game_settings.input_common.input_buffer_size,
            );
        }

        if !input.prev_frame.shield && input.current_frame.shield {
            input.set_lifetime(
                FighterCommands::Shield,
                game_settings.input_common.input_buffer_size,
            );
        }

        if x_abs >= axis_threshold && input.frames_in_smash_move_deadzone <= frame_threshold {
            input.clear_command(FighterCommands::SmashMoveLeft);
            input.clear_command(FighterCommands::SmashMoveRight);

            let command_to_set = if input.current_frame.movement.x > 0.0 {
                FighterCommands::SmashMoveRight
            } else {
                FighterCommands::SmashMoveLeft
            };

            input.set_lifetime(command_to_set, game_settings.input_common.input_buffer_size);
        }
    }
}
