
use bevy::prelude::*;
pub struct FighterPortrait {
    pub player_slot: usize
}

impl FighterPortrait {
    pub fn create(player_slot: usize) -> impl Scene {
        bsn![
            Node {
                max_width: px(300),
                height: px(400),
                flex_grow: 1.0
            }
            BackgroundColor(bevy::color::palettes::css::ORANGE)
        ]
    }
}