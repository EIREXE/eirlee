use bevy::{
    asset::uuid::Uuid,
    picking::{pointer::{Location, PointerAction, PointerId, PointerInput, PointerLocation}, Pickable},
    prelude::*,
    window::WindowRef,
};

use crate::{input::LocalInputAssignments, menus::input::{MenuInput, MenuInputState}};

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
        player_slot: usize,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> impl Bundle {
        let material = materials.add(ColorMaterial::from_color(
            bevy::color::palettes::tailwind::ROSE_600,
        ));
        let circle_mesh = meshes.add(Circle::new(25.0));
        let pointer_id = PointerId::Custom(Uuid::new_v4());
        (
            FGMenuCursor {
                input_source: super::cursor::MenuCursorInputSource::Player(player_slot),
                ..default()
            },
            pointer_id,
            MeshMaterial2d(material),
            Mesh2d(circle_mesh),
            Transform::IDENTITY,
            GlobalZIndex(1000),
            Pickable::IGNORE,
        )
    }
}

pub fn cursor_input(
    window: Single<(Entity, &Window), With<bevy::window::PrimaryWindow>>,
    query: Query<(
        &mut FGMenuCursor,
        &PointerLocation,
        &PointerId,
    )>,
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

        if input.accept != cursor.accepting {
            cursor.accepting = input.accept;
            pointer_writer.write(PointerInput {
                pointer_id: *pointer_id,
                location: pointer_location,
                action: if input.accept {
                    PointerAction::Press(PointerButton::Primary)
                } else {
                    PointerAction::Release(PointerButton::Primary)
                },
            });
        }
    }
}

pub fn update_cursor_transform(
    camera: Single<(&Camera, &GlobalTransform), With<IsDefaultUiCamera>>,
    query: Query<(&mut Transform, &PointerLocation), With<FGMenuCursor>>,
) {
    let (camera, camera_transform) = camera.into_inner();

    for (mut trf, location) in query {
        let cursor_pos_viewport = camera
            .viewport_to_world_2d(
                camera_transform,
                location
                    .clone()
                    .location
                    .map(|d| d.position)
                    .unwrap_or_default(),
            )
            .ok()
            .unwrap_or_default();

        trf.translation = Vec3::new(cursor_pos_viewport.x, cursor_pos_viewport.y, 0.0);
    }
}

pub fn despawn_cursors(query: Query<Entity, With<FGMenuCursor>>, mut commands: Commands) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}
