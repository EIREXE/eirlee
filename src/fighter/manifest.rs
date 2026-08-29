use std::collections::HashMap;

use bevy::{asset::UntypedHandle, prelude::*};
use bevy_asset_loader::asset_collection::AssetCollection;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    AppState, fighter::{FighterAttributes, FighterCameraProfile, animation::AnimKind, attack::AttackKind}, game_settings::GameSettings,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect, EnumIter)]
pub enum FighterId {
    TestFighter,
}

#[derive(Serialize, Deserialize, Clone, Asset, Reflect)]
pub struct FighterManifest {
    pub id: FighterId,
    pub model_path: String,
    pub baked_animation_path: String,
    pub attributes: FighterAttributes,
    pub camera: FighterCameraProfile,
    pub animations: HashMap<AnimKind, String>,
    pub attack_scripts: HashMap<AttackKind, String>,
}

#[derive(Resource)]
pub struct FighterManifestAssets(pub Vec<Handle<FighterManifest>>);

impl FighterManifestAssets {
    fn requested(world: &World) -> Self {
        let settings = world.resource::<GameSettings>();
        let asset_server = world.resource::<AssetServer>();

        Self(
            settings
                .fighter_manifest_paths
                .iter()
                .map(|path| asset_server.load(path.clone()))
                .collect(),
        )
    }
}

impl AssetCollection for FighterManifestAssets {
    fn create(world: &mut World) -> Self {
        Self::requested(world)
    }

    fn load(world: &mut World) -> Vec<UntypedHandle> {
        Self::requested(world)
            .0
            .into_iter()
            .map(Handle::untyped)
            .collect()
    }
}

#[derive(Resource)]
pub struct FighterManifestRegistry(HashMap<FighterId, Handle<FighterManifest>>);

impl FighterManifestRegistry {
    pub fn get(&self, fighter: FighterId) -> Option<&Handle<FighterManifest>> {
        self.0.get(&fighter)
    }
}

pub fn prepare_fighter_manifests(
    mut commands: Commands,
    handles: Res<FighterManifestAssets>,
    manifests: Res<Assets<FighterManifest>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let mut registry = HashMap::with_capacity(handles.0.len());

    for handle in &handles.0 {
        let Some(manifest) = manifests.get(handle) else {
            fail(&mut next_state, "a loaded fighter manifest is unavailable");
            return;
        };
        if manifest.model_path.is_empty() {
            fail(
                &mut next_state,
                &format!("{:?} has an empty model path", manifest.id),
            );
            return;
        }
        if manifest.baked_animation_path.is_empty() {
            fail(
                &mut next_state,
                &format!("{:?} has an empty baked animation path", manifest.id),
            );
            return;
        }
        if !manifest.animations.contains_key(&AnimKind::Wait) {
            fail(
                &mut next_state,
                &format!("{:?} has no Wait animation", manifest.id),
            );
            return;
        }
        if !manifest.camera.is_valid() {
            fail(
                &mut next_state,
                &format!("{:?} has an invalid camera profile", manifest.id),
            );
            return;
        }
        if registry.insert(manifest.id, handle.clone()).is_some() {
            fail(
                &mut next_state,
                &format!("duplicate manifest for {:?}", manifest.id),
            );
            return;
        }
    }

    for fighter in FighterId::iter() {
        if !registry.contains_key(&fighter) {
            fail(
                &mut next_state,
                &format!("no manifest configured for {fighter:?}"),
            );
            return;
        }
    }

    commands.insert_resource(FighterManifestRegistry(registry));
    next_state.set(AppState::LoadStageManifests);
}

fn fail(next_state: &mut NextState<AppState>, message: &str) {
    error!("Fighter manifest preparation failed: {message}");
    next_state.set(AppState::CommonAssetLoadFailed);
}
