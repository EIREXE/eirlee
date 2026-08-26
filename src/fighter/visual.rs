use bevy::gltf::Gltf;
use bevy::prelude::*;
use std::collections::HashMap;

use crate::{AppState, fighter::{
    animation::{AnimKind, AnimManifest},
    spawn_fighter,
}};

// One entry per character being loaded
pub struct PendingCharacter {
    pub id: String,
    pub gltf: Handle<Gltf>,
    pub manifest: Handle<AnimManifest>,
    pub resolved: bool,
}

#[derive(Resource, Default)]
pub struct CharacterLoadQueue(pub Vec<PendingCharacter>);

#[derive(Component)]
pub struct FighterAnimations {
    pub graph: Handle<AnimationGraph>,
    pub clips: HashMap<AnimKind, AnimationNodeIndex>,
}

pub fn start_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    let queue = vec![PendingCharacter {
        id: "jigglypuff".to_owned(),
        gltf: asset_server.load(format!("fighters/jiggs/jigglypuff-normal.glb")),
        manifest: asset_server.load(format!("fighters/jiggs/jigglypuff-anims.ron")),
        resolved: false,
    }];

    commands.insert_resource(CharacterLoadQueue(queue));
}

pub fn resolve_character_assets(
    mut commands: Commands,
    mut queue: ResMut<CharacterLoadQueue>,
    gltf_assets: Res<Assets<Gltf>>,
    manifest_assets: Res<Assets<AnimManifest>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    anim_clips: Res<Assets<AnimationClip>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for pending in queue.0.iter_mut().filter(|p| !p.resolved) {
        let (Some(gltf), Some(manifest)) = (
            gltf_assets.get(&pending.gltf),
            manifest_assets.get(&pending.manifest),
        ) else {
            continue; // still loading, try again next frame
        };

        let mut graph = AnimationGraph::new();
        let mut clips = HashMap::new();
        let mut missing = Vec::new();

        for (kind, clip_name) in manifest.0.iter() {
            match gltf.named_animations.get(clip_name.as_str()) {
                Some(clip_handle) => {
                    let animation = anim_clips.get(clip_handle);
                    info!("{clip_name}, {}", animation.unwrap().duration());
                    let node = graph.add_clip(clip_handle.clone(), 1.0, graph.root);
                    clips.insert(*kind, node);
                }
                None => missing.push(clip_name.clone()),
            }
        }

        if !missing.is_empty() {
            panic!(
                "character '{}': manifest references animations not found in glb: {:?}",
                pending.id, missing
            );
        }

        let graph_handle = graphs.add(graph);


        spawn_fighter(
            &mut commands,
            FighterAnimations {
                graph: graph_handle,
                clips,
            },
            WorldAssetRoot(gltf.scenes[0].clone()),
        );

        pending.resolved = true;
    }

    // All done? Move on.
    if queue.0.iter().all(|p| p.resolved) {
        next_state.set(AppState::InGame);
        commands.remove_resource::<CharacterLoadQueue>();
    }
}
