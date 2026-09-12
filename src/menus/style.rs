use std::collections::HashMap;

use bevy::prelude::*;
use ron_asset_manager::prelude::*;
use serde::{Deserialize, Serialize};

use crate::texture_reference::TextureReference;

pub const H1_SIZE: i32 = 72;
pub const H2_SIZE: i32 = 54;
pub const H3_SIZE: i32 = 40;
pub const TEXT_SIZE: i32 = 24;

#[derive(Serialize, Deserialize, Reflect, Default)]
#[serde(default)]
pub struct BoxShadowStyle {
    color: Color,
    offset_px_x: i32,
    offset_px_y: i32,
    spread_radius: f32,
    blur_radius: f32,
}

#[derive(Serialize, Deserialize, Reflect, Default)]
#[serde(default)]
pub struct ButtonStyle {
    background_color: Color,
    border_color: Color,
    border_radius: f32,
    text_color: Color,
    box_shadow: BoxShadowStyle,
    border_size: i32,
}

#[derive(Serialize, Deserialize, Reflect)]
pub struct ButtonStyles {
    pub normal: ButtonStyle,
    pub hover: ButtonStyle,
    pub press: ButtonStyle,
    pub focus_outline_width_px: i32,
    pub focus_outline_offset_px: i32,
    pub focus_outline_color: Color,
}

impl ButtonStyle {
    pub fn apply(&self, commands: &mut Commands, entity: Entity, node: &mut Node) {
        node.border = UiRect::all(px(self.border_size));
        node.border_radius = BorderRadius::all(px(self.border_radius));
        commands.entity(entity).insert((
            BackgroundColor(self.background_color),
            BorderColor::all(self.border_color),
            BoxShadow::new(
                self.box_shadow.color,
                px(self.box_shadow.offset_px_x),
                px(self.box_shadow.offset_px_y),
                px(self.box_shadow.spread_radius),
                px(self.box_shadow.blur_radius),
            ),
        ));
    }
}

#[derive(Reflect, Eq, PartialEq, Hash, Deserialize, Serialize, Default, Clone)]
pub enum FGUiButtonType {
    #[default]
    MainMenu,
    CharacterSelectCharacter,
    CharacterSelectStartBanner,
    StageSelectIcon,
}

#[derive(Asset, Resource, Reflect, Deserialize, RonAsset)]
pub struct FGUiStyle {
    pub button_styles: HashMap<FGUiButtonType, ButtonStyles>,
    #[asset]
    #[dependency]
    #[reflect(ignore)]
    pub cursor: TextureReference,
    #[asset]
    #[dependency]
    #[reflect(ignore)]
    pub css_token: TextureReference,
    pub player_colors: Vec<Color>,
}
