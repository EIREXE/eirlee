//! Per-fighter input state: the current and previous frame, plus the buffered
//! commands (smash inputs and friends) that states query when deciding whether
//! to interrupt.

use std::ops::{Index, IndexMut};

use bevy::prelude::*;
use bevy_ggrs::{PlayerInputs, Rollback};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::fighter::attack::AttackKind;
use crate::game_settings::GameSettings;
use crate::input::FighterInputFrame;
use crate::netcode::GGRSCfg;
use crate::player::Player;

#[derive(Clone, Copy, EnumIter, Debug)]
pub enum FighterCommands {
    Jump,
    Shield,
    Attack,
    SmashMoveLeft,
    SmashMoveRight,
}

#[derive(Default, Clone, Hash, Copy)]
pub enum FighterAttackCommandType {
    #[default]
    Neutral,
    SmashAttackUp,
    SmashAttackDown,
    SmashAttackLeft,
    SmashAttackRight,
    TiltAttackUp,
    TiltAttackDown,
    TiltAttackLeft,
    TiltAttackRight,
}

impl FighterAttackCommandType {
    pub fn to_attack_kind_grounded(&self) -> AttackKind {
        match self {
            FighterAttackCommandType::Neutral => AttackKind::Jab,
            FighterAttackCommandType::SmashAttackUp => AttackKind::UpSmash,
            FighterAttackCommandType::SmashAttackDown => AttackKind::DownSmash,
            FighterAttackCommandType::SmashAttackLeft => AttackKind::ForwardSmash,
            FighterAttackCommandType::SmashAttackRight => AttackKind::ForwardSmash,
            FighterAttackCommandType::TiltAttackUp => AttackKind::UpTilt,
            FighterAttackCommandType::TiltAttackDown => AttackKind::DownTilt,
            FighterAttackCommandType::TiltAttackLeft => AttackKind::ForwardTilt,
            FighterAttackCommandType::TiltAttackRight => AttackKind::ForwardTilt,
        }
    }
}

/// Frames of life remaining for each buffered command. Indexed by
/// [`FighterCommands`] so adding a command is a two-line change here.
#[derive(Default, Clone, Copy, Hash)]
pub struct FighterCommandLifetimes {
    smash_move_left: u8,
    smash_move_right: u8,
    jump: u8,
    shield: u8,
    attack: u8,
}

impl Index<FighterCommands> for FighterCommandLifetimes {
    type Output = u8;

    fn index(&self, command: FighterCommands) -> &Self::Output {
        match command {
            FighterCommands::SmashMoveLeft => &self.smash_move_left,
            FighterCommands::SmashMoveRight => &self.smash_move_right,
            FighterCommands::Jump => &self.jump,
            FighterCommands::Shield => &self.shield,
            FighterCommands::Attack => &self.attack,
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
            FighterCommands::Attack => &mut self.attack,
        }
    }
}

#[derive(Component, Default, Clone, Copy, Hash)]
#[require(Rollback)]
pub struct FighterInput {
    command_lifetimes: FighterCommandLifetimes,
    prev_frame: FighterInputFrame,
    current_frame: FighterInputFrame,
    frames_in_x_flick_deadzone: u8,
    frames_in_y_flick_deadzone: u8,
    attack_command: FighterAttackCommandType,
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

    pub fn get_frames_in_smash_move_deadzone(&self) -> u8 {
        self.frames_in_x_flick_deadzone
    }

    pub fn set_attack_command_type(&mut self, command: FighterAttackCommandType) {
        self.attack_command = command;
    }

    pub fn get_attack_command_type(&self) -> FighterAttackCommandType {
        self.attack_command
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
        let y_abs = input.current_frame.movement.y.abs();

        let reset_deadzone = game_settings.input_common.smash_input_reset_axis_threshold;
        let axis_threshold = game_settings.input_common.smash_input_axis_threshold;
        let frame_threshold = game_settings.input_common.smash_input_frame_threshold;

        if x_abs < reset_deadzone {
            input.frames_in_x_flick_deadzone = 0xFE;
        } else {
            let prev_x_abs = input.prev_frame.movement.x.abs();
            if prev_x_abs < reset_deadzone
                || input.prev_frame.movement.x.signum() != input.current_frame.movement.x.signum()
            {
                input.frames_in_x_flick_deadzone = 1;
            } else {
                input.frames_in_x_flick_deadzone =
                    input.frames_in_x_flick_deadzone.saturating_add(1);
            }
        }

        if y_abs < reset_deadzone {
            input.frames_in_y_flick_deadzone = 0xFE;
        } else {
            let prev_y_abs = input.prev_frame.movement.y.abs();
            if prev_y_abs < reset_deadzone
                || input.prev_frame.movement.y.signum() != input.current_frame.movement.y.signum()
            {
                input.frames_in_y_flick_deadzone = 1;
            } else {
                input.frames_in_y_flick_deadzone =
                    input.frames_in_y_flick_deadzone.saturating_add(1);
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

        if !input.prev_frame.attack && input.current_frame.attack {
            input.set_lifetime(
                FighterCommands::Attack,
                game_settings.input_common.input_buffer_size,
            );
        }

        if x_abs >= axis_threshold && input.frames_in_x_flick_deadzone <= frame_threshold {
            input.clear_command(FighterCommands::SmashMoveLeft);
            input.clear_command(FighterCommands::SmashMoveRight);

            let command_to_set = if input.current_frame.movement.x > 0.0 {
                FighterCommands::SmashMoveRight
            } else {
                FighterCommands::SmashMoveLeft
            };

            input.set_lifetime(command_to_set, game_settings.input_common.input_buffer_size);
        }

        let mut y_flick_detected =
            y_abs >= axis_threshold && input.frames_in_y_flick_deadzone <= frame_threshold;
        let mut x_flick_detected =
            x_abs >= axis_threshold && input.frames_in_x_flick_deadzone <= frame_threshold;

        // if both flicks are detected, only one wins

        if x_flick_detected && y_flick_detected {
            x_flick_detected = x_abs > y_abs;
            y_flick_detected = !x_flick_detected;
        }

        let attack_detected = input.current_frame.attack && !input.prev_frame.attack;

        // Tilt/smash up/down
        if attack_detected {
            let attack_type = if y_flick_detected {
                if input.current_frame.movement.y > 0.0 {
                    FighterAttackCommandType::SmashAttackUp
                } else {
                    FighterAttackCommandType::SmashAttackDown
                }
            } else if x_flick_detected {
                if input.current_frame.movement.x > 0.0 {
                    FighterAttackCommandType::SmashAttackRight
                } else {
                    FighterAttackCommandType::SmashAttackLeft
                }
            } else if x_abs > y_abs {
                if input.current_frame.movement.x.is_positive() {
                    FighterAttackCommandType::TiltAttackRight
                } else {
                    FighterAttackCommandType::TiltAttackLeft
                }
            } else if y_abs > x_abs {
                if input.current_frame.movement.y.is_positive() {
                    FighterAttackCommandType::TiltAttackUp
                } else {
                    FighterAttackCommandType::TiltAttackDown
                }
            } else {
                FighterAttackCommandType::Neutral
            };

            input.set_lifetime(
                FighterCommands::Attack,
                game_settings.input_common.input_buffer_size,
            );
            input.set_attack_command_type(attack_type);
        }

        if x_flick_detected {
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
