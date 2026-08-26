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

use bevy::{mesh::SphereMeshBuilder, prelude::*};
use bevy_wind_waker_shader::prelude::*;
use clap::Parser;
use jackdaw_runtime::prelude::*;

use crate::{
    fighter::{
        FighterAttributes, FighterECB, FighterTranslation, FighterVelocity, FighterVisual, animation::{AnimManifest}, state::{FighterState, fall::FallState, wait::WaitState}, visual::resolve_character_assets,
    }, game_settings::{FighterSettingsCommon, GameSettings, InputSettingsCommon}, input::FighterInput, math::{int::FGi32, vec::FGVec2}, player::Player, schedule::GameplaySchedulePlugin,
};

mod args;
pub mod fighter;
pub mod game_settings;
pub mod input;
mod math;
pub mod netcode;
pub mod player;
pub mod schedule;
pub mod stage;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    LoadingCharacters,
    InGame,
}

/// Your game's Bevy plugin. The editor finds it by this name (override
/// with `plugin = "..."` in jackdaw.toml) and runs it on Play; the
/// standalone binary adds it in `main.rs`.
#[derive(Default)]
pub struct GamePlugin;

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // debug stage

    let _stage = commands.spawn((
        WorldAssetRoot(asset_server.load("stages/YoshiStory.glb#Scene0")),
    ));
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // Each module owns its own systems, rollback registrations and
        // resources; this plugin only wires them together and supplies the
        // game-level content and configuration.
        app
        .init_state::<AppState>()
        .add_plugins((
            WindWakerShaderPlugin::default(),
            // Brings up GGRS (and with it `GgrsSchedule`), so it goes first.
            netcode::FighterNetcodePlugin::default(),
            GameplaySchedulePlugin,
            input::FighterInputPlugin,
            fighter::FighterPlugin,
            stage::StagePlugin,
            bevy_common_assets::ron::RonAssetPlugin::<AnimManifest>::new(&[])
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, spin_cubes)
        .insert_resource(args::Args::parse())
        .insert_resource(GameSettings {
            figher_common: FighterSettingsCommon {
                walk_speed_ease: FGi32::lit("0.5"),
                ground_max_horizontal_velocity: FGi32::lit("3.0"),
                ground_friction_over_walk_speed_multiplier: FGi32::lit("2.0"),
            },
            input_common: InputSettingsCommon::default(),
        })
        .add_systems(OnEnter(AppState::LoadingCharacters), crate::fighter::visual::start_loading)
        .add_systems(
            Update,
            resolve_character_assets.run_if(in_state(AppState::LoadingCharacters)),
        );
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
