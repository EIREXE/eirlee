use bevy::prelude::*;
use std::collections::HashMap;

use crate::fighter::animation::AnimKind;

#[derive(Component, Clone)]
pub struct FighterAnimations {
    pub graph: Handle<AnimationGraph>,
    pub clips: HashMap<AnimKind, AnimationNodeIndex>,
}
