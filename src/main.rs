//! Standalone binary for game-test: `cargo run` plays the game
//! without the editor. The editor's Play button runs the same
//! [`game_test::GamePlugin`] through its own game runner instead.

use bevy::asset::AssetMode::Processed;
use bevy::camera_controller::free_camera::FreeCameraPlugin;
use bevy::image::{ImageAddressMode, ImagePlugin, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() -> AppExit {
    // A `Repeat` sampler so the tileable materials authored in the
    // editor render the same way at runtime; Bevy's default
    // `ClampToEdge` smears texture edges on brush geometry.
    let default_plugins = DefaultPlugins
        .set(ImagePlugin {
            default_sampler: ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                address_mode_w: ImageAddressMode::Repeat,
                ..ImageSamplerDescriptor::linear()
            },
        })
        .set(AssetPlugin {
            mode: Processed,
            ..default()
        });

    App::new()
        .add_plugins(default_plugins)
        .add_plugins(avian3d::prelude::PhysicsPlugins::default())
        .add_plugins(game_test::GamePlugin)
        .add_plugins(FreeCameraPlugin)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::default()) // adds default options and `InspectorEguiImpl`s
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .run()
}
