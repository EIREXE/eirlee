use bevy::prelude::*;
use std::collections::HashMap;

use crate::fighter::animation::AnimKind;
use crate::fighter::baked_animation::BakedFighterAnimations;

#[derive(Component, Clone)]
pub struct FighterAnimations {
    pub graph: Handle<AnimationGraph>,
    pub clips: HashMap<AnimKind, AnimationNodeIndex>,
    /// Gameplay samples this fixed-point asset; the graph remains visual-only.
    pub baked: Handle<BakedFighterAnimations>,
}
