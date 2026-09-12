use bevy::{
    asset::uuid::Uuid,
    picking::{
        Pickable,
        pointer::{Location, PointerAction, PointerId, PointerInput, PointerLocation},
    },
    prelude::*,
    window::WindowRef,
};

use crate::{
    input::{LocalInputAssignments, LocalInputSource},
    menus::{
        input::{MenuInput, MenuInputState},
        scaling,
        style::FGUiStyle,
    },
};

#[derive(Default)]
pub enum MenuCursorInputSource {
    #[default]
    Any,
    Player(usize),
}

#[derive(Component, Default)]
pub struct FGMenuCursor {
    pub input_source: MenuCursorInputSource,
    pub velocity: Vec2,
    pub accepting: bool,
}

impl FGMenuCursor {
    pub fn create_cursor(
        input_source: MenuCursorInputSource,
        style: &FGUiStyle,
        pointer_id: PointerId,
    ) -> impl Bundle {
        let circle_icon = style.cursor.handle().clone();
        (
            FGMenuCursor {
                input_source: input_source,
                ..default()
            },
            Node {
                width: px(64),
                height: px(64),
                left: percent(50),
                top: percent(50),
                position_type: PositionType::Absolute,
                ..default()
            },
            ImageNode {
                image: circle_icon,
                ..default()
            },
            pointer_id,
            Transform::IDENTITY,
            GlobalZIndex(1000),
            Pickable::IGNORE,
        )
    }
    pub fn create_shared_cursor(style: &FGUiStyle) -> impl Bundle {
        Self::create_cursor(MenuCursorInputSource::Any, style, PointerId::Mouse)
    }
    pub fn create_player_cursor(
        player_slot: usize,
        style: &FGUiStyle,
        input_source: &LocalInputSource,
    ) -> impl Bundle {
        let pointer_id = if matches!(input_source, LocalInputSource::Keyboard) {
            PointerId::Mouse
        } else {
            PointerId::Custom(Uuid::new_v4())
        };
        Self::create_cursor(
            MenuCursorInputSource::Player(player_slot),
            style,
            pointer_id,
        )
    }
}

/// The player using the keyboard can input things with the mouse, so we should copy their movement input
pub fn copy_mouse_input(
    mut query: Query<(&FGMenuCursor, &mut PointerLocation, &PointerId)>,
    assignments: Res<LocalInputAssignments>,
    mut input_events: MessageReader<Pointer<Move>>,
) {
    for event in input_events.read() {
        if let PointerId::Mouse = event.pointer_id {
            for (cursor, mut location, _) in query.iter_mut() {
                let is_mouse = match cursor.input_source {
                    MenuCursorInputSource::Any => true,
                    MenuCursorInputSource::Player(slot) => assignments
                        .get(slot)
                        .map(|(_, source)| matches!(source, LocalInputSource::Keyboard))
                        .unwrap_or_default(),
                };

                if is_mouse {
                    location.location = Some(event.pointer_location.clone());
                }
            }
        }
    }
}

pub fn cursor_input(
    window: Single<(Entity, &Window), With<bevy::window::PrimaryWindow>>,
    query: Query<(&mut FGMenuCursor, &PointerLocation, &PointerId)>,
    assignments: Res<LocalInputAssignments>,
    menu_inputs: Res<MenuInputState>,
    time: Res<Time>,
    mut pointer_writer: MessageWriter<PointerInput>,
) {
    let (window_entity, window) = window.into_inner();

    for (mut cursor, location, pointer_id) in query {
        let input = match cursor.input_source {
            MenuCursorInputSource::Any => {
                let mut input = MenuInput::default();
                for (_, source) in assignments.iter() {
                    input.accumulate(&menu_inputs.get(source));
                }
                input
            }
            MenuCursorInputSource::Player(slot) => {
                if let Some((_, source)) = assignments.iter().find(|(idx, _)| *idx == slot) {
                    menu_inputs.get(source)
                } else {
                    MenuInput::default()
                }
            }
        };

        let movement_screen_space = input.movement * Vec2::new(1.0, -1.0);
        cursor.velocity = movement_screen_space * 500.0;

        let initial_cursor_pos = Vec2::new(window.width(), window.height()) / 2.0;
        let old_cursor_pos = location
            .location
            .clone()
            .map(|loc| loc.position)
            .unwrap_or(initial_cursor_pos);
        let cursor_pos = (old_cursor_pos + cursor.velocity * time.delta_secs())
            .clamp(Vec2::ZERO, Vec2::new(window.width(), window.height()));
        let pointer_location = Location {
            target: bevy::camera::NormalizedRenderTarget::Window(
                WindowRef::Primary
                    .normalize(Some(window_entity))
                    .expect("Primary window should be valid"),
            ),
            position: cursor_pos,
        };

        if location.location.is_none() || cursor_pos != old_cursor_pos {
            pointer_writer.write(PointerInput {
                pointer_id: *pointer_id,
                location: pointer_location.clone(),
                action: PointerAction::Move {
                    delta: cursor_pos - old_cursor_pos,
                },
            });
        }

        if input.accept.is_pressed() != cursor.accepting {
            cursor.accepting = input.accept.is_pressed();
            pointer_writer.write(PointerInput {
                pointer_id: *pointer_id,
                location: pointer_location,
                action: if input.accept.is_pressed() {
                    PointerAction::Press(PointerButton::Primary)
                } else {
                    PointerAction::Release(PointerButton::Primary)
                },
            });
        }
    }
}

pub fn update_cursor_transform(
    camera: Single<&Camera, With<IsDefaultUiCamera>>,
    query: Query<(&mut Node, &PointerLocation), With<FGMenuCursor>>,
    ui_scale: Res<UiScale>,
) {
    let camera = camera.into_inner();

    for (mut node, location) in query {
        let ui_pos = scaling::logical_to_ui_position(
            location
                .location
                .as_ref()
                .map(|d| d.position)
                .unwrap_or_default(),
            camera,
            &ui_scale,
        );
        node.left = px(ui_pos.x);
        node.top = px(ui_pos.y);
    }
}

pub fn despawn_cursors(query: Query<Entity, With<FGMenuCursor>>, mut commands: Commands) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}
