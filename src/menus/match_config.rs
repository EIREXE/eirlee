use bevy::prelude::*;
use thiserror::Error;

use crate::fighter::FighterId;

pub struct MenuMatchPlayerSlot {
    pub slot: usize,
    pub fighter: Option<FighterId>
}

#[derive(Resource, Default)]
pub struct MenuMatchConfig {
    pub players: Vec<MenuMatchPlayerSlot>
}

#[derive(Error, Debug, Reflect)]
pub enum MenuMatchConfigErrors {
    #[error("Fighter slot {0} was invalid!")]
    PlayerSlotInvalid(usize)
}

impl MenuMatchConfig {
    pub fn set_player_fighter(&mut self, player_slot: usize, fighter_id: Option<FighterId>) -> Result<(), MenuMatchConfigErrors> {
        let slot = self.players.get_mut(player_slot).ok_or(MenuMatchConfigErrors::PlayerSlotInvalid(player_slot))?;
        slot.fighter = fighter_id;
        Ok(())
    }
    pub fn get_player_fighter(&self, player_slot: usize) -> Result<Option<FighterId>, MenuMatchConfigErrors> {
        let slot = self.players.get(player_slot).ok_or(MenuMatchConfigErrors::PlayerSlotInvalid(player_slot))?;
        Ok(slot.fighter)
    }

    pub fn all_players_ready(&self) -> bool {
        let has_unselected_fighter = self.players.iter().any(|player_slot| player_slot.fighter.is_none());
        !has_unselected_fighter
    }
}