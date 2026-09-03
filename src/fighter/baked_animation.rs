//! Deterministic, fighter-local skeletal animation samples for gameplay and rendering.

use std::{
    collections::{HashMap, HashSet},
    io,
    path::Path,
};

use bevy::{
    asset::{
        AssetLoader, AssetPath, AsyncWriteExt, LoadContext,
        io::{Reader, Writer},
        processor::LoadTransformAndSave,
        saver::{AssetSaver, SavedAsset},
        transformer::{AssetTransformer, TransformedAsset},
    },
    prelude::*,
    reflect::TypePath,
    world_serialization::{WorldInstanceReady, WorldInstanceSpawner},
};
use bevy_ggrs::prelude::GgrsSchedule;
use serde::{Deserialize, Serialize};

use crate::{
    fighter::{
        animation::{AnimKind, FighterAnimationFrame},
        manifest::FighterManifest,
        visual::FighterAnimations,
    },
    math::{
        int::{FGWide, FGi32, FGi32Ext},
        vec3::FGVec3,
    },
    schedule::GameplaySet,
};

pub const BAKED_ANIMATION_FPS: u32 = 60;

/// Parent transforms are composed after
/// quantization, which makes every runtime sample platform-independent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct FixedMat4 {
    pub cols: [[FGi32; 4]; 4],
}

impl FixedMat4 {
    pub const IDENTITY: Self = Self {
        cols: [
            [FGi32::ONE, FGi32::ZERO, FGi32::ZERO, FGi32::ZERO],
            [FGi32::ZERO, FGi32::ONE, FGi32::ZERO, FGi32::ZERO],
            [FGi32::ZERO, FGi32::ZERO, FGi32::ONE, FGi32::ZERO],
            [FGi32::ZERO, FGi32::ZERO, FGi32::ZERO, FGi32::ONE],
        ],
    };
    fn from_mat4(matrix: Mat4) -> io::Result<Self> {
        let values = matrix.to_cols_array();
        let mut cols = [[FGi32::ZERO; 4]; 4];
        for (index, value) in values.into_iter().enumerate() {
            cols[index / 4][index % 4] = FGi32::checked_from_num(value).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "animation transform overflows Q16.16",
                )
            })?;
        }
        Ok(Self { cols })
    }

    pub fn to_mat4(self) -> Mat4 {
        Mat4::from_cols_array_2d(&self.cols.map(|column| column.map(|value| value.to_num())))
    }

    pub fn mul(self, rhs: Self) -> Self {
        let mut cols = [[FGi32::ZERO; 4]; 4];
        for column in 0..4 {
            for row in 0..4 {
                let mut value = FGWide::ZERO;
                for k in 0..4 {
                    value +=
                        FGWide::from_num(self.cols[k][row]) * FGWide::from_num(rhs.cols[column][k]);
                }
                cols[column][row] = FGi32::from_num(value);
            }
        }
        Self { cols }
    }

    pub fn transform_point(self, point: FGVec3) -> FGVec3 {
        let input = [point.x, point.y, point.z, FGi32::ONE];
        let mut output = [FGi32::ZERO; 3];
        for row in 0..3 {
            let mut value = FGWide::ZERO;
            for (column, input) in input.into_iter().enumerate() {
                value += FGWide::from_num(self.cols[column][row]) * FGWide::from_num(input);
            }
            output[row] = FGi32::from_num(value);
        }
        FGVec3 {
            x: output[0],
            y: output[1],
            z: output[2],
        }
    }

    pub fn translate(&mut self, translation: FGVec3) {
        self.cols[3][0] = translation.x;
        self.cols[3][1] = translation.y;
        self.cols[3][2] = translation.z;
    }

    pub fn with_translation(mut self, translation: FGVec3) -> Self {
        self.translate(translation);
        self
    }

    /// Replaces the rotation with XYZ Euler angles expressed in radians.
    pub fn with_rotation(mut self, rotation: FGVec3) -> Self {
        let (sin_x, cos_x) = rotation.x.sin_cos();
        let (sin_y, cos_y) = rotation.y.sin_cos();
        let (sin_z, cos_z) = rotation.z.sin_cos();
        self.cols[0][0] = cos_y * cos_z;
        self.cols[0][1] = cos_x * sin_z + sin_x * sin_y * cos_z;
        self.cols[0][2] = sin_x * sin_z - cos_x * sin_y * cos_z;
        self.cols[1][0] = -cos_y * sin_z;
        self.cols[1][1] = cos_x * cos_z - sin_x * sin_y * sin_z;
        self.cols[1][2] = sin_x * cos_z + cos_x * sin_y * sin_z;
        self.cols[2][0] = sin_y;
        self.cols[2][1] = -sin_x * cos_y;
        self.cols[2][2] = cos_x * cos_y;
        self
    }

    pub fn get_translation(&self) -> FGVec3 {
        FGVec3::new(self.cols[3][0], self.cols[3][1], self.cols[3][2])
    }

    pub fn rotate_y(self, quarter_turns: u8) -> Self {
        let rotation = match quarter_turns % 4 {
            0 => Self {
                cols: [
                    [FGi32::ONE, FGi32::ZERO, FGi32::ZERO, FGi32::ZERO],
                    [FGi32::ZERO, FGi32::ONE, FGi32::ZERO, FGi32::ZERO],
                    [FGi32::ZERO, FGi32::ZERO, FGi32::ONE, FGi32::ZERO],
                    [FGi32::ZERO, FGi32::ZERO, FGi32::ZERO, FGi32::ONE],
                ],
            },
            1 => Self {
                cols: [
                    [FGi32::ZERO, FGi32::ZERO, FGi32::NEG_ONE, FGi32::ZERO],
                    [FGi32::ZERO, FGi32::ONE, FGi32::ZERO, FGi32::ZERO],
                    [FGi32::ONE, FGi32::ZERO, FGi32::ZERO, FGi32::ZERO],
                    [FGi32::ZERO, FGi32::ZERO, FGi32::ZERO, FGi32::ONE],
                ],
            },
            2 => Self {
                cols: [
                    [FGi32::NEG_ONE, FGi32::ZERO, FGi32::ZERO, FGi32::ZERO],
                    [FGi32::ZERO, FGi32::ONE, FGi32::ZERO, FGi32::ZERO],
                    [FGi32::ZERO, FGi32::ZERO, FGi32::NEG_ONE, FGi32::ZERO],
                    [FGi32::ZERO, FGi32::ZERO, FGi32::ZERO, FGi32::ONE],
                ],
            },
            _ => Self {
                cols: [
                    [FGi32::ZERO, FGi32::ZERO, FGi32::ONE, FGi32::ZERO],
                    [FGi32::ZERO, FGi32::ONE, FGi32::ZERO, FGi32::ZERO],
                    [FGi32::NEG_ONE, FGi32::ZERO, FGi32::ZERO, FGi32::ZERO],
                    [FGi32::ZERO, FGi32::ZERO, FGi32::ZERO, FGi32::ONE],
                ],
            },
        };
        self.mul(rotation)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BakedAnimationClip {
    /// One parent-relative matrix per stored node, for each 60 Hz simulation frame.
    pub frames: Vec<Vec<FixedMat4>>,
}

#[derive(Asset, TypePath, Clone, Debug, Serialize, Deserialize)]
pub struct BakedFighterAnimations {
    pub frame_rate: u32,
    /// Indexed exactly like every frame in each clip.
    pub bone_names: Vec<String>,
    /// Parent node for each stored node, indexed like `bone_names`.
    pub parents: Vec<Option<usize>>,
    clips: Vec<BakedAnimationClip>,
    clip_indices: HashMap<AnimKind, usize>,
}

impl BakedFighterAnimations {
    pub fn sample_pose(&self, frame: &FighterAnimationFrame) -> Option<&[FixedMat4]> {
        let frames = &self.clip(frame.kind)?.frames;
        let frame_index = if frame.repeat {
            frame.frame % frames.len() as u32
        } else {
            frame.frame
        };
        frames
            .get(frame_index as usize)
            .or_else(|| frames.last())
            .map(Vec::as_slice)
    }

    pub fn frame_count(&self, kind: AnimKind) -> Option<u32> {
        self.clip(kind).map(|clip| clip.frames.len() as u32)
    }

    fn clip(&self, kind: AnimKind) -> Option<&BakedAnimationClip> {
        self.clip_indices
            .get(&kind)
            .and_then(|index| self.clips.get(*index))
    }
}

/// The current baked model-space pose for a fighter, recalculated every
/// rollback frame from its animation clock and immutable baked asset.
#[derive(Component, Clone, Debug, Default)]
pub struct FighterBoneMatrices {
    bone_names: Vec<String>,
    parents: Vec<Option<usize>>,
    matrices: Vec<FixedMat4>,
    previous_frame_matrices: Vec<FixedMat4>,
    local_matrices: Vec<FixedMat4>,
}

#[derive(Component)]
struct BakedFighterBone {
    fighter: Entity,
    index: usize,
}

impl FighterBoneMatrices {
    pub fn get(&self, bone: &str) -> Option<(FixedMat4, FixedMat4)> {
        let index = self.bone_names.iter().position(|name| name == bone)?;
        let curr = self.matrices.get(index).copied();
        let prev = self.previous_frame_matrices.get(index).copied();

        if let Some(prev) = prev {
            if let Some(curr) = curr {
                return Some((prev, curr));
            }
        }

        None
    }

    pub fn get_current(&self, bone: &str) -> Option<FixedMat4> {
        let index = self.bone_names.iter().position(|name| name == bone)?;
        self.matrices.get(index).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, FixedMat4)> {
        self.bone_names
            .iter()
            .map(String::as_str)
            .zip(self.matrices.iter().copied())
    }

    pub fn local(&self, index: usize) -> Option<FixedMat4> {
        self.local_matrices.get(index).copied()
    }

    fn set_pose(&mut self, bone_names: &[String], parents: &[Option<usize>], local: &[FixedMat4]) {
        assert_eq!(
            bone_names.len(),
            local.len(),
            "baked animation pose does not match its bone names"
        );
        assert_eq!(
            bone_names.len(),
            parents.len(),
            "baked bone parents do not match names"
        );
        if self.bone_names.is_empty() {
            self.bone_names.extend_from_slice(bone_names);
            self.parents.extend_from_slice(parents);
        } else {
            assert_eq!(self.bone_names, bone_names, "baked bone names changed");
            assert_eq!(self.parents, parents, "baked bone parents changed");
        }
        std::mem::swap(&mut self.matrices, &mut self.previous_frame_matrices);
        self.local_matrices.clear();
        self.local_matrices.extend_from_slice(local);
        let mut model = vec![None; local.len()];
        for index in 0..local.len() {
            resolve_model_transform(index, parents, local, &mut model)
                .expect("baked bone hierarchy should be valid");
        }
        self.matrices = model
            .into_iter()
            .map(|matrix| matrix.expect("baked bone hierarchy should be valid"))
            .collect();
    }
}

pub fn update_fighter_bone_matrices(
    mut fighters: Query<(
        &FighterAnimationFrame,
        &FighterAnimations,
        &mut FighterBoneMatrices,
    )>,
    baked_assets: Res<Assets<BakedFighterAnimations>>,
) {
    for (frame, animations, mut matrices) in &mut fighters {
        let baked = baked_assets
            .get(&animations.baked)
            .expect("fighter baked animation asset should be loaded before the match starts");
        let pose = baked
            .sample_pose(frame)
            .expect("fighter animation frame should select a baked pose");
        matrices.set_pose(&baked.bone_names, &baked.parents, pose);
    }
}

fn bind_fighter_visual_bones(
    ready: On<WorldInstanceReady>,
    mut commands: Commands,
    spawner: Res<WorldInstanceSpawner>,
    fighters: Query<&FighterAnimations>,
    baked_assets: Res<Assets<BakedFighterAnimations>>,
    nodes: Query<(&Name, Option<&ChildOf>)>,
    animation_players: Query<(), With<AnimationPlayer>>,
) {
    let fighter = ready.entity;
    let Ok(animations) = fighters.get(fighter) else {
        return;
    };
    let baked = baked_assets
        .get(&animations.baked)
        .expect("fighter baked animation asset should be loaded before its visual scene");
    let entities = spawner
        .iter_instance_entities(ready.instance_id)
        .collect::<HashSet<_>>();
    for &entity in &entities {
        if animation_players.contains(entity) {
            commands.entity(entity).remove::<AnimationPlayer>();
        }
    }
    let mut indices = HashMap::with_capacity(baked.bone_names.len());
    for (index, name) in baked.bone_names.iter().enumerate() {
        let key = (baked.parents[index], name.as_str());
        assert!(
            indices.insert(key, index).is_none(),
            "fighter GLTF has duplicate node names under the same parent"
        );
    }

    let mut assigned = HashMap::new();
    let mut remaining = entities.iter().copied().collect::<Vec<_>>();
    while !remaining.is_empty() {
        let mut next = Vec::new();
        let mut progress = false;
        for entity in remaining {
            let Ok((name, parent)) = nodes.get(entity) else {
                continue;
            };
            let parent = parent.map(ChildOf::parent);
            let parent_index = match parent {
                Some(parent) if entities.contains(&parent) => {
                    if let Some(index) = assigned.get(&parent) {
                        Some(*index)
                    } else if nodes.get(parent).ok().is_some_and(|(name, _)| {
                        baked.bone_names.iter().any(|bone| bone == name.as_str())
                    }) {
                        next.push(entity);
                        continue;
                    } else {
                        // WorldAsset adds an unnamed instance root above the GLTF scene roots.
                        None
                    }
                }
                _ => None,
            };
            if let Some(&index) = indices.get(&(parent_index, name.as_str())) {
                assigned.insert(entity, index);
                progress = true;
            }
        }
        if !progress {
            break;
        }
        remaining = next;
    }

    if assigned.len() != baked.bone_names.len() {
        warn!(
            fighter = ?fighter,
            bound = assigned.len(),
            expected = baked.bone_names.len(),
            "could not bind every baked fighter node to its visual scene"
        );
    }
    for (entity, index) in assigned {
        commands
            .entity(entity)
            .insert(BakedFighterBone { fighter, index });
    }
}

fn apply_baked_fighter_visual_pose(
    mut nodes: Query<(&mut Transform, &BakedFighterBone)>,
    fighters: Query<&FighterBoneMatrices>,
) {
    for (mut transform, bone) in &mut nodes {
        let Ok(matrices) = fighters.get(bone.fighter) else {
            continue;
        };
        let Some(local) = matrices.local(bone.index) else {
            continue;
        };
        *transform = Transform::from_matrix(local.to_mat4());
    }
}

#[derive(Deserialize)]
struct FighterBakeDeclaration {
    manifest_path: String,
}

#[derive(Asset, TypePath)]
struct FighterBakeSource {
    model_bytes: Vec<u8>,
    animations: HashMap<AnimKind, String>,
}

#[derive(Default, TypePath)]
struct FighterBakeSourceLoader;

impl AssetLoader for FighterBakeSourceLoader {
    type Asset = FighterBakeSource;
    type Settings = ();
    type Error = io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut declaration = Vec::new();
        reader.read_to_end(&mut declaration).await?;
        let declaration: FighterBakeDeclaration = ron::de::from_bytes(&declaration)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

        let manifest_bytes = load_context
            .read_asset_bytes(&declaration.manifest_path)
            .await
            .map_err(|error| io::Error::other(error.to_string()))?;
        let manifest: FighterManifest = ron::de::from_bytes(&manifest_bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

        // The processor's AssetServer reads processed assets. Reading this
        // source dependency through it would wait for a processed GLB while
        // this processor is still producing the first processed asset.
        let model_bytes = std::fs::read(Path::new("assets").join(&manifest.model_path))?;
        Ok(FighterBakeSource {
            model_bytes,
            animations: manifest.animations,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["fighterbake"]
    }
}

#[derive(Default, TypePath)]
struct BakedFighterAnimationsLoader;

impl AssetLoader for BakedFighterAnimationsLoader {
    type Asset = BakedFighterAnimations;
    type Settings = ();
    type Error = io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &Self::Settings,
        _: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        bincode::deserialize(&bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }

    fn extensions(&self) -> &[&str] {
        &["fighterbake"]
    }
}

#[derive(Default, TypePath)]
struct FighterBakeTransformer;

impl AssetTransformer for FighterBakeTransformer {
    type AssetInput = FighterBakeSource;
    type AssetOutput = BakedFighterAnimations;
    type Settings = ();
    type Error = io::Error;

    async fn transform<'a>(
        &'a self,
        source: TransformedAsset<Self::AssetInput>,
        _: &'a Self::Settings,
    ) -> Result<TransformedAsset<Self::AssetOutput>, Self::Error> {
        let baked = bake_gltf(&source.model_bytes, &source.animations)?;
        Ok(source.replace_asset(baked))
    }
}

#[derive(Default, TypePath)]
struct BakedFighterAnimationsSaver;

impl AssetSaver for BakedFighterAnimationsSaver {
    type Asset = BakedFighterAnimations;
    type Settings = ();
    type OutputLoader = BakedFighterAnimationsLoader;
    type Error = io::Error;

    async fn save(
        &self,
        writer: &mut Writer,
        asset: SavedAsset<'_, '_, Self::Asset>,
        _: &Self::Settings,
        _: AssetPath<'_>,
    ) -> Result<(), Self::Error> {
        let bytes = bincode::serialize(&*asset)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        writer.write_all(&bytes).await
    }
}

type FighterBakeProcessor = LoadTransformAndSave<
    FighterBakeSourceLoader,
    FighterBakeTransformer,
    BakedFighterAnimationsSaver,
>;

pub struct BakedAnimationPlugin;

impl Plugin for BakedAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<BakedFighterAnimations>()
            .register_asset_loader(FighterBakeSourceLoader)
            .register_asset_loader(BakedFighterAnimationsLoader)
            .register_asset_processor(FighterBakeProcessor::new(
                FighterBakeTransformer,
                BakedFighterAnimationsSaver,
            ))
            .set_default_asset_processor::<FighterBakeProcessor>("fighterbake")
            .add_systems(
                GgrsSchedule,
                update_fighter_bone_matrices.in_set(GameplaySet::Animation),
            )
            .add_systems(Update, apply_baked_fighter_visual_pose)
            .add_observer(bind_fighter_visual_bones);
    }
}

fn bake_gltf(
    bytes: &[u8],
    animations: &HashMap<AnimKind, String>,
) -> io::Result<BakedFighterAnimations> {
    let (document, buffers, _) = gltf::import_slice(bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let nodes: Vec<_> = document.nodes().collect();
    if nodes.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "fighter model has no nodes",
        ));
    }
    let mut parents = vec![None; nodes.len()];
    for node in &nodes {
        for child in node.children() {
            parents[child.index()] = Some(node.index());
        }
    }
    let names = nodes
        .iter()
        .map(|node| {
            node.name().map(str::to_owned).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("node {} has no name", node.index()),
                )
            })
        })
        .collect::<io::Result<Vec<_>>>()?;

    let mut pending = HashMap::<&str, Vec<AnimKind>>::new();
    for (kind, name) in animations {
        pending.entry(name).or_default().push(*kind);
    }

    let mut clips = Vec::with_capacity(pending.len());
    let mut clip_indices = HashMap::with_capacity(animations.len());
    for animation in document.animations() {
        let name = animation.name().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("animation {} has no name", animation.index()),
            )
        })?;
        let Some(kinds) = pending.remove(name) else {
            continue;
        };
        let channels = animation
            .channels()
            .map(|channel| read_channel(channel, &buffers))
            .collect::<io::Result<Vec<_>>>()?;
        let duration = channels
            .iter()
            .filter_map(Channel::duration)
            .fold(0.0_f32, f32::max);
        let frame_count = (duration * BAKED_ANIMATION_FPS as f32).ceil() as usize + 1;
        let mut frames = Vec::with_capacity(frame_count);
        for frame in 0..frame_count {
            let time = frame as f32 / BAKED_ANIMATION_FPS as f32;
            let local = nodes
                .iter()
                .map(|node| local_transform(node.clone(), &channels, time))
                .collect::<io::Result<Vec<_>>>()?;
            frames.push(local);
        }
        let index = clips.len();
        clips.push(BakedAnimationClip { frames });
        for kind in kinds {
            clip_indices.insert(kind, index);
        }
    }
    if clips.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "fighter model has no animations",
        ));
    }
    if !pending.is_empty() {
        let mut missing = pending.keys().copied().collect::<Vec<_>>();
        missing.sort_unstable();
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "fighter model has no animations named {}",
                missing.join(", ")
            ),
        ));
    }
    Ok(BakedFighterAnimations {
        frame_rate: BAKED_ANIMATION_FPS,
        bone_names: names,
        parents,
        clips,
        clip_indices,
    })
}

fn resolve_model_transform(
    index: usize,
    parents: &[Option<usize>],
    local: &[FixedMat4],
    model: &mut [Option<FixedMat4>],
) -> io::Result<FixedMat4> {
    if let Some(value) = model[index] {
        return Ok(value);
    }
    let value = match parents[index] {
        Some(parent) => resolve_model_transform(parent, parents, local, model)?.mul(local[index]),
        None => local[index],
    };
    model[index] = Some(value);
    Ok(value)
}

#[derive(Clone)]
struct Channel {
    node: usize,
    property: gltf::animation::Property,
    interpolation: gltf::animation::Interpolation,
    times: Vec<f32>,
    values: ChannelValues,
}

impl Channel {
    fn duration(&self) -> Option<f32> {
        self.times.last().copied()
    }
}

#[derive(Clone)]
enum ChannelValues {
    Vec3(Vec<[f32; 3]>),
    Rotation(Vec<[f32; 4]>),
}

fn read_channel(
    channel: gltf::animation::Channel<'_>,
    buffers: &[gltf::buffer::Data],
) -> io::Result<Channel> {
    let reader = channel.reader(|buffer| Some(&buffers[buffer.index()].0));
    let times = reader
        .read_inputs()
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "animation channel has no input")
        })?
        .collect();
    let values = match reader.read_outputs().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "animation channel has no output",
        )
    })? {
        gltf::animation::util::ReadOutputs::Translations(values)
        | gltf::animation::util::ReadOutputs::Scales(values) => {
            ChannelValues::Vec3(values.collect())
        }
        gltf::animation::util::ReadOutputs::Rotations(values) => {
            ChannelValues::Rotation(values.into_f32().collect())
        }
        gltf::animation::util::ReadOutputs::MorphTargetWeights(_) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "morph target animation is not supported for bone baking",
            ));
        }
    };
    if channel.sampler().interpolation() == gltf::animation::Interpolation::CubicSpline {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "cubic spline animation is not supported for bone baking",
        ));
    }
    Ok(Channel {
        node: channel.target().node().index(),
        property: channel.target().property(),
        interpolation: channel.sampler().interpolation(),
        times,
        values,
    })
}

fn local_transform(node: gltf::Node<'_>, channels: &[Channel], time: f32) -> io::Result<FixedMat4> {
    let (mut translation, mut rotation, mut scale) = match node.transform() {
        gltf::scene::Transform::Decomposed {
            translation,
            rotation,
            scale,
        } => (
            Vec3::from(translation),
            Quat::from_array(rotation),
            Vec3::from(scale),
        ),
        gltf::scene::Transform::Matrix { matrix } => {
            return FixedMat4::from_mat4(Mat4::from_cols_array_2d(&matrix));
        }
    };
    for channel in channels
        .iter()
        .filter(|channel| channel.node == node.index())
    {
        match (&channel.values, channel.property) {
            (ChannelValues::Vec3(values), gltf::animation::Property::Translation) => {
                translation = Vec3::from(sample_vec3(channel, values, time))
            }
            (ChannelValues::Vec3(values), gltf::animation::Property::Scale) => {
                scale = Vec3::from(sample_vec3(channel, values, time))
            }
            (ChannelValues::Rotation(values), gltf::animation::Property::Rotation) => {
                rotation = Quat::from_array(sample_rotation(channel, values, time))
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "animation property does not match its values",
                ));
            }
        }
    }
    FixedMat4::from_mat4(Mat4::from_scale_rotation_translation(
        scale,
        rotation,
        translation,
    ))
}

fn sample_index(channel: &Channel, time: f32) -> (usize, usize, f32) {
    let next = channel.times.partition_point(|key| *key <= time);
    if next == 0 {
        return (0, 0, 0.0);
    }
    if next == channel.times.len() {
        let last = next - 1;
        return (last, last, 0.0);
    }
    let previous = next - 1;
    let factor = (time - channel.times[previous]) / (channel.times[next] - channel.times[previous]);
    (previous, next, factor)
}

fn sample_vec3(channel: &Channel, values: &[[f32; 3]], time: f32) -> [f32; 3] {
    let (from, to, factor) = sample_index(channel, time);
    if channel.interpolation == gltf::animation::Interpolation::Step {
        return values[from];
    }
    Vec3::from(values[from])
        .lerp(Vec3::from(values[to]), factor)
        .to_array()
}

fn sample_rotation(channel: &Channel, values: &[[f32; 4]], time: f32) -> [f32; 4] {
    let (from, to, factor) = sample_index(channel, time);
    if channel.interpolation == gltf::animation::Interpolation::Step {
        return values[from];
    }
    Quat::from_array(values[from])
        .slerp(Quat::from_array(values[to]), factor)
        .to_array()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_fighter_clips_bake_to_fixed_point_frames() {
        let manifest: FighterManifest =
            ron::from_str(include_str!("../../assets/fighters/test/test.fighter.ron")).unwrap();
        let baked = bake_gltf(
            include_bytes!("../../assets/fighters/test/test-fighter-normal.glb"),
            &manifest.animations,
        )
        .unwrap();

        assert_eq!(baked.frame_rate, BAKED_ANIMATION_FPS);
        assert!(baked.frame_count(AnimKind::Wait).unwrap() > 1);
        assert_eq!(baked.clip_indices.len(), manifest.animations.len());
        assert_eq!(
            baked.clip_indices[&AnimKind::JumpSquat],
            baked.clip_indices[&AnimKind::Landing]
        );
        assert_eq!(
            baked.clip(AnimKind::Wait).unwrap().frames[0].len(),
            baked.bone_names.len()
        );
        assert_eq!(baked.parents.len(), baked.bone_names.len());
    }

    #[test]
    fn pose_sampling_wraps_repeating_frames_and_clamps_finished_frames() {
        let first = FixedMat4::IDENTITY;
        let mut second = FixedMat4::IDENTITY;
        second.translate(FGVec3::lit("2", "0", "0"));
        let baked = BakedFighterAnimations {
            frame_rate: BAKED_ANIMATION_FPS,
            bone_names: vec!["root".into()],
            parents: vec![None],
            clips: vec![BakedAnimationClip {
                frames: vec![vec![first], vec![second]],
            }],
            clip_indices: HashMap::from([(AnimKind::Wait, 0)]),
        };

        let repeating = FighterAnimationFrame {
            kind: AnimKind::Wait,
            frame: 3,
            repeat: true,
        };
        let finished = FighterAnimationFrame {
            kind: AnimKind::Wait,
            frame: 3,
            repeat: false,
        };

        assert_eq!(baked.sample_pose(&repeating), Some(&[second][..]));
        assert_eq!(baked.sample_pose(&finished), Some(&[second][..]));
    }

    #[test]
    fn bone_matrix_component_keeps_names_aligned_with_the_current_pose() {
        let first = FixedMat4::IDENTITY;
        let mut second = FixedMat4::IDENTITY;
        second.translate(FGVec3::lit("0", "3", "0"));
        let mut matrices = FighterBoneMatrices::default();
        matrices.set_pose(
            &["root".into(), "hand".into()],
            &[None, Some(0)],
            &[first, second],
        );

        assert_eq!(matrices.get_current("hand"), Some(second));
        assert_eq!(
            matrices.iter().collect::<Vec<_>>(),
            vec![("root", first), ("hand", second)]
        );
    }

    #[test]
    fn local_pose_reconstructs_model_space_for_gameplay() {
        let mut root = FixedMat4::IDENTITY;
        root.translate(FGVec3::lit("10", "0", "0"));
        let mut hand = FixedMat4::IDENTITY;
        hand.translate(FGVec3::lit("0", "3", "0"));
        let mut matrices = FighterBoneMatrices::default();

        matrices.set_pose(
            &["root".into(), "hand".into()],
            &[None, Some(0)],
            &[root, hand],
        );

        assert_eq!(matrices.local(1), Some(hand));
        assert_eq!(
            matrices.get_current("hand").unwrap().get_translation(),
            FGVec3::lit("10", "3", "0")
        );
        assert_eq!(
            matrices
                .local(1)
                .unwrap()
                .to_mat4()
                .transform_point3(Vec3::ZERO),
            Vec3::new(0.0, 3.0, 0.0)
        );
    }

    #[test]
    fn fixed_matrix_transforms_points() {
        let transform = FixedMat4 {
            cols: [
                [FGi32::lit("2"), FGi32::ZERO, FGi32::ZERO, FGi32::ZERO],
                [FGi32::ZERO, FGi32::lit("3"), FGi32::ZERO, FGi32::ZERO],
                [FGi32::ZERO, FGi32::ZERO, FGi32::lit("4"), FGi32::ZERO],
                [
                    FGi32::lit("10"),
                    FGi32::lit("-5"),
                    FGi32::lit("1.5"),
                    FGi32::ONE,
                ],
            ],
        };

        assert_eq!(
            transform.transform_point(FGVec3::lit("1.25", "-2", "0.5")),
            FGVec3::lit("12.5", "-11", "3.5")
        );
    }

    #[test]
    fn fixed_matrix_rotates_points_around_y() {
        let transform = FixedMat4 {
            cols: [
                [FGi32::ONE, FGi32::ZERO, FGi32::ZERO, FGi32::ZERO],
                [FGi32::ZERO, FGi32::ONE, FGi32::ZERO, FGi32::ZERO],
                [FGi32::ZERO, FGi32::ZERO, FGi32::ONE, FGi32::ZERO],
                [
                    FGi32::lit("10"),
                    FGi32::lit("2"),
                    FGi32::lit("-3"),
                    FGi32::ONE,
                ],
            ],
        };

        assert_eq!(
            transform
                .rotate_y(5)
                .transform_point(FGVec3::lit("1", "0", "0")),
            FGVec3::lit("10", "2", "-4")
        );
    }

    #[test]
    fn fixed_matrix_builders_set_euler_rotation_and_translation() {
        let transform = FixedMat4::IDENTITY
            .with_rotation(FGVec3::lit("0", "0", "1.5707963267948966"))
            .with_translation(FGVec3::lit("10", "2", "-3"));

        assert_eq!(
            transform.transform_point(FGVec3::lit("1", "0", "0")),
            FGVec3::lit("10", "3", "-3")
        );
        assert_eq!(transform.get_translation(), FGVec3::lit("10", "2", "-3"));
    }

    #[test]
    fn fixed_matrix_rotation_builder_preserves_translation() {
        let transform = FixedMat4::IDENTITY
            .with_translation(FGVec3::lit("4", "5", "6"))
            .with_rotation(FGVec3::lit("0", "0", "1.5707963267948966"));

        assert_eq!(transform.get_translation(), FGVec3::lit("4", "5", "6"));
    }
}
