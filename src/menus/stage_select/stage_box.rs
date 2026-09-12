use bevy::{picking::Pickable, prelude::*, text::FontSourceTemplate};

use crate::{
    menus::{button::FGUiButton, style::FGUiButtonType},
    stage::manifest::{StageId, StageManifest},
};

#[derive(Component, Clone, FromTemplate)]
pub struct StageBox {
    pub stage_manifest: Handle<StageManifest>,
    pub stage_id: StageId,
}

impl StageBox {
    pub fn scene(
        manifest_handle: Handle<StageManifest>,
        manifests: &Res<Assets<StageManifest>>,
    ) -> impl Scene {
        let manifest = manifests
            .get(&manifest_handle)
            .expect("Stage manifest should be valid");
        let fighter_icon = manifest.icon.handle().clone();
        let stage_id = manifest.id;
        bsn! {
            Node {
                width: px(256),
                height: px(256),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd
            }
            ImageNode {
                image: fighter_icon
            }
            FGUiButton {
                button_type: FGUiButtonType::StageSelectIcon
            }
            StageBox {
                stage_manifest: manifest_handle,
                stage_id: stage_id
            }
            BackgroundColor(Color::srgb(1.0, 1.0, 1.0))
            Children [
                Pickable::IGNORE
                Node {
                    width: percent(100)
                }
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
