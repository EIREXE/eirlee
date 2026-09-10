//! Deterministic input recording and playback.

use std::{collections::BTreeMap, fs, path::{Path, PathBuf}};

use bevy::prelude::*;
use bevy_ggrs::{prelude::*, PlayerInputs, RollbackFrameCount, SaveWorld};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::{args::Args, input::FighterInputFrame, match_loading::{MatchPlayer, PendingMatch}, netcode::GGRSCfg, schedule::GameplaySet, AppState};

const MAGIC: &[u8; 8] = b"SPRPLY01";
pub const REPLAY_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum ReplayFormat {
    #[default]
    Binary,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Replay {
    pub replay_version: u32,
    pub game_version: String,
    pub match_config: ReplayMatch,
    pub frames: Vec<ReplayFrame>,
    pub checksums: Vec<ReplayChecksum>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReplayMatch {
    pub stage: crate::stage::manifest::StageId,
    pub players: Vec<MatchPlayer>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReplayFrame {
    pub frame: i32,
    pub inputs: Vec<FighterInputFrame>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayChecksum {
    pub frame: i32,
    pub checksum: u128,
}

#[derive(Resource)]
pub struct LoadedReplay(pub Replay);

#[derive(Resource)]
pub struct ReplayComplete;

#[derive(Resource, Default)]
pub struct StopReplayRecording;

enum ReplayMode {
    Record { path: PathBuf, format: ReplayFormat },
    Playback { replay: Replay, mismatch: bool },
}

#[derive(Resource)]
struct ReplayRuntime {
    mode: ReplayMode,
    frames: BTreeMap<i32, Vec<FighterInputFrame>>,
    checksums: BTreeMap<i32, u128>,
    finished: bool,
}

pub struct ReplayPlugin;

impl Plugin for ReplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InMatch), start_replay)
            .add_systems(
                GgrsSchedule,
                replay_inputs
                    .in_set(GameplaySet::Input)
                    .before(crate::input::buffer::postprocess_input),
            )
            .add_systems(SaveWorld, record_checksum.after(SaveWorldSystems::Checksum).before(SaveWorldSystems::Snapshot))
            .add_systems(OnExit(AppState::InMatch), finish_recording)
            .add_systems(Update, stop_recording.run_if(in_state(AppState::InMatch)))
            .add_systems(Update, finish_playback.run_if(in_state(AppState::InMatch)));
    }
}

pub fn load(path: &Path) -> Result<Replay, String> {
    let bytes = fs::read(path).map_err(|error| format!("cannot read replay {}: {error}", path.display()))?;
    let replay = if bytes.starts_with(MAGIC) {
        if bytes.len() < MAGIC.len() + 4 {
            return Err("replay binary header is truncated".into());
        }
        let version = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
        if version != REPLAY_VERSION {
            return Err(format!("unsupported replay version {version}"));
        }
        bincode::deserialize(&bytes[12..]).map_err(|error| format!("invalid binary replay: {error}"))?
    } else {
        serde_json::from_slice(&bytes).map_err(|error| format!("invalid JSON replay: {error}"))?
    };
    validate(&replay)?;
    Ok(replay)
}

pub fn save(path: &Path, format: ReplayFormat, replay: &Replay) -> Result<(), String> {
    validate(replay)?;
    let bytes = match format {
        ReplayFormat::Binary => {
            let mut bytes = MAGIC.to_vec();
            bytes.extend(REPLAY_VERSION.to_le_bytes());
            bytes.extend(bincode::serialize(replay).map_err(|error| format!("cannot encode replay: {error}"))?);
            bytes
        }
        ReplayFormat::Json => serde_json::to_vec_pretty(replay).map_err(|error| format!("cannot encode replay JSON: {error}"))?,
    };
    let temporary = path.with_extension("replay.tmp");
    fs::write(&temporary, bytes).map_err(|error| format!("cannot write replay: {error}"))?;
    fs::rename(&temporary, path).map_err(|error| format!("cannot finalize replay: {error}"))
}

fn validate(replay: &Replay) -> Result<(), String> {
    if replay.replay_version != REPLAY_VERSION {
        return Err(format!("unsupported replay version {}", replay.replay_version));
    }
    if replay.players_are_invalid() {
        return Err("replay player handles must cover 0..player_count".into());
    }
    if replay.frames.is_empty() {
        return Err("replay must contain at least one frame".into());
    }
    if replay.frames.windows(2).any(|frames| frames[0].frame >= frames[1].frame) {
        return Err("replay frames must be strictly ordered".into());
    }
    if replay.frames.iter().any(|frame| frame.inputs.len() != replay.match_config.players.len()) {
        return Err("replay frame input count does not match player count".into());
    }
    Ok(())
}

impl Replay {
    fn players_are_invalid(&self) -> bool {
        let mut handles: Vec<_> = self.match_config.players.iter().map(|player| player.handle).collect();
        handles.sort_unstable();
        handles != (0..handles.len()).collect::<Vec<_>>()
    }
}

fn start_replay(mut commands: Commands, args: Res<Args>, loaded: Option<Res<LoadedReplay>>) {
    let mode = if let Some(replay) = loaded {
        ReplayMode::Playback { replay: replay.0.clone(), mismatch: false }
    } else if let Some(path) = args.record_replay.clone() {
        ReplayMode::Record { path, format: args.replay_format }
    } else {
        return;
    };
    commands.insert_resource(ReplayRuntime { mode, frames: BTreeMap::new(), checksums: BTreeMap::new(), finished: false });
}

fn replay_inputs(mut inputs: ResMut<PlayerInputs<GGRSCfg>>, frame: Res<RollbackFrameCount>, runtime: Option<ResMut<ReplayRuntime>>) {
    let Some(mut runtime) = runtime else { return };
    match &mut runtime.mode {
        ReplayMode::Record { .. } => {
            runtime.frames.insert(frame.0, inputs.iter().map(|(input, _)| *input).collect());
        }
        ReplayMode::Playback { replay, mismatch } if !*mismatch => {
            let Some(recorded) = replay.frames.iter().find(|recorded| recorded.frame == frame.0) else {
                error!("Replay has no input for frame {}", frame.0);
                *mismatch = true;
                return;
            };
            for ((input, _), recorded) in inputs.iter_mut().zip(&recorded.inputs) {
                *input = *recorded;
            }
        }
        ReplayMode::Playback { .. } => {}
    }
}

fn record_checksum(frame: Res<RollbackFrameCount>, checksum: Res<Checksum>, runtime: Option<ResMut<ReplayRuntime>>) {
    let Some(mut runtime) = runtime else { return };
    if matches!(&runtime.mode, ReplayMode::Record { .. }) {
        runtime.checksums.insert(frame.0, checksum.0);
    } else if let ReplayMode::Playback { replay, mismatch } = &mut runtime.mode {
        if let Some(expected) = replay.checksums.iter().find(|value| value.frame == frame.0).map(|value| value.checksum) {
            if expected != checksum.0 {
                error!("Replay desync at frame {}: expected {:X}, got {:X}", frame.0, expected, checksum.0);
                *mismatch = true;
            }
        }
    }
}

fn stop_recording(
    mut commands: Commands,
    stop_request: Option<Res<StopReplayRecording>>,
    runtime: Option<ResMut<ReplayRuntime>>,
    request: Option<Res<PendingMatch>>,
) {
    if stop_request.is_none() {
        return;
    }
    commands.remove_resource::<StopReplayRecording>();
    finish_recording_inner(&mut commands, runtime, request);
}

fn finish_recording(mut commands: Commands, runtime: Option<ResMut<ReplayRuntime>>, request: Option<Res<PendingMatch>>) {
    finish_recording_inner(&mut commands, runtime, request);
}

fn finish_recording_inner(
    commands: &mut Commands,
    runtime: Option<ResMut<ReplayRuntime>>,
    request: Option<Res<PendingMatch>>,
) {
    let Some(mut runtime) = runtime else { return };
    let ReplayMode::Record { path, format } = &runtime.mode else { return };
    if runtime.finished || runtime.frames.is_empty() { return; }
    let Some(request) = request else { return };
    let path = path.clone();
    let format = *format;
    let replay = Replay {
        replay_version: REPLAY_VERSION,
        game_version: env!("CARGO_PKG_VERSION").into(),
        match_config: ReplayMatch { stage: request.stage, players: request.players.clone() },
        frames: runtime.frames.iter().map(|(&frame, inputs)| ReplayFrame { frame, inputs: inputs.clone() }).collect(),
        checksums: runtime.checksums.iter().map(|(&frame, &checksum)| ReplayChecksum { frame, checksum }).collect(),
    };
    if let Err(error) = save(&path, format, &replay) { error!("Failed to save replay: {error}"); }
    runtime.finished = true;
    commands.remove_resource::<ReplayRuntime>();
}

fn finish_playback(mut commands: Commands, mut next_state: ResMut<NextState<AppState>>, frame: Res<RollbackFrameCount>, runtime: Option<Res<ReplayRuntime>>) {
    let Some(runtime) = runtime else { return };
    let ReplayMode::Playback { replay, mismatch } = &runtime.mode else { return };
    if *mismatch || replay.frames.last().is_some_and(|last| frame.0 >= last.frame) {
        commands.remove_resource::<ReplayRuntime>();
        commands.remove_resource::<LoadedReplay>();
        commands.insert_resource(ReplayComplete);
        next_state.set(AppState::MainMenu);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{fighter::FighterId, math::{int::FGi32, vec::FGVec2}, match_loading::MatchPlayer, stage::manifest::StageId};

    fn sample() -> Replay {
        Replay {
            replay_version: REPLAY_VERSION,
            game_version: "test".into(),
            match_config: ReplayMatch {
                stage: StageId::TestStage,
                players: vec![MatchPlayer { handle: 0, fighter: FighterId::TestFighter }],
            },
            frames: vec![ReplayFrame {
                frame: 1,
                inputs: vec![FighterInputFrame { movement: FGVec2::new(FGi32::from_num(1), FGi32::ZERO), ..default() }],
            }],
            checksums: vec![ReplayChecksum { frame: 1, checksum: 42 }],
        }
    }

    #[test]
    fn binary_and_json_round_trip() {
        let replay = sample();
        let directory = std::env::temp_dir();
        let binary = directory.join(format!("game-test-replay-{}.bin", std::process::id()));
        let json = directory.join(format!("game-test-replay-{}.json", std::process::id()));

        save(&binary, ReplayFormat::Binary, &replay).unwrap();
        save(&json, ReplayFormat::Json, &replay).unwrap();
        assert_eq!(load(&binary).unwrap(), replay);
        assert_eq!(load(&json).unwrap(), replay);
        let _ = fs::remove_file(binary);
        let _ = fs::remove_file(json);
    }

    #[test]
    fn malformed_frame_input_count_is_rejected() {
        let mut replay = sample();
        replay.frames[0].inputs.clear();
        assert!(validate(&replay).is_err());
    }
}
