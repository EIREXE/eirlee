//! Deterministic, fighter-local skeletal animation samples for gameplay.
//!
//! Rendering continues to use Bevy's floating-point animation player. Gameplay
//! reads this asset instead, so rollback never depends on render-time sampling.

use std::{collections::HashMap, io, path::Path};

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
        int::{FGWide, FGi32},
        vec3::FGVec3,
    },
    schedule::GameplaySet,
};

pub const BAKED_ANIMATION_FPS: u32 = 60;

/// Parent transforms are composed after
/// quantization, which makes every runtime sample platform-independent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    /// One model-space matrix per stored node, for each 60 Hz simulation frame.
    pub frames: Vec<Vec<FixedMat4>>,
}

#[derive(Asset, TypePath, Clone, Debug, Serialize, Deserialize)]
pub struct BakedFighterAnimations {
    pub frame_rate: u32,
    /// Indexed exactly like every frame in each clip.
    pub bone_names: Vec<String>,
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
    matrices: Vec<FixedMat4>,
}

impl FighterBoneMatrices {
    pub fn get(&self, bone: &str) -> Option<FixedMat4> {
        let index = self.bone_names.iter().position(|name| name == bone)?;
        self.matrices.get(index).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, FixedMat4)> {
        self.bone_names
            .iter()
            .map(String::as_str)
            .zip(self.matrices.iter().copied())
    }

    fn set_pose(&mut self, bone_names: &[String], matrices: &[FixedMat4]) {
        assert_eq!(
            bone_names.len(),
            matrices.len(),
            "baked animation pose does not match its bone names"
        );
        if self.bone_names.is_empty() {
            self.bone_names.extend_from_slice(bone_names);
        } else {
            assert_eq!(self.bone_names, bone_names, "baked bone names changed");
        }
        self.matrices.clear();
        self.matrices.extend_from_slice(matrices);
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
        matrices.set_pose(&baked.bone_names, pose);
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
            );
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
            let mut model = vec![None; nodes.len()];
            for index in 0..nodes.len() {
                resolve_model_transform(index, &parents, &local, &mut model)?;
            }
            frames.push(model.into_iter().map(Option::unwrap).collect());
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
    }

    #[test]
    fn pose_sampling_wraps_repeating_frames_and_clamps_finished_frames() {
        let first = FixedMat4::IDENTITY;
        let mut second = FixedMat4::IDENTITY;
        second.translate(FGVec3::lit("2", "0", "0"));
        let baked = BakedFighterAnimations {
            frame_rate: BAKED_ANIMATION_FPS,
            bone_names: vec!["root".into()],
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
        matrices.set_pose(&["root".into(), "hand".into()], &[first, second]);

        assert_eq!(matrices.get("hand"), Some(second));
        assert_eq!(
            matrices.iter().collect::<Vec<_>>(),
            vec![("root", first), ("hand", second)]
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
}
