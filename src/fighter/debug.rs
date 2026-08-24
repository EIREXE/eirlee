//! Fighter debug views: gizmos and the egui inspector window.

use bevy::{color::palettes::css::YELLOW, prelude::*};
use bevy_egui::{EguiContexts, egui};

use crate::fighter::state::StateNameDebug;
use crate::fighter::{FighterECB, FighterTranslation, FighterVelocity};
use crate::input::FighterInput;

pub fn debug_draw_ecb(ecbs: Query<(&FighterECB, &FighterTranslation)>, mut gizmos: Gizmos) {
    for (ecb, trf) in ecbs {
        let transformed = ecb.to_3d_lineloop(Transform::from_xyz(trf.x, trf.y, 0.0));
        gizmos.lineloop(transformed, YELLOW);
    }
}

pub fn fighter_debug(
    mut contexts: EguiContexts,
    query: Query<(&FighterVelocity, &FighterInput, &StateNameDebug)>,
) -> Result {
    for (vel, input, name) in query {
        egui::Window::new("Hello").show(contexts.ctx_mut()?, |ui| {
            ui.label(format!("State: {}", name.0));
            ui.label(format!("{:?}", vel));
            ui.label(format!("{:?}", input.get_last_frame()));
        });
    }

    Ok(())
}

pub fn update_config(
    mut config_store: ResMut<GizmoConfigStore>,
    keyboard: Res<ButtonInput<KeyCode>>,
    real_time: Res<Time<Real>>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    if keyboard.just_pressed(KeyCode::KeyT) {
        for (_, config, _) in config_store.iter_mut() {
            config.depth_bias = if config.depth_bias == 0. { -1. } else { 0. };
        }
    }
}