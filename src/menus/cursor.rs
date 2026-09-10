use bevy::{
    asset::uuid::Uuid,
    math::VectorSpace,
    picking::pointer::{Location, PointerId, PointerInput, PointerLocation},
    prelude::*,
    window::{NormalizedWindowRef, WindowRef},
};

use crate::{game_settings::GameSettings, input::LocalInputAssignments, menus::input::MenuInput};

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
}

impl FGMenuCursor {
    pub fn create_cursor(
        player_slot: usize,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
        mapping: Res<LocalInputAssignments>,
    ) -> impl Bundle {
        let (_, mapping) = mapping
            .iter()
            .find(|(slot, _)| *slot == player_slot)
            .unwrap();

        let material = materials.add(ColorMaterial::from_color(
            bevy::color::palettes::tailwind::ROSE_600,
        ));
        let circle_mesh = meshes.add(Circle::new(25.0));
        let pointer_id = match mapping {
            crate::input::LocalInputSource::Keyboard => PointerId::Mouse,
            crate::input::LocalInputSource::Gamepad(_) => PointerId::Custom(Uuid::new_v4()),
        };
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
        )
    }
}

pub fn cursor_input(
    window: Single<Entity, With<bevy::window::PrimaryWindow>>,
    query: Query<(
        &mut FGMenuCursor,
        &PointerLocation,
        &PointerId,
    )>,
    key: Res<ButtonInput<KeyCode>>,
    assignments: Res<LocalInputAssignments>,
    game_settings: Res<GameSettings>,
    time: Res<Time>,
    pads: Query<(Entity, &Gamepad)>,
    mut pointer_writer: MessageWriter<PointerInput>,
) {
    for (mut cursor, location, pointer_id) in query {
        let input = match cursor.input_source {
            MenuCursorInputSource::Any => {
                let mut input = MenuInput::default();
                for (_, source) in assignments.iter() {
                    input.accumulate(&MenuInput::from_source(
                        source,
                        &key,
                        &pads,
                        &assignments,
                        &game_settings,
                    ));
                }
                input
            }
            MenuCursorInputSource::Player(slot) => {
                if let Some((_, source)) = assignments.iter().find(|(idx, _)| *idx == slot) {
                    MenuInput::from_source(source, &key, &pads, &assignments, &game_settings)
                } else {
                    MenuInput::default()
                }
            }
        };

        let movement_screen_space = input.movement * Vec2::new(1.0, -1.0);
        cursor.velocity = movement_screen_space * 500.0;

        if cursor.velocity == Vec2::ZERO {
            continue;
        }

        let old_cursor_pos = location
            .location
            .clone()
            .map(|loc| loc.position)
            .unwrap_or(Vec2::ZERO);
        let cursor_pos = old_cursor_pos + cursor.velocity * time.delta_secs();

        dbg!(cursor_pos);

        pointer_writer.write(PointerInput {
            pointer_id: *pointer_id,
            location: Location {
                target: bevy::camera::NormalizedRenderTarget::Window(
                    WindowRef::Primary
                        .normalize(Some(window.entity()))
                        .expect("Primary window should be valid"),
                ),
                position: cursor_pos,
            },
            action: bevy::picking::pointer::PointerAction::Move {
                delta: cursor_pos - old_cursor_pos,
            },
        });

        //cursor.position
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
