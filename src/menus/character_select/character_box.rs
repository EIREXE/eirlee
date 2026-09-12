use bevy::{picking::Pickable, prelude::*, text::FontSourceTemplate, ui::widget::ImageNodeSize};

use crate::{
    fighter::{FighterId, manifest::FighterManifest},
    menus::{button::FGUiButton, style::FGUiButtonType},
};

#[derive(Component, Clone, FromTemplate)]
pub struct CharacterBox {
    pub fighter_manifest: Handle<FighterManifest>,
    pub fighter_id: FighterId,
}

impl CharacterBox {
    pub fn scene(
        manifest_handle: Handle<FighterManifest>,
        manifests: &Res<Assets<FighterManifest>>,
    ) -> impl Scene {
        let manifest = manifests
            .get(&manifest_handle)
            .expect("Fighter manifest should be valid");
        let fighter_icon = manifest.icon.handle().clone();
        let fighter_id = manifest.id;
        bsn! {
            Node {
                width: px(128),
                height: px(128),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd
            }
            ImageNode {
                image: fighter_icon
            }
            FGUiButton {
                button_type: FGUiButtonType::CharacterSelectCharacter
            }
            CharacterBox {
                fighter_manifest: manifest_handle,
                fighter_id: fighter_id
            }
            BackgroundColor(Color::srgb(1.0, 1.0, 1.0))
            Children [
                Pickable::IGNORE
                Node {
                    width: percent(100)
                }
                Text("El Grande Padre")
                TextFont {
                    font: FontSourceTemplate::Handle("fonts/roboto.ttf"),
                    font_size: px(crate::menus::style::TEXT_SIZE),
                }
                TextColor(Color::srgb(0.9, 0.9, 0.9))
                BackgroundColor(Color::srgb(0.0, 0.0, 1.0))
            ]
        }
    }
}
