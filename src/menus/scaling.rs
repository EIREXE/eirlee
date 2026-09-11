use bevy::camera::CameraProjection;
use bevy::camera::Projection::Orthographic;
use bevy::prelude::*;
use bevy::window::WindowResized;

fn compute_ui_scale(window_size: Vec2) -> f32 {
    const BASE_SIZE: Vec2 = Vec2::new(1920.0, 1080.0);
    const BASE_ASPECT_RATIO: f32 = BASE_SIZE.x / BASE_SIZE.y;
    let new_aspect_ratio = window_size.x / window_size.y;
    if new_aspect_ratio > BASE_ASPECT_RATIO {
        window_size.y / BASE_SIZE.y
    } else {
        window_size.x / BASE_SIZE.x
    }
}

pub fn ui_update_scale_on_startup(window: Single<&Window>, mut ui_scale: ResMut<UiScale>) {
    ui_scale.0 = compute_ui_scale(window.size());
}

pub fn ui_scaling_system(
    mut resize_reader: MessageReader<WindowResized>,
    mut ui_scale: ResMut<UiScale>,
) {
    for event in resize_reader.read() {
        let dimensions = Vec2::new(event.width, event.height);
        *ui_scale = UiScale(compute_ui_scale(dimensions));

        break;
    }
}

pub fn world_to_ui_position(pos: Vec2, camera: &Camera, camera_transform: &GlobalTransform, ui_scale: &UiScale) -> Vec2 {
    let viewport_position = camera.world_to_viewport(camera_transform, Vec3::new(pos.x, pos.y, 0.0)).unwrap();

    let viewport_rect = camera.logical_viewport_rect().unwrap();

    let ui_position = (viewport_position - viewport_rect.min) / ui_scale.0;

    ui_position
}

pub fn logical_to_ui_position(pos: Vec2, camera: &Camera, ui_scale: &UiScale) -> Vec2 {
    let viewport_rect = camera.logical_viewport_rect().unwrap();

    let ui_position = (pos - viewport_rect.min) / ui_scale.0;

    ui_position
}