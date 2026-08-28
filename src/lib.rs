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

use bevy::{asset::processor::{AssetProcessor, ProcessorState}, prelude::*, tasks::block_on};
use bevy_asset_loader::prelude::*;
use bevy_wind_waker_shader::prelude::*;
use clap::Parser;
use jackdaw_runtime::EditorCategory;

use crate::{
    camera::MatchCameraPlugin, fighter::manifest::FighterManifest, game_settings::GameSettings, schedule::GameplaySchedulePlugin, stage::manifest::{StageManifest, StageManifestAssets},
};

mod args;
pub mod camera;
pub mod fighter;
pub mod game_settings;
pub mod input;
pub mod match_loading;
mod math;
pub mod netcode;
pub mod player;
pub mod schedule;
pub mod stage;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    WaitForAssetProcessing,
    LoadCommonAssets,
    LoadFighterManifests,
    PrepareFighterManifests,
    LoadStageManifests,
    PrepareStageManifests,
    CommonAssetLoadFailed,
    Idle,
    LoadingMatch,
    PreparingMatch,
    InMatch,
    MatchLoadFailed,
}

/// Your game's Bevy plugin. The editor finds it by this name (override
/// with `plugin = "..."` in jackdaw.toml) and runs it on Play; the
/// standalone binary adds it in `main.rs`.
#[derive(Default)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // Each module owns its own systems, rollback registrations and
        // resources; this plugin only wires them together and supplies the
        // game-level content and configuration.
        app.init_state::<AppState>()
            .add_plugins((
                WindWakerShaderPlugin::default(),
                // Brings up GGRS (and with it `GgrsSchedule`), so it goes first.
                netcode::FighterNetcodePlugin::default(),
                GameplaySchedulePlugin,
                input::FighterInputPlugin,
                fighter::FighterPlugin,
                fighter::baked_animation::BakedAnimationPlugin,
                stage::StagePlugin,
                MatchCameraPlugin,
                bevy_common_assets::ron::RonAssetPlugin::<FighterManifest>::new(&["fighter.ron"]),
                bevy_common_assets::ron::RonAssetPlugin::<StageManifest>::new(&["stage.ron"]),
                bevy_common_assets::ron::RonAssetPlugin::<GameSettings>::new(&["ron"]),
            ))
            .add_loading_state(
                LoadingState::new(AppState::LoadingMatch)
                    .continue_to_state(AppState::PreparingMatch)
                    .on_failure_continue_to_state(AppState::MatchLoadFailed)
                    .load_collection::<match_loading::MatchAssets>(),
            )
            .add_loading_state(
                LoadingState::new(AppState::LoadCommonAssets)
                    .continue_to_state(AppState::LoadFighterManifests)
                    .on_failure_continue_to_state(AppState::CommonAssetLoadFailed)
                    .load_collection::<game_settings::CommonAssets>()
                    .finally_init_resource::<GameSettings>(),
            )
            .add_loading_state(
                LoadingState::new(AppState::LoadFighterManifests)
                    .continue_to_state(AppState::PrepareFighterManifests)
                    .on_failure_continue_to_state(AppState::CommonAssetLoadFailed)
                    .load_collection::<fighter::manifest::FighterManifestAssets>(),
            )
            .add_loading_state(
                LoadingState::new(AppState::LoadStageManifests)
                    .continue_to_state(AppState::PrepareStageManifests)
                    .on_failure_continue_to_state(AppState::CommonAssetLoadFailed)
                    .load_collection::<stage::manifest::StageManifestAssets>(),
            )
            .add_systems(
                Update,
                wait_for_asset_processor.run_if(in_state(AppState::WaitForAssetProcessing)),
            )
            .add_systems(
                OnEnter(AppState::PrepareFighterManifests),
                fighter::manifest::prepare_fighter_manifests,
            )
            .add_systems(
                OnEnter(AppState::PrepareStageManifests),
                stage::manifest::prepare_stage_manifests,
            )
            .add_systems(
                OnEnter(AppState::Idle),
                match_loading::initiate_default_match,
            )
            .add_systems(
                OnEnter(AppState::PreparingMatch),
                match_loading::prepare_match,
            )
            .add_systems(OnExit(AppState::InMatch), match_loading::cleanup_match)
            .add_systems(Update, spin_cubes)
            .insert_resource(args::Args::parse());
    }
}

fn wait_for_asset_processor(
    processor: Option<Res<AssetProcessor>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if processor.is_none_or(|processor| block_on(processor.get_state()) == ProcessorState::Finished) {
        next_state.set(AppState::LoadCommonAssets);
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
