use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use serde::{Deserialize, Serialize};

use crate::math::int::FGi32;

#[derive(Reflect, Serialize, Deserialize, Clone)]
#[reflect(opaque)]
pub struct FighterSettingsCommon {
    pub walk_speed_ease: FGi32,
    pub ground_max_horizontal_velocity: FGi32,
    pub ground_friction_over_walk_speed_multiplier: FGi32,
}
#[derive(Reflect, Serialize, Deserialize, Clone)]
#[reflect(opaque)]
pub struct InputSettingsCommon {
    pub stick_deadzone: FGi32,
    pub trigger_deadzone: FGi32,
    pub smash_input_reset_axis_threshold: FGi32,
    pub smash_input_axis_threshold: FGi32,
    pub run_stick_threshold: FGi32,
    pub smash_input_frame_threshold: u8,
    pub input_buffer_size: u8,
}

#[derive(Reflect, Serialize, Deserialize, Resource, Asset, Clone)]
pub struct GameSettings {
    pub fighter_common: FighterSettingsCommon,
    pub input_common: InputSettingsCommon,
    pub fighter_manifest_paths: Vec<String>,
    pub stage_manifest_paths: Vec<String>,
}

#[derive(AssetCollection, Resource)]
pub struct CommonAssets {
    #[asset(path = "game_settings.ron")]
    game_settings: Handle<GameSettings>,
}

impl FromWorld for GameSettings {
    fn from_world(world: &mut World) -> Self {
        let game_settings_handle = world.resource::<CommonAssets>().game_settings.clone();
        world
            .resource::<Assets<GameSettings>>()
            .get(&game_settings_handle)
            .expect("GameSettings should already be loaded! Bug?")
            .clone()
    }
}

#[derive(Reflect, Resource, Default)]
#[reflect(Resource)]
pub struct DebugSettings {
    pub show_attack_hitboxes: bool,
    pub show_stage_lines: bool,
    pub show_ecb: bool,
    pub debug_gizmos_draw_in_front: bool
}