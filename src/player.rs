use bevy::prelude::*;
use bevy_ggrs::prelude::*;

// #[require(Rollback)] is the idiomatic way to ensure an entity is always included in
// the rollback system. Whenever a Player is spawned, Rollback (and its RollbackId) will
// be added automatically — no need to add it manually in the spawn bundle.
/// Ties an entity to one of the session's input streams. Lives at the crate
/// root rather than in `netcode` because "which player is this" is a game
/// concept; `netcode` only decides where the inputs come from.
#[derive(Default, Component)]
#[require(Rollback, Transform)]
pub struct Player {
    pub handle: usize,
}
