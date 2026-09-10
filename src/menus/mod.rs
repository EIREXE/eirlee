use bevy::{
    input_focus::{
        InputFocus, InputFocusVisible,
        directional_navigation::{AutoNavigationConfig, DirectionalNavigationPlugin},
    },
    prelude::*,
};

use bevy::prelude::*;

use crate::{
    AppState,
    game_settings::CommonAssets,
    menus::{character_select::CharacterSelectScreen, main_menu::MainMenu, style::FGUiStyle},
};
pub mod button;
pub mod character_select;
pub mod cursor;
pub mod input;
pub mod main_menu;
pub mod navigation;
pub mod scaling;
pub mod style;

#[derive(Component, Clone, Default)]
pub struct MenuMarker {}

pub struct MenuPlugin;

pub fn build_menu_scene<S>(existing_menu: Option<Single<Entity, With<MenuMarker>>>) -> impl Scene
where
    S: SceneComponent + Default + Clone + Send + Sync,
{
}

pub fn wrap_menu(menu: impl Scene) -> impl Scene {
    bsn! {
        #MenuRoot
        MenuMarker
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
        }
        Children [
            (
                #MenuContainer
                Node {
                    width: px(1920),
                    height: px(1080),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: vmin(5.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect { left: vmin(5.0), right: vmin(5.0), top: vmin(5.0), bottom: vmin(5.0) }
                }
                menu
            )
        ]
    }
}

pub fn spawn_menu(
    existing_menu: Option<Single<Entity, With<MenuMarker>>>,
    mut commands: Commands,
    menu: impl Scene,
) {
    if let Some(entity) = existing_menu {
        commands
            .entity(entity.into_inner())
            .remove::<Camera2d>()
            .remove::<Camera>()
            .despawn();
    }
    commands.queue_spawn_scene(wrap_menu(menu));
}

pub fn build_menu_with_props<S>(props: S::Props) -> impl Scene
where
    S: SceneComponent + Default + Clone + Send + Sync,
{
    S::scene(props)
}

pub fn build_menu<S>(
    existing_menu: Option<Single<Entity, With<MenuMarker>>>,
    mut commands: Commands,
) where
    S: SceneComponent + Default + Clone + Send + Sync,
{
    spawn_menu(
        existing_menu,
        commands,
        build_menu_with_props::<S>(S::Props::default()),
    );
}

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::MainMenu),
            (build_menu::<MainMenu>, |mut commands: Commands| {
                commands.spawn((Camera2d, IsDefaultUiCamera));
                commands.spawn((
                    Camera2d,
                    Camera {
                        order: 1,
                        ..default()
                    },
                ));
            }),
        )
        .add_systems(
            OnEnter(AppState::CharacterSelect),
            character_select::setup_character_select,
        )
        .init_resource::<navigation::UINavigationActionState>()
        .insert_resource(InputFocusVisible(true))
        .insert_resource(GlobalUiDebugOptions {
            enabled: true,
            ..default()
        })
        // Configure auto-navigation behavior
        .insert_resource(AutoNavigationConfig {
            // Require at least 10% overlap in perpendicular axis for cardinal directions
            min_alignment_factor: 0.1,
            // Don't connect nodes more than 500 pixels apart between their closest edges
            max_search_distance: Some(500.0),
            // Prefer nodes that are well-aligned
            prefer_aligned: true,
        })
        .add_plugins(DirectionalNavigationPlugin)
        .add_systems(
            PreUpdate,
            (
                navigation::give_default_focus,
                scaling::ui_scaling_system,
                button::button_setup,
                button::button_style_system,
                button::button_focus_style_system,
            )
                .run_if(resource_exists::<CommonAssets>),
        )
        .add_systems(Startup, scaling::ui_update_scale_on_startup)
        .add_systems(
            PostUpdate,
            (character_select::css_input, (cursor::cursor_input, cursor::update_cursor_transform).chain())
                .run_if(in_state(AppState::CharacterSelect))
                .run_if(any_with_component::<CharacterSelectScreen>),
        )
        .add_observer(character_select::handle_fighter_slot_addition)
        .add_plugins(bevy_common_assets::ron::RonAssetPlugin::<FGUiStyle>::new(
            &["stylesheet.ron"],
        ))
        .add_systems(
            Update,
            (
                navigation::process_inputs,
                navigation::navigate,
                navigation::interact_with_focused_button,
            )
                .chain(),
        )
        .init_resource::<InputFocus>();
    }
}
