use bevy::{mesh::skinning::SkinnedMesh, prelude::*};

use crate::{fighter::{FighterECB, FighterTranslation, FighterVisual}, player::Player};

#[derive(Component)]
pub struct FighterAnimations {
    pub wait: Handle<AnimationClip>
}

#[derive(Component)]
pub struct FighterAnimationPlayerLink(Entity);

pub fn apply_fighter_translation_to_visuals(query: Query<(&FighterTranslation, &FighterECB, &mut Transform)>) {
    for (translation, ecb, mut transform) in query {
        let translation = translation.0 + ecb.get_bottom_point();
        transform.translation = Vec3::new(translation.x.to_num(), translation.y.to_num(), 0.0);
    }
}

pub fn setup_fighter_animation_player(
    mut commands: Commands,
    animated: Query<Entity, Added<AnimationPlayer>>,
    parents: Query<&ChildOf>,
    fighters: Query<Entity, With<Player>>,
) {
    for child in &animated {
        let Some(parent) = parents
            .iter_ancestors(child)
            .filter_map(|parent| fighters.get(parent).ok())
            .next()
        else {
            continue;
        };
        commands.entity(parent).insert(
            FighterAnimationPlayerLink(child)
        );
    }
}

pub fn apply_animation(
    children: Query<&mut Transform, With<FighterVisual>>,
) {
    for mut trf in children {
        // Melee model scale is in decimeters
        trf.scale = Vec3::splat(0.1);
    }

}