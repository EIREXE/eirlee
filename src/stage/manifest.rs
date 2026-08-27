use std::collections::HashMap;

use bevy_asset_loader::asset_collection::AssetCollection;
use serde::{Deserialize, Serialize};
use strum_macros::{EnumIter};
use strum::IntoEnumIterator;

use crate::{
    AppState, game_settings::GameSettings, math::{int::FGi32, vec::FGVec2}, stage::{StagePoly, line::{StageCollision, StagePolyLineSegmentType, StagePolyType}},
};
use bevy::prelude::*;

#[derive(Reflect, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Hash, Debug, EnumIter)]
pub enum StageId {
    TestStage,
}

#[derive(Serialize, Deserialize, Clone, Reflect)]
#[reflect(opaque)]
pub struct StagePolyDefinition {
    poly_type: StagePolyType,
    lines: Vec<(FGVec2, StagePolyLineSegmentType)>,
}

#[derive(Serialize, Deserialize, Clone, Asset, Reflect)]
pub struct StageManifest {
    pub id: StageId,
    pub model_path: String,
    pub polygons: Vec<StagePolyDefinition>,
}

#[derive(Resource)]
pub struct StageManifestAssets(pub Vec<Handle<StageManifest>>);

impl StageManifestAssets {
    fn requested(world: &World) -> Self {
        let settings = world.resource::<GameSettings>();
        let asset_server = world.resource::<AssetServer>();

        Self(
            settings
                .stage_manifest_paths
                .iter()
                .map(|path| asset_server.load(path.clone()))
                .collect(),
        )
    }
}

impl AssetCollection for StageManifestAssets {
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
pub struct StageManifestRegistry(HashMap<StageId, Handle<StageManifest>>);

impl StageManifestRegistry {
    pub fn get(&self, stage: StageId) -> Option<Handle<StageManifest>> {
        self.0.get(&stage).cloned()
    }
}

fn fail(next_state: &mut NextState<AppState>, message: &str) {
    error!("Stage manifest preparation failed: {message}");
    next_state.set(AppState::CommonAssetLoadFailed);
}


pub fn prepare_stage_manifests(
    mut commands: Commands,
    handles: Res<StageManifestAssets>,
    manifests: Res<Assets<StageManifest>>,
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
        if registry.insert(manifest.id, handle.clone()).is_some() {
            fail(
                &mut next_state,
                &format!("duplicate manifest for stage {:?}", manifest.id),
            );
            return;
        }
    }

    for stage in StageId::iter() {
        if !registry.contains_key(&stage) {
            fail(
                &mut next_state,
                &format!("no manifest configured for stage {stage:?}"),
            );
            return;
        }
    }

    commands.insert_resource(StageManifestRegistry(registry));
    next_state.set(AppState::Idle);
}

impl StageManifest {
    pub fn to_collision(&self) -> StageCollision {
        let stage_polys: Vec<StagePoly> = self.polygons.iter().map(|def| {
            StagePoly::build(def.poly_type, &def.lines)
        }).collect();
        
        StageCollision {
            stage_polys
        }
    }
}