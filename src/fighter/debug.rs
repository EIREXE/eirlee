//! Fighter debug views: gizmos and the egui inspector window.

use bevy::color::palettes::css::ORANGE;
use bevy::gizmos::primitives::dim3::Capsule3dBuilder;
use bevy::{color::palettes::css::YELLOW, prelude::*};
use bevy_egui::{EguiContexts, egui};

use crate::debug_tools::{DebugSettings, FighterDebugRowKind};
use crate::fighter::animation::FighterAnimationFrame;
use crate::fighter::baked_animation::{BakedFighterAnimations, FighterBoneMatrices};
use crate::fighter::ecb::FighterPreviousECB;
use crate::fighter::state::StateNameDebug;
use crate::fighter::visual::FighterAnimations;
use crate::fighter::{
    FighterECB, FighterFacingDirection, FighterHitboxes, FighterPreviousTranslation,
    FighterTranslation, FighterVelocity,
};
use crate::input::FighterInput;
use crate::math::vec3::FGVec3;
use crate::player::Player;
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
    query: Query<(
        &Player,
        &FighterVelocity,
        &FighterInput,
        &FighterAnimationFrame,
        &StateNameDebug,
        &FighterAnimations,
        &FighterTranslation,
        &FighterFacingDirection,
    )>,
    anims: Res<Assets<BakedFighterAnimations>>,
    settings: Res<DebugSettings>,
) -> Result {
    let mut fighters = query.iter().collect::<Vec<_>>();
    fighters.sort_unstable_by_key(|(player, ..)| player.handle);

    for (player, velocity, input, animation_frame, state_name, animations, translation, facing) in
        fighters
    {
        let Some(rows) = settings.player_rows(player.handle) else {
            continue;
        };
        if rows.iter().all(|row| *row == FighterDebugRowKind::None) {
            continue;
        }

        egui::Window::new(format!("Player {}", player.handle + 1)).show(
            contexts.ctx_mut()?,
            |ui| {
                for row in rows {
                    match row {
                        FighterDebugRowKind::None => {}
                        FighterDebugRowKind::CurrentState => {
                            let frame_count = anims
                                .get(&animations.baked)
                                .and_then(|animation| animation.frame_count(animation_frame.kind));
                            if let Some(frame_count) = frame_count {
                                ui.label(format!(
                                    "State: {:?} ({:?} {}/{})",
                                    state_name.0,
                                    animation_frame.kind,
                                    animation_frame.frame,
                                    frame_count
                                ));
                            } else {
                                ui.label(format!(
                                    "State: {:?} ({:?})",
                                    state_name.0, animation_frame.kind
                                ));
                            }
                        }
                        FighterDebugRowKind::Velocity => {
                            ui.label(format!(
                                "Velocity: ({:.2}, {:.2})",
                                velocity.0.x, velocity.0.y
                            ));
                        }
                        FighterDebugRowKind::Input => {
                            const STICK_SIZE: f32 = 50.0;
                            const STICK_SIZE_FRACTION: f32 = 0.75;
                            let last_frame = input.get_last_frame();
                            let movement = egui::Vec2::new(
                                last_frame.movement.x.to_num(),
                                -last_frame.movement.y.to_num::<f32>(),
                            );

                            ui.horizontal(|ui| {
                                let (_, rect) = ui.allocate_space(egui::Vec2::new(
                                    STICK_SIZE * (1.0 + STICK_SIZE_FRACTION),
                                    STICK_SIZE * (1.0 + STICK_SIZE_FRACTION),
                                ));
                                let painter = ui.painter();
                                let button_bg =
                                    egui::Rgba::WHITE * egui::Rgba::from_white_alpha(0.25);
                                painter.circle_filled(rect.center(), STICK_SIZE * 0.5, button_bg);
                                painter.circle_filled(
                                    rect.center() + STICK_SIZE * 0.5 * movement,
                                    STICK_SIZE * STICK_SIZE_FRACTION * 0.5,
                                    button_bg,
                                );
                                ui.label(format!(
                                    "X: {:.2}\nY:{:.2}",
                                    last_frame.movement.x, last_frame.movement.y
                                ));
                            });
                        }
                        FighterDebugRowKind::Position => {
                            ui.label(format!("Position: {:?}", translation.0));
                        }
                        FighterDebugRowKind::Facing => {
                            ui.label(format!("Facing: {:?}", facing));
                        }
                    }
                }
            },
        );
    }
    Ok(())
}

pub fn animation_debug(query: Query<(&GlobalTransform, &FighterBoneMatrices)>, mut gizmos: Gizmos) {
    for (global_transform, matrices) in query {
        for (_, matrix) in matrices.iter() {
            let pos = global_transform.transform_point(Vec3::new(
                matrix.cols[3][0].to_num(),
                matrix.cols[3][1].to_num(),
                matrix.cols[3][2].to_num(),
            ));
            gizmos.cross(pos, 0.5, bevy::color::palettes::css::RED);
            gizmos.axes(*global_transform, 1.0);
        }
    }
}

pub fn attack_debug(
    query: Query<(
        &FighterHitboxes,
        &FighterBoneMatrices,
        &FighterTranslation,
        &FighterFacingDirection,
    )>,
    attack_scripts: Res<Assets<FighterAttackScript>>,
    mut gizmos: Gizmos,
) {
    for (hitboxes, matrices, translation, facing_direction) in query {
        if let Some(script) = &hitboxes.attack_script {
            let script = attack_scripts
                .get(script)
                .expect("Script ref should be valid");
            let player_trf = translation.get_3d_transform(facing_direction);

            for i in hitboxes.active_hitboxes.iter() {
                let offset = script.hitboxes[*i].offset;
                let radius = script.hitboxes[*i].radius;
                let bone = &script.hitboxes[*i].bone;

                let trfs = matrices.get(bone);
                if let Some((prev_trf, curr_trf)) = trfs {
                    let prev_trf = player_trf.mul(prev_trf);
                    let curr_trf = player_trf.mul(curr_trf);
                    
                    let prev_pos = prev_trf.transform_point(offset);
                    let curr_pos = curr_trf.transform_point(offset);

                    let prev_pos_draw = Vec3::new(prev_pos.x.to_num(), prev_pos.y.to_num(), prev_pos.z.to_num());
                    let curr_pos_draw = Vec3::new(curr_pos.x.to_num(), curr_pos.y.to_num(), curr_pos.z.to_num());

                    let dist = Dir3::new_and_length(curr_pos_draw - prev_pos_draw);

                    if let Ok((aim_dir, dist)) = dist {
                        let attack_capsule = Capsule3d {
                            half_length: dist * 0.5,
                            radius: radius.to_num()
                        };
                        let mut capsule_draw_trf = Transform::IDENTITY;
                        capsule_draw_trf.align(Dir3::Y, aim_dir, Dir3::X, aim_dir.any_orthonormal_vector());
                        capsule_draw_trf.translation = (prev_pos_draw + curr_pos_draw) * 0.5;
                        gizmos.primitive_3d(&attack_capsule, capsule_draw_trf.to_isometry(), bevy::color::palettes::css::RED);
                    }



                    gizmos.sphere(prev_pos_draw, radius.to_num(), bevy::color::palettes::css::RED.darker(0.5));
                    gizmos.sphere(curr_pos_draw, radius.to_num(), bevy::color::palettes::css::RED);

                } else {
                    warn!("Bone not found required by attack script: {}", bone);
                }
            }
        }
    }
}
