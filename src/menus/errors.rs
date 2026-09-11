use bevy::{ecs::query::QueryEntityError, reflect::Reflect};

use crate::{fighter::FighterId, menus::match_config::MenuMatchConfigErrors};

#[derive(thiserror::Error, Debug, Reflect)]
pub enum MenuErrors {
    #[error("Fighter manifest not found")]
    FighterManifestNotFound,
    #[error("Match config error: {0:?}")]
    MatchConfigError(#[from] MenuMatchConfigErrors),
}