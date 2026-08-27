use std::time::Duration;

use bevy::prelude::*;

use crate::{
    fighter::{FighterECB, FighterFacingDirection, FighterTranslation, FighterVisual, visual::FighterAnimations}, player::Player,
};

#[derive(Component)]
pub struct FighterAnimationPlayerLink(Entity);

impl FighterAnimationPlayerLink {
    pub fn player(&self) -> Entity {
        self.0
    }
}

pub fn apply_fighter_translation_to_visuals(
    query: Query<(&FighterTranslation, &FighterECB, &mut Transform)>,
) {
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

        commands.entity(child).insert(AnimationTransitions::new());
        info!("LINK");
        commands
            .entity(parent)
            .insert(FighterAnimationPlayerLink(child));
    }
}

pub fn apply_animation(children: Query<(&mut Transform, &FighterFacingDirection), With<FighterVisual>>) {
    for (mut trf, facing_direction) in children {
        // Melee model scale is in decimeters
        trf.rotation = Quat::IDENTITY;
        trf.rotate_local_y(std::f32::consts::PI * 0.5 * facing_direction.to_sign().to_num::<f32>());
    }
}

pub fn animation_init(
    fighters: Query<
        (&FighterAnimations, &FighterAnimationPlayerLink),
        Added<FighterAnimationPlayerLink>,
    >,
    mut commands: Commands,
    mut anim_player_query: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    for (anim, link) in fighters {
        commands
            .entity(link.0)
            .insert(AnimationGraphHandle(anim.graph.clone()));
        info!("BEGIN! {:?}", fighters);
        if let Ok((mut player, mut transitions)) = anim_player_query.get_mut(link.0) {
            info!("DOS");
            transitions
                .play(&mut player, anim.clips[&AnimKind::Wait], Duration::ZERO)
                .repeat();
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize, Reflect,
)]
pub enum AnimKind {
    Wait,
    Walk,
    Run,
    Dash,
    JumpSquat,
    JumpForward,
    JumpBack,
    DoubleJump,
    Landing,
    AirDodge,
}
