use bevy::{
    picking::pointer::PointerId,
    prelude::*,
};

use crate::{
    AppState,
    fighter::manifest::{FighterManifest, FighterManifestRegistry},
    game_settings::CommonAssets,
    input::{LocalInputAssignments, LocalInputSource},
    menus::{
        MenuMarker,
        button::FGUiButton,
        character_select::{character_box::CharacterBox, token::CharacterSelectToken},
        cursor::{FGMenuCursor, MenuCursorInputSource},
        errors::MenuErrors::FighterManifestNotFound,
        match_config::{MenuMatchConfig, MenuMatchPlayerSlot},
        scaling, spawn_menu,
        style::{FGUiButtonType, FGUiStyle},
    },
};

pub mod character_box;
pub mod errors;
pub mod fighter_portrait;
pub mod token;

#[derive(Component, Clone, Default)]
pub struct StartText;

#[derive(Component, FromTemplate)]
pub struct CharacterSelectScreen {
    pub fighter_portait_container: Entity,
}

#[derive(Component, FromTemplate)]
pub struct StartButtonBanner;

pub fn setup_character_select(
    existing_menu: Option<Single<Entity, With<MenuMarker>>>,
    mut commands: Commands,
    manifest_registry: Res<FighterManifestRegistry>,
    _assignments: Res<LocalInputAssignments>,
    manifests: Res<Assets<FighterManifest>>,
    _common_assets: Res<CommonAssets>,
    _styles: Res<Assets<FGUiStyle>>,
) {
    let character_boxes = manifest_registry
        .iter()
        .map(|(_, v)| character_box::CharacterBox::scene(v.clone(), &manifests))
        .collect::<Vec<_>>();

    let menu = bsn! {
        // Characters
        CharacterSelectScreen {
            fighter_portait_container: #PortraitContainer
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
                {character_boxes},
            ]
        ),
        (
            #PortraitContainer
            Node {
                flex_direction: FlexDirection::Row,
                width: percent(100),
                min_height: px(400),
                justify_content: JustifyContent::Center
            }
        ),
        (
            // Start banner thingy
            StartButtonBanner
            Node {
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                width: percent(100),
            }
            FGUiButton {
                button_type: FGUiButtonType::CharacterSelectStartBanner
            }
            Text::new("Start poetry")
            Visibility::Hidden
            TextLayout {
                justify: Justify::Center
            }
            TextFont {
                font_size: FontSize::Px(48.0),
            }
            TextColor(Color::WHITE)
            BackgroundColor(Color::BLACK)
            GlobalZIndex(500)
            on(|_ev: On<Pointer<Press>>, mut next_state: ResMut<NextState<AppState>>| next_state.set(AppState::StageSelect))
        )
        ]
    };

    commands.init_resource::<MenuMatchConfig>();
    spawn_menu(existing_menu, commands, menu);
}

pub fn create_portrait(
    player_slot: usize,
    container: Entity,
    commands: &mut Commands,
    style: &FGUiStyle,
    input_source: &LocalInputSource,
) {
    let entity = commands
        .spawn_scene(fighter_portrait::FighterPortrait::create(player_slot))
        .id();
    commands.entity(container).add_child(entity);

    let cursor = commands
        .spawn(FGMenuCursor::create_player_cursor(
            player_slot,
            style,
            input_source,
        ))
        .id();
    commands.spawn(token::CharacterSelectToken::create(
        cursor,
        player_slot,
        style,
    ));
    commands.entity(container).add_child(entity);
}

#[derive(Message)]
pub struct PlayerSlotAssigned {
    pub slot: usize,
    pub input_source: LocalInputSource,
}

pub fn handle_fighter_slot_addition(
    mut addition: MessageReader<PlayerSlotAssigned>,
    query: Single<&CharacterSelectScreen>,
    mut commands: Commands,
    common_assets: Res<CommonAssets>,
    styles: Res<Assets<FGUiStyle>>,
) {
    for addition in addition.read() {
        let slot = addition.slot;
        create_portrait(
            slot,
            query.fighter_portait_container,
            &mut commands,
            styles
                .get(&common_assets.ui_style)
                .expect("UI Style should be loaded by now"),
            &addition.input_source,
        );
    }
}

pub fn css_auto_assign_slots(
    menu_inputs: Res<crate::menus::input::MenuInputState>,
    mut assignments: ResMut<LocalInputAssignments>,
    mut match_config: ResMut<MenuMatchConfig>,
    mut ev_assigned: MessageWriter<PlayerSlotAssigned>,
) {
    for (source, input) in &menu_inputs.inputs {
        if input.movement != Vec2::ZERO {
            let existed = assignments.iter().any(|(_, assigned)| assigned == source);
            if !existed && let Some(free_idx) = assignments.get_free_player_index() {
                assignments.push((free_idx, *source));
                match_config.players.push(MenuMatchPlayerSlot {
                    slot: free_idx,
                    fighter: None,
                });
                info!("New input assignment");
                ev_assigned.write(PlayerSlotAssigned {
                    slot: free_idx,
                    input_source: source.clone(),
                });
            }
        }
    }
}

#[derive(Event)]
pub struct FighterSelectedEvent {
    _slot: usize,
}

pub fn select_fighter(
    event: On<Pointer<Press>>,
    character_boxes: Query<&CharacterBox>,
    manifests: Res<Assets<FighterManifest>>,
    mut match_config: ResMut<MenuMatchConfig>,
    cursors: Query<(&FGMenuCursor, &PointerId)>,
    mut commands: Commands,
) -> Result {
    if let Some(character_box) = character_boxes.get(event.event_target()).ok() {
        let cursor = cursors.iter().find(|(_, id)| **id == event.pointer_id);

        if let Some((cursor, _)) = cursor {
            if let MenuCursorInputSource::Player(slot) = cursor.input_source {
                let manifest = manifests
                    .get(&character_box.fighter_manifest)
                    .ok_or(FighterManifestNotFound)?;
                match_config.set_player_fighter(slot, Some(manifest.id))?;
                commands.trigger(FighterSelectedEvent { _slot: slot });
            }
        }
    }

    Ok(())
}

pub fn update_start_banner_visibility(
    _addition: On<FighterSelectedEvent>,
    banner: Single<Entity, With<StartButtonBanner>>,
    mut commands: Commands,
    config: Res<MenuMatchConfig>,
) {
    let match_ready = config.all_players_ready();
    commands.entity(banner.into_inner()).insert(if match_ready {
        Visibility::Visible
    } else {
        Visibility::Hidden
    });
}

pub fn copy_cursor_transform_to_token(
    tokens: Query<(
        &token::CharacterSelectToken,
        &mut UiTransform,
        &ComputedNode,
    )>,
    character_boxes: Query<(&CharacterBox, &ComputedNode, &UiGlobalTransform)>,
    match_config: Res<MenuMatchConfig>,
    cursors: Query<&UiGlobalTransform, With<FGMenuCursor>>,
    cam: Single<&Camera, With<IsDefaultUiCamera>>,
    ui_scale: Res<UiScale>,
) -> Result {
    let camera = cam.into_inner();
    for (token, mut trf, computed) in tokens {
        let fighter = match_config.get_player_fighter(token.slot)?;

        if let Some(fighter_id) = fighter {
            // Has selected fighter, move the token to the fighter portrait
            let character_box = character_boxes
                .iter()
                .find(|(character_box, _, _)| character_box.fighter_id == fighter_id);
            if let Some((_, _, box_trf)) = character_box {
                let offset = computed.size * computed.inverse_scale_factor;
                let translation =
                    scaling::logical_to_ui_position(box_trf.translation, camera, &ui_scale)
                        - offset * 0.5;
                trf.translation = Val2::new(px(translation.x), px(translation.y));
            }
        } else {
            if let Ok(cursor_trf) = cursors.get(token.cursor) {
                let offset = computed.size * computed.inverse_scale_factor;
                let translation =
                    scaling::logical_to_ui_position(cursor_trf.translation, camera, &ui_scale)
                        - offset;
                trf.translation.x = px(translation.x);
                trf.translation.y = px(translation.y);
            }
        }
    }
    Ok(())
}

pub fn despawn_tokens(tokens: Query<Entity, With<CharacterSelectToken>>, mut commands: Commands) {
    for token in tokens {
        commands.entity(token).despawn();
    }
}
