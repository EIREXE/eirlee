use bevy::prelude::*;
use clap::Parser;

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
    pub players: usize
}
