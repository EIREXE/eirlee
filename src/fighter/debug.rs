//! Fighter debug views: gizmos and the egui inspector window.

use bevy::color::palettes::css::ORANGE;
use bevy::{color::palettes::css::YELLOW, prelude::*};
use bevy_egui::{EguiContexts, egui};

use crate::fighter::animation::FighterAnimationFrame;
use crate::fighter::baked_animation::BakedFighterAnimations;
use crate::fighter::ecb::FighterPreviousECB;
use crate::fighter::state::StateNameDebug;
use crate::fighter::visual::FighterAnimations;
use crate::fighter::{
    FighterECB, FighterFacingDirection, FighterHitboxes, FighterPreviousTranslation, FighterTranslation, FighterVelocity,
};
use crate::input::FighterInput;
use crate::math::vec3::FGVec3;
use crate::scripting::FighterAttackScript;

pub fn debug_draw_ecb(
    ecbs: Query<(
        &FighterECB,
        &FighterTranslation,
        &FighterPreviousECB,
        &FighterPreviousTranslation,
    )>,
    mut gizmos: Gizmos,
) {
    for (ecb, trf, ecb_prev, trf_prev) in ecbs {
        let transformed =
            ecb.to_3d_lineloop(Transform::from_xyz(trf.x.to_num(), trf.y.to_num(), 0.0));
        gizmos.lineloop(transformed, YELLOW);
        let prev_transformed = ecb_prev.to_3d_lineloop(Transform::from_xyz(
            trf_prev.x.to_num(),
            trf_prev.y.to_num(),
            -0.1,
        ));
        gizmos.lineloop(prev_transformed, ORANGE);
    }
}
pub fn fighter_debug(
    mut contexts: EguiContexts,
    query: Query<(&FighterVelocity, &FighterInput, &StateNameDebug)>,
) -> Result {
    for (i, (vel, input, name)) in query.iter().enumerate() {
        egui::Window::new(format!("Player {}", i)).show(contexts.ctx_mut()?, |ui| {
            ui.label(format!("State: {}", name.0));
            ui.label(format!("{:?}", vel));
            ui.label(format!("{:?}", input.get_last_frame()));
        });
    }
    Ok(())
}

pub fn animation_debug(
    query: Query<(&GlobalTransform, &FighterAnimationFrame, &FighterAnimations)>,
    baked_anims: Res<Assets<BakedFighterAnimations>>,
    mut gizmos: Gizmos,
) {
    for (global_transform, frame, anims) in query {
        let baked: &BakedFighterAnimations = baked_anims
            .get(&anims.baked)
            .expect("Baked anims should be in");
        for bone in &baked.bone_names {
            if bone != "Bone_30" {
                continue;
            }
            let out = anims.sample_bone(&baked_anims, frame, bone);
            if let Some(out) = out {
                let pos = global_transform.transform_point(Vec3::new(
                    out.cols[3][0].to_num(),
                    out.cols[3][1].to_num(),
                    out.cols[3][2].to_num(),
                ));
                //gizmos.text(pos, bone, 1.0, Vec2::new(0.0, 0.0), bevy::color::palettes::css::BLACK);
                gizmos.cross(pos, 0.5, bevy::color::palettes::css::RED);
                gizmos.axes(*global_transform, 1.0);
            }
        }
    }
}

pub fn attack_debug(
    query: Query<(&FighterHitboxes, &FighterAnimationFrame, &FighterAnimations, &FighterTranslation, &FighterFacingDirection)>,
    baked_anims: Res<Assets<BakedFighterAnimations>>,
    attack_scripts: Res<Assets<FighterAttackScript>>,
    mut gizmos: Gizmos,
) {
    for (hitboxes, frame, animations, translation, facing_direction) in query {
        if let Some(script) = &hitboxes.attack_script {
            let script = attack_scripts.get(script).expect("Script ref should be valid");
            let player_trf = translation.get_3d_transform(facing_direction);

            for i in hitboxes.active_hitboxes.iter() {
                let offset = script.hitboxes[*i].offset;
                let radius = script.hitboxes[*i].radius;
                let bone = &script.hitboxes[*i].bone;

                let trf = animations.sample_bone(&baked_anims, frame, bone);
                if let Some(trf) = trf {
                    let trf = player_trf.mul(trf);
                    let start = player_trf.transform_point(FGVec3::lit("0.0", "1.0", "0.0"));
                    let end = player_trf.transform_point(FGVec3::lit("0.0", "1.0", "1.0"));
                    let pos = trf.transform_point(offset);
                    let pos_draw = Vec3::new(pos.x.to_num(), pos.y.to_num(), pos.z.to_num());
                    gizmos.sphere(pos_draw, radius.to_num(), bevy::color::palettes::css::RED);
                    
                    gizmos.arrow(start.to_vec3(), end.to_vec3(),bevy::color::palettes::css::RED);
                }
            }
        }
    }
}

pub fn update_config(
    mut config_store: ResMut<GizmoConfigStore>,
    keyboard: Res<ButtonInput<KeyCode>>,
    _real_time: Res<Time<Real>>,
    _virtual_time: ResMut<Time<Virtual>>,
) {
    if keyboard.just_pressed(KeyCode::KeyT) {
        for (_, config, _) in config_store.iter_mut() {
            config.depth_bias = if config.depth_bias == 0. { -1. } else { 0. };
        }
    }
}
