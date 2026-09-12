use std::collections::HashMap;

use bevy::{asset::UntypedHandle, prelude::*};
use bevy_asset_loader::asset_collection::AssetCollection;
use ron_asset_manager::prelude::RonAsset;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    AppState,
    fighter::{FighterAttributes, FighterCameraProfile, animation::AnimKind, attack::AttackKind},
    game_settings::GameSettings,
    math::{int::FGi32, vec3::FGVec3},
    texture_reference::TextureReference,
};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect, EnumIter, Default,
)]
pub enum FighterId {
    #[default]
    TestFighter,
}

#[derive(Serialize, Deserialize, Clone, Asset, Reflect, Default)]
pub struct FighterHurtbox {
    pub bone: String,
    pub offset: FGVec3,
    pub rotation: FGVec3,
    #[reflect(ignore)]
    pub half_length: FGi32,
    #[reflect(ignore)]
    pub radius: FGi32,
}

impl FighterHurtbox {
    pub fn validate(&self) -> Result<(), String> {
        if self.bone.trim().is_empty() {
            return Err("bone name is empty".into());
        }
        if self.radius <= FGi32::ZERO {
            return Err("radius must be positive".into());
        }
        if self.half_length < FGi32::ZERO {
            return Err("half-length must be non-negative".into());
        }
        Ok(())
    }
}

#[derive(Clone, Asset, Deserialize, Reflect, RonAsset)]
pub struct FighterManifest {
    pub id: FighterId,
    pub model_path: String,
    #[reflect(ignore)]
    pub model_scale: FGi32,
    pub baked_animation_path: String,
    #[asset]
    #[dependency]
    #[reflect(ignore)]
    pub icon: TextureReference,
    pub attributes: FighterAttributes,
    pub camera: FighterCameraProfile,
    pub animations: HashMap<AnimKind, String>,
    pub attack_scripts: HashMap<AttackKind, String>,
    pub hurtboxes: Vec<FighterHurtbox>,
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

#[derive(Resource, Deref)]
pub struct FighterManifestRegistry(pub HashMap<FighterId, Handle<FighterManifest>>);

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
        if manifest.model_scale <= FGi32::ZERO {
            fail(
                &mut next_state,
                &format!("{:?} has a non-positive model scale", manifest.id),
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

        for anim_kind in AnimKind::iter() {
            if !manifest.animations.contains_key(&anim_kind) {
                fail(
                    &mut next_state,
                    &format!("{:?} has no {:?} animation", manifest.id, anim_kind),
                );
                return;
            }
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
