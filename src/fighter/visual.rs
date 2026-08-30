use bevy::prelude::*;
use std::collections::HashMap;

use crate::fighter::animation::{AnimKind, FighterAnimationFrame};
use crate::fighter::baked_animation::BakedFighterAnimations;

#[derive(Component, Clone)]
pub struct FighterAnimations {
    pub graph: Handle<AnimationGraph>,
    pub clips: HashMap<AnimKind, AnimationNodeIndex>,
    /// Gameplay samples this fixed-point asset; the graph remains visual-only.
    pub baked: Handle<BakedFighterAnimations>,
}

impl FighterAnimations {
    pub fn sample_bone(
        &self,
        baked_assets: &Assets<BakedFighterAnimations>,
        frame: &FighterAnimationFrame,
        bone: &str,
    ) -> Option<crate::fighter::baked_animation::FixedMat4> {
        let baked = baked_assets.get(&self.baked)?;
        let frame_count = baked.frame_count(frame.kind)?;
        if frame_count == 0 {
            return None;
        }
        let frame_index = if frame.repeat {
            frame.frame % frame_count
        } else {
            frame.frame
        };
        baked.sample(frame.kind, frame_index, bone)
    }
}
