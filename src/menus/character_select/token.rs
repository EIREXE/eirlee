//! Token that is used to select characters in the CSS

use bevy::prelude::*;

use crate::menus::style::FGUiStyle;

#[derive(Component, Clone)]
pub struct CharacterSelectToken {
    pub slot: usize,
    pub cursor: Entity,
}

impl CharacterSelectToken {
    pub fn create(cursor: Entity, slot: usize, style: &FGUiStyle) -> impl Bundle {
        (
            CharacterSelectToken { slot, cursor },
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                width: px(64),
                height: px(64),
                ..default()
            },
            ImageNode {
                image: style.css_token.handle.handle.clone(),
                color: style
                    .player_colors
                    .get(slot)
                    .copied()
                    .unwrap_or(Color::WHITE),
                ..default()
            },
        )
    }
}
