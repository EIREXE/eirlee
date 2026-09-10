use bevy::prelude::*;

use crate::{
    fighter::manifest::{FighterManifest, FighterManifestRegistry},
    input::{LocalInputAssignments, LocalInputSource},
    menus::{MenuMarker, build_menu_with_props, cursor::FGMenuCursor, spawn_menu, wrap_menu},
};

pub mod character_box;
pub mod fighter_portrait;

#[derive(Component, FromTemplate)]
pub struct CharacterSelectScreen {
    pub fighter_portait_container: Entity,
}

pub fn setup_character_select(
    existing_menu: Option<Single<Entity, With<MenuMarker>>>,
    commands: Commands,
    manifest_registry: Res<FighterManifestRegistry>,
    assignments: Res<LocalInputAssignments>,
    manifests: Res<Assets<FighterManifest>>,
) {
    let character_boxes = manifest_registry
        .iter()
        .map(|(_, v)| character_box::CharacterBox::scene(v.clone(), &manifests))
        .collect::<Vec<_>>();

    // Create existing portraits
    let fighter_portraits = assignments
        .iter()
        .map(|(slot, _)| fighter_portrait::FighterPortrait::create(*slot))
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
            (Node {
                flex_direction: FlexDirection::Column,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center
            }
            Children [
                {character_boxes}
            ]
        ),
        (
            #PortraitContainer
            Node {
                flex_direction: FlexDirection::Row,
                width: percent(100),
                justify_content: JustifyContent::Center
            }
            Children [
                {fighter_portraits}
            ]
        )
        ]
    };

    spawn_menu(existing_menu, commands, menu);
}

pub fn create_portrait(
    player_slot: usize,
    container: Entity,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mapping: Res<LocalInputAssignments>,
) {
    let entity = commands
        .spawn_scene(fighter_portrait::FighterPortrait::create(player_slot))
        .id();
    commands.entity(container).add_child(entity);

    commands.spawn(
        FGMenuCursor::create_cursor(player_slot, meshes, materials, mapping)
    );
}

#[derive(Event)]
pub struct FighterSlotAssigned {
    pub slot: usize,
}

pub fn handle_fighter_slot_addition(
    addition: On<FighterSlotAssigned>,
    query: Single<&CharacterSelectScreen>,
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    mapping: Res<LocalInputAssignments>,
) {
    let slot = addition.slot;
    create_portrait(
        slot,
        query.fighter_portait_container,
        commands,
        meshes,
        materials,
        mapping,
    );
}

pub fn css_input(
    key: Res<ButtonInput<KeyCode>>,
    pads: Query<(Entity, &Gamepad)>,
    mut assignments: ResMut<LocalInputAssignments>,
    mut commands: Commands,
) {
    if key.get_pressed().len() != 0 {
        if let None = assignments
            .iter()
            .find(|(_, source)| matches!(source, LocalInputSource::Keyboard))
        {
            if let Some(free_idx) = assignments.get_free_player_index() {
                assignments.push((free_idx, LocalInputSource::Keyboard));
                info!("New keyboard assignment");
                commands.trigger(FighterSlotAssigned { slot: free_idx });
            }
        }
    }

    for (entity, pad) in pads {
        if pad.any_pressed(GamepadButton::all()) {
            let existed = assignments
                .iter()
                .find(|(_, assignment)| {
                    if let LocalInputSource::Gamepad(assignment_entity) = assignment {
                        return entity == *assignment_entity;
                    }
                    false
                })
                .is_some();

            if !existed && let Some(free_idx) = assignments.get_free_player_index() {
                assignments.push((free_idx, LocalInputSource::Gamepad(entity)));
                commands.trigger(FighterSlotAssigned { slot: free_idx });
            }
        }
    }
}
