use crate::fighter::baked_animation::BakedFighterAnimations;
use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct FighterAnimations {
    /// The single animation source for gameplay and the visual skeleton.
    pub baked: Handle<BakedFighterAnimations>,
}
