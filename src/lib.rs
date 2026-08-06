//! game-test: gameplay shared between `cargo run` and the
//! editor's Play button.
//!
//! Scene content (entities, components, positions) lives in
//! `assets/*.bsn` files authored in the editor. Game behaviour
//! (systems, observers, resources) lives in [`GamePlugin`].
//!
//! # Try it
//!
//! 1. Open this project in jackdaw.
//! 2. Hierarchy: right-click, `Add > Cube`.
//! 3. Inspector: `Add Component > SpinningCube`. Set `speed` to `1.5`.
//! 4. `File > Save`, then click Play. The cube spins.
//! 5. `cargo run` launches the same game without the editor.
//!
//! # Adding your own components
//!
//! Write a component anywhere in this library (any module, not
//! `main.rs`), deriving `Component, Reflect, Default` with
//! `#[reflect(Component, Default)]`, like `SpinningCube` below. After you
//! save, click Rebuild in jackdaw (or run `jd build`) and it
//! appears in `Add Component`. No registration code is needed.

use bevy::prelude::*;
use jackdaw_runtime::prelude::*;

use crate::{fighter::{states::wait::WaitState, *}, game_settings::{FighterSettingsCommon, GameSettings, InputSettingsCommon}, stage::debug_draw_scene};

pub mod fighter;
pub mod input;
pub mod stage;
pub mod game_settings;

/// Your game's Bevy plugin. The editor finds it by this name (override
/// with `plugin = "..."` in jackdaw.toml) and runs it on Play; the
/// standalone binary adds it in `main.rs`.
#[derive(Default)]
pub struct GamePlugin;

pub fn setup(mut commands: Commands) {
    let fighter = commands
        .spawn((
            Name("Fighter".into()),
            FighterECB {
                vertical_half: 0.5,
                horizontal_half: 0.25,
            },
            FighterVelocity::default(),
            FighterTranslation(Vec2::new(0.5, 1.25)),
            FighterAttributes {
                base_walk_accel: 0.01,
                stick_walk_accel: 0.02,
                max_walk_vel: 0.16,
                ground_friction: 0.008,
                dash_duration: 15
            },
            WaitState {},
            input::player::FighterInput::default()
        ))
        .id();
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, stage::test_scene.spawn())
            .add_systems(Startup, setup)
            .add_systems(Update, spin_cubes)
            .add_systems(Update, debug_draw_scene)
            .add_plugins(input::plugin::FighterInputPlugin)
            .add_plugins(fighter::FighterPlugin)
            .insert_resource(GameSettings {
                figher_common: FighterSettingsCommon {
                    walk_speed_ease: 0.5,
                    ground_max_horizontal_velocity: 0.3
                },
                input_common: InputSettingsCommon::default(),
            });
    }
}

/// Spin-rate in radians per second. Attach it in the inspector; the
/// editor sees this component with no registration code because Bevy's
/// `reflect_auto_register` picks up the `Reflect` derive. `Default` +
/// `#[reflect(Default)]` let the editor add it and the game reconstruct
/// it with sensible starting values.
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default, @EditorCategory::new("Actor"))]
pub struct SpinningCube {
    pub speed: f32,
}

fn spin_cubes(time: Res<Time>, mut cubes: Query<(&SpinningCube, &mut Transform)>) {
    let dt = time.delta_secs();
    for (cube, mut transform) in &mut cubes {
        transform.rotate_y(cube.speed * dt);
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default, @EditorCategory::new("Actor"))]
pub struct TestCube {
    pub frog: f32,
}
