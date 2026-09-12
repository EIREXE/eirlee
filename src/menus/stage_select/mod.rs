use bevy::prelude::*;

use crate::{
    AppState,
    game_settings::CommonAssets,
    match_loading::initiate_match,
    menus::{
        MenuMarker, cursor, match_config::MenuMatchConfig, spawn_menu,
        stage_select::stage_box::StageBox, style::FGUiStyle,
    },
    stage::manifest::{StageManifest, StageManifestRegistry},
};

pub mod stage_box;

#[derive(Component, FromTemplate)]
pub struct StageSelectScreen {
    pub stage_icons_container: Entity,
}

pub fn setup_stage_select(
    existing_menu: Option<Single<Entity, With<MenuMarker>>>,
    mut commands: Commands,
    manifest_registry: Res<StageManifestRegistry>,
    manifests: Res<Assets<StageManifest>>,
    common_assets: Res<CommonAssets>,
    styles: Res<Assets<FGUiStyle>>,
) {
    let style = styles
        .get(&common_assets.ui_style)
        .expect("UI style should be loaded");

    let stage_boxes = manifest_registry
        .iter()
        .map(|(_, v)| stage_box::StageBox::scene(v.clone(), &manifests))
        .collect::<Vec<_>>();

    let menu = bsn! {
        // Characters
        StageSelectScreen {
            stage_icons_container: #PortraitContainer
        }
        Node {
            flex_direction: FlexDirection::Column
        }
        Children [
        (
            Node {
                flex_direction: FlexDirection::Column,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center
            }
            Children [
                {stage_boxes},
            ]
        )
        ]
    };

    commands.spawn(cursor::FGMenuCursor::create_shared_cursor(style));

    spawn_menu(existing_menu, commands, menu);
}

pub fn stage_selected(
    ev: On<Pointer<Press>>,
    query: Query<&StageBox>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut match_config: ResMut<MenuMatchConfig>,
) -> Result {
    let stage_box = query.get(ev.event_target());
    if let Ok(stage_box) = stage_box {
        match_config.stage = Some(stage_box.stage_id);
        initiate_match(
            &mut commands,
            &mut next_state,
            match_config.to_pending_match()?,
        );
    }
    Ok(())
}
