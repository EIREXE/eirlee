use bevy::{prelude::*, text::FontSourceTemplate};

use crate::{fighter::manifest::FighterManifest, menus::{button::FGUiButton, style::FGUiButtonType}};

#[derive(Component, Clone, Default, FromTemplate)]
pub struct CharacterBox {
    fighter_manifest: Handle<FighterManifest>,
}

impl CharacterBox {
    pub fn scene(manifest_handle: Handle<FighterManifest>, manifests: &Res<Assets<FighterManifest>>) -> impl Scene {
        let manifest = manifests.get(&manifest_handle).expect("Fighter manifest should be valid");
        bsn! {
            Node {
                width: px(128),
                height: px(128),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd
            }
            FGUiButton {
                button_type: FGUiButtonType::CharacterSelectCharacter
            }
            CharacterBox {
                fighter_manifest: manifest_handle
            }
            BackgroundColor(Color::srgb(1.0, 1.0, 1.0))
            Children [
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