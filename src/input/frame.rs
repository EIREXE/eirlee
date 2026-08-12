use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// One frame of a player's raw input. This is the type GGRS serializes and
/// sends over the wire, so keep it small and keep it `PartialEq`.
#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct FighterInputFrame {
    pub movement: Vec2,
    pub directional_attack: Vec2,
}
