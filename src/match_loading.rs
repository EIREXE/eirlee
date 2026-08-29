//! Match selection, asset loading, preparation, and cleanup.

use std::collections::HashMap;

use bevy::{asset::UntypedHandle, gltf::Gltf, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_ggrs::prelude::Session;

use crate::{
    AppState, args::Args, fighter::{
        FighterId, animation::AnimKind, attack::AttackKind, baked_animation::BakedFighterAnimations, manifest::{FighterManifest, FighterManifestRegistry}, spawn_fighter, visual::FighterAnimations,
    }, math::{int::FGi32, vec::FGVec2}, netcode::{GGRSCfg, session::create_session}, scripting::FighterAttackScript, stage::{
        self,
        manifest::{StageId, StageManifest, StageManifestRegistry},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchPlayer {
    pub handle: usize,
    pub fighter: FighterId,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct PendingMatch {
    pub stage: StageId,
    pub players: Vec<MatchPlayer>,
}

pub struct SelectedFighterAssets {
    pub fighter: FighterId,
    pub model: Handle<Gltf>,
    pub baked_animations: Handle<BakedFighterAnimations>,
    pub attack_scripts: HashMap<AttackKind, Handle<FighterAttackScript>>
}

#[derive(Resource)]
pub struct MatchAssets {
    pub stage: Handle<WorldAsset>,
    pub fighters: Vec<SelectedFighterAssets>,
}

impl MatchAssets {
    fn requested(world: &World) -> Self {
        let request = world.resource::<PendingMatch>();
        let asset_server = world.resource::<AssetServer>();
        let registry = world.resource::<FighterManifestRegistry>();
        let stage_registry = world.resource::<StageManifestRegistry>();
        let manifests = world.resource::<Assets<FighterManifest>>();
        let stage_manifests = world.resource::<Assets<StageManifest>>();

        let stage_handle = stage_registry
            .get(request.stage)
            .expect("Startup validation guarantees every stage has a manifest");
        let stage_manifest = stage_manifests
            .get(&stage_handle)
            .expect("startup-loaded stage manifest should remain available");

        Self {
            stage: asset_server.load(stage_manifest.model_path.clone()),
            fighters: request
                .players
                .iter()
                .map(|player| {
                    let handle = registry
                        .get(player.fighter)
                        .expect("startup validation guarantees every fighter has a manifest");
                    let manifest = manifests
                        .get(handle)
                        .expect("startup-loaded fighter manifest should remain available");

                    SelectedFighterAssets {
                        fighter: player.fighter,
                        model: asset_server.load(manifest.model_path.clone()),
                        baked_animations: asset_server.load(manifest.baked_animation_path.clone()),
                        attack_scripts: manifest.attack_scripts.clone().into_iter().map(|(kind, path)| (kind, asset_server.load(path))).collect()
                    }
                })
                .collect(),
        }
    }
}

impl AssetCollection for MatchAssets {
    fn create(world: &mut World) -> Self {
        Self::requested(world)
    }

    fn load(world: &mut World) -> Vec<UntypedHandle> {
        let assets = Self::requested(world);
        let mut handles = Vec::with_capacity(1 + assets.fighters.len());
        handles.push(assets.stage.clone().untyped());
        for character in assets.fighters {
            handles.push(character.model.untyped());
            handles.push(character.baked_animations.untyped());
        }
        handles
    }
}

#[derive(Component)]
pub struct MatchRoot;

pub fn initiate_match(
    commands: &mut Commands,
    next_state: &mut NextState<AppState>,
    request: PendingMatch,
) {
    commands.insert_resource(request);
    next_state.set(AppState::LoadingMatch);
}

pub fn initiate_default_match(
    mut commands: Commands,
    args: Res<Args>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let players = (0..args.players)
        .map(|handle| MatchPlayer {
            handle,
            fighter: FighterId::TestFighter,
        })
        .collect();

    initiate_match(
        &mut commands,
        &mut next_state,
        PendingMatch {
            stage: StageId::TestStage,
            players,
        },
    );
}

pub fn prepare_match(
    mut commands: Commands,
    request: Res<PendingMatch>,
    assets: Res<MatchAssets>,
    gltfs: Res<Assets<Gltf>>,
    baked_animations: Res<Assets<BakedFighterAnimations>>,
    registry: Res<FighterManifestRegistry>,
    stage_registry: Res<StageManifestRegistry>,
    manifests: Res<Assets<FighterManifest>>,
    stage_manifests: Res<Assets<StageManifest>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    args: Res<Args>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if !valid_player_handles(&request.players) {
        fail_match(
            &mut next_state,
            "player handles must be unique and cover 0..player_count",
        );
        return;
    }

    if assets.fighters.len() != request.players.len() {
        fail_match(
            &mut next_state,
            "loaded character assets do not match the requested player slots",
        );
        return;
    }

    let mut prepared = Vec::with_capacity(assets.fighters.len());
    let mut shared_animations = HashMap::<FighterId, FighterAnimations>::new();

    for character in &assets.fighters {
        let manifest_handle = registry
            .get(character.fighter)
            .expect("startup validation guarantees every fighter has a manifest");
        let Some(manifest) = manifests.get(manifest_handle) else {
            fail_match(&mut next_state, "a fighter manifest is unavailable");
            return;
        };
        let Some(gltf) = gltfs.get(&character.model) else {
            fail_match(&mut next_state, "a loaded character GLTF is unavailable");
            return;
        };
        let Some(baked) = baked_animations.get(&character.baked_animations) else {
            fail_match(
                &mut next_state,
                "a fighter baked animation asset is unavailable",
            );
            return;
        };
        let Some(scene) = gltf.scenes.first().cloned() else {
            fail_match(&mut next_state, "a character GLTF contains no scene");
            return;
        };

        if let Some(animations) = shared_animations.get(&character.fighter) {
            prepared.push((
                scene,
                animations.clone(),
                manifest_handle,
            ));
            continue;
        }

        let mut graph = AnimationGraph::new();
        let mut clips = HashMap::<AnimKind, AnimationNodeIndex>::new();

        for (kind, clip_name) in &manifest.animations {
            let Some(clip) = gltf.named_animations.get(clip_name.as_str()) else {
                fail_match(
                    &mut next_state,
                    &format!(
                        "{:?} animation manifest references missing clip '{clip_name}'",
                        character.fighter
                    ),
                );
                return;
            };
            clips.insert(*kind, graph.add_clip(clip.clone(), 1.0, graph.root));
            if baked.frame_count(*kind).is_none() {
                fail_match(
                    &mut next_state,
                    &format!(
                        "{:?} baked data has no {kind:?} animation",
                        character.fighter
                    ),
                );
                return;
            }
        }

        if !clips.contains_key(&AnimKind::Wait) {
            fail_match(&mut next_state, "a character has no Wait animation");
            return;
        }

        let animations = FighterAnimations {
            graph: graphs.add(graph),
            clips,
            baked: character.baked_animations.clone(),
        };
        shared_animations.insert(character.fighter, animations.clone());
        prepared.push((
            scene,
            animations,
            manifest_handle,
        ));
    }

    let session = match create_session(&args, request.players.len()) {
        Ok(session) => session,
        Err(error) => {
            fail_match(
                &mut next_state,
                &format!("failed to create GGRS session: {error}"),
            );
            return;
        }
    };

    let root = commands.spawn((Name::new("Match"), MatchRoot)).id();

    let stage_handle = stage_registry
        .get(request.stage)
        .expect("Initialization check should ensure all stages exist");
    let stage_manifest = stage_manifests
        .get(&stage_handle)
        .expect("Initialization check should ensure stage manifests are kept alive");
    stage::spawn_stage(
        &mut commands,
        root,
        assets.stage.clone(),
        stage_manifest.to_collision(),
        stage_manifest.camera.clone(),
    );

    for (
        index,
        (player, (scene, animations, manifest_handle)),
    ) in request.players.iter().zip(prepared).enumerate()
    {
        let spawn_x = (index as i32 * 2 + 1 - request.players.len() as i32) * 5;
        spawn_fighter(
            &mut commands,
            player.handle,
            FGVec2::new(FGi32::from_num(spawn_x), FGi32::lit("12.5")),
            manifest_handle.clone(),
            animations,
            WorldAssetRoot(scene),
        );
    }

    commands.insert_resource::<Session<GGRSCfg>>(session);
    next_state.set(AppState::InMatch);
}

fn fail_match(next_state: &mut NextState<AppState>, message: &str) {
    error!("Match preparation failed: {message}");
    next_state.set(AppState::MatchLoadFailed);
}

fn valid_player_handles(players: &[MatchPlayer]) -> bool {
    let mut handles = players
        .iter()
        .map(|player| player.handle)
        .collect::<Vec<_>>();
    handles.sort_unstable();
    handles == (0..players.len()).collect::<Vec<_>>()
}

pub fn cleanup_match(
    mut commands: Commands,
    roots: Query<Entity, With<MatchRoot>>,
    fighters: Query<Entity, With<crate::player::Player>>,
) {
    for root in &roots {
        commands.entity(root).despawn();
    }
    for fighter in &fighters {
        commands.entity(fighter).despawn();
    }
    commands.remove_resource::<Session<GGRSCfg>>();
    commands.remove_resource::<stage::line::StageCollision>();
    commands.remove_resource::<MatchAssets>();
    commands.remove_resource::<PendingMatch>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_request_keeps_duplicate_fighter_slots() {
        let request = PendingMatch {
            stage: StageId::TestStage,
            players: vec![
                MatchPlayer {
                    handle: 0,
                    fighter: FighterId::TestFighter,
                },
                MatchPlayer {
                    handle: 1,
                    fighter: FighterId::TestFighter,
                },
            ],
        };

        assert_eq!(request.players.len(), 2);
        assert_eq!(request.players[0].handle, 0);
        assert_eq!(request.players[1].handle, 1);
        assert!(valid_player_handles(&request.players));
    }

    #[test]
    fn player_handles_must_match_the_session_range() {
        let players = [
            MatchPlayer {
                handle: 0,
                fighter: FighterId::TestFighter,
            },
            MatchPlayer {
                handle: 0,
                fighter: FighterId::TestFighter,
            },
        ];

        assert!(!valid_player_handles(&players));
    }
}
