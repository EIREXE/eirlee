use bevy::prelude::*;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone, Resource)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(long)]
    pub synctest: bool,
    #[clap(long, default_value_t = 2)]
    pub input_delay: usize,
    #[clap(long, default_value_t = 2)]
    pub check_distance: usize,
    #[clap(long, default_value_t = 1)]
    pub players: usize,
    /// Record a replay to this path.
    #[clap(long, conflicts_with = "play_replay")]
    pub record_replay: Option<PathBuf>,
    /// Play a replay from this path.
    #[clap(long, conflicts_with = "record_replay")]
    pub play_replay: Option<PathBuf>,
    /// Replay output format. JSON is useful for inspection; binary is compact.
    #[clap(long, value_enum, default_value_t = crate::replay::ReplayFormat::Binary)]
    pub replay_format: crate::replay::ReplayFormat,
}
