use bevy::prelude::*;

use crate::{
    fighter::manifest::{FighterManifest, FighterManifestRegistry},
    input::LocalInputAssignments,
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
    mut commands: Commands,
    manifest_registry: Res<FighterManifestRegistry>,
    assignments: Res<LocalInputAssignments>,
    manifests: Res<Assets<FighterManifest>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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

    for (slot, _) in assignments.iter() {
        commands.spawn(FGMenuCursor::create_cursor(*slot, &mut meshes, &mut materials));
    }

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
) {
    let entity = commands
        .spawn_scene(fighter_portrait::FighterPortrait::create(player_slot))
        .id();
    commands.entity(container).add_child(entity);

    commands.spawn(
        FGMenuCursor::create_cursor(player_slot, &mut meshes, &mut materials)
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
) {
    let slot = addition.slot;
    create_portrait(
        slot,
        query.fighter_portait_container,
        commands,
        meshes,
        materials,
    );
}

pub fn css_input(
    menu_inputs: Res<crate::menus::input::MenuInputState>,
    mut assignments: ResMut<LocalInputAssignments>,
    mut commands: Commands,
) {
    for (source, input) in &menu_inputs.inputs {
        if input.movement != Vec2::ZERO {
            let existed = assignments.iter().any(|(_, assigned)| assigned == source);
            if !existed && let Some(free_idx) = assignments.get_free_player_index() {
                assignments.push((free_idx, *source));
                info!("New input assignment");
                commands.trigger(FighterSlotAssigned { slot: free_idx });
            }
        }
    }
}
