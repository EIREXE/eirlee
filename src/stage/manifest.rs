use std::collections::HashMap;

use bevy_asset_loader::asset_collection::AssetCollection;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    AppState,
    game_settings::GameSettings,
    math::vec::FGVec2,
    stage::{
        StagePoly,
        line::{StageCollision, StagePolyLineSegmentType, StagePolyType},
    },
};
use bevy::prelude::*;

/// Per-stage presentation tuning for the standard match camera.
///
/// Spatial values use gameplay world units on the XY plane. Angle values use
/// degrees
#[derive(Component, Serialize, Deserialize, Clone, Reflect)]
#[reflect(opaque)]
pub struct StageCameraProfile {
    /// World-space left edge of the camera range.
    pub left: f32,
    /// World-space right edge of the camera range.
    pub right: f32,
    /// World-space bottom edge of the camera range.
    pub bottom: f32,
    /// World-space top edge of the camera range.
    pub top: f32,
    /// Neutral focus point used for empty framing and dynamic pan calculations.
    pub origin: Vec2,
    /// Vertical perspective field of view for the standard match camera.
    pub vertical_fov_degrees: f32,
    /// Nearest allowed positive Z distance from the gameplay plane.
    pub min_depth: f32,
    /// Farthest allowed positive Z distance from the gameplay plane.
    pub max_depth: f32,
    /// Multiplies fighter camera extents after the player-count scale.
    pub subject_scale: f32,
    /// Multiplies the facing-forward horizontal fighter camera extent.
    pub fighter_forward_extent_scale: f32,
    /// Multiplies the standard camera's interest and eye follow speeds.
    pub tracking_smoothness: f32,
    /// Constant vertical pitch added after dynamic vertical pan is clamped.
    pub vertical_pan_degrees: f32,
    /// Dynamic horizontal pan in degrees for every world unit from 0,0
    pub horizontal_pan_degrees_per_unit: f32,
    /// Dynamic vertical pan in degrees for every world unit from the vertical reference line
    pub vertical_pan_degrees_per_unit: f32,
    /// Symmetric limit on dynamic horizontal pan, before any other adjustment.
    pub max_horizontal_pan_degrees: f32,
    /// Maximum upward dynamic vertical pan, before constant pitch is added.
    pub max_upward_pan_degrees: f32,
    /// Magnitude of the maximum downward dynamic vertical pan, before constant
    /// pitch is added.
    pub max_downward_pan_degrees: f32,
}

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
    pub camera: StageCameraProfile,
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
        let stage_polys: Vec<StagePoly> = self
            .polygons
            .iter()
            .map(|def| StagePoly::build(def.poly_type, &def.lines))
            .collect();

        StageCollision { stage_polys }
    }
}

impl StageCameraProfile {
    fn is_valid(&self) -> bool {
        self.left.is_finite()
            && self.right.is_finite()
            && self.bottom.is_finite()
            && self.top.is_finite()
            && self.origin.is_finite()
            && self.vertical_fov_degrees.is_finite()
            && self.min_depth.is_finite()
            && self.max_depth.is_finite()
            && self.subject_scale.is_finite()
            && self.fighter_forward_extent_scale.is_finite()
            && self.tracking_smoothness.is_finite()
            && self.vertical_pan_degrees.is_finite()
            && self.horizontal_pan_degrees_per_unit.is_finite()
            && self.vertical_pan_degrees_per_unit.is_finite()
            && self.max_horizontal_pan_degrees.is_finite()
            && self.max_upward_pan_degrees.is_finite()
            && self.max_downward_pan_degrees.is_finite()
            && self.left < self.right
            && self.bottom < self.top
            && (self.left..=self.right).contains(&self.origin.x)
            && (self.bottom..=self.top).contains(&self.origin.y)
            && (0.0..180.0).contains(&self.vertical_fov_degrees)
            && self.min_depth > 0.0
            && self.min_depth <= self.max_depth
            && self.subject_scale > 0.0
            && self.fighter_forward_extent_scale > 0.0
            && self.tracking_smoothness >= 0.0
            && self.max_horizontal_pan_degrees >= 0.0
            && self.max_upward_pan_degrees >= 0.0
            && self.max_downward_pan_degrees >= 0.0
            && self.vertical_fov_degrees * 0.5 + self.max_horizontal_pan_degrees < 90.0
            && self.vertical_fov_degrees * 0.5
                + self.vertical_pan_degrees.abs()
                + self
                    .max_upward_pan_degrees
                    .max(self.max_downward_pan_degrees)
                < 90.0
    }
}
