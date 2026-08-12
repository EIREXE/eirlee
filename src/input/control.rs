//! Which entity is driving which. Kept separate from the input pipeline
//! itself: this is a relationship between entities, not a source of input.

use bevy::prelude::*;

#[derive(Component, Debug)]
#[relationship(relationship_target = ControlledBy)]
pub struct Controls(pub Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = Controls)]
pub struct ControlledBy(Entity);
