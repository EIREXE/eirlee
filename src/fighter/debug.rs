//! Fighter debug views: gizmos and the egui inspector window.

use std::sync::OnceLock;

use bevy::color::palettes::css::ORANGE;
use bevy::{color::palettes::css::YELLOW, prelude::*};
use bevy_egui::{EguiContexts, egui};

use crate::debug_tools::{DebugSettings, FighterDebugRowKind};
use crate::fighter::animation::FighterAnimationFrame;
use crate::fighter::attack::FighterSolvedHurtboxes;
use crate::fighter::baked_animation::{BakedFighterAnimations, FighterBoneMatrices};
use crate::fighter::ecb::FighterPreviousECB;
use crate::fighter::hurtbox::FixedAffineCapsule;
use crate::fighter::state::StateNameDebug;
use crate::fighter::visual::FighterAnimations;
use crate::fighter::{
    FighterECB, FighterFacingDirection, FighterHitboxes, FighterPreviousTranslation,
    FighterTranslation, FighterVelocity,
};
use crate::input::FighterInput;
use crate::math::int::FGi32;
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
        let (_, rotation, translation) = global_transform.to_scale_rotation_translation();
        let marker_transform = Transform::from_translation(translation).with_rotation(rotation);
        for (_, matrix) in matrices.iter() {
            let pos = marker_transform.transform_point(Vec3::new(
                matrix.cols[3][0].to_num(),
                matrix.cols[3][1].to_num(),
                matrix.cols[3][2].to_num(),
            ));
            gizmos.cross(pos, 0.5, bevy::color::palettes::css::RED);
            gizmos.axes(marker_transform, 1.0);
        }
    }
}

pub fn hurtbox_debug(query: Query<&FighterSolvedHurtboxes>, mut gizmos: Gizmos) {
    for hurtboxes in &query {
        for hurtbox in &hurtboxes.0 {
            draw_affine_capsule(&mut gizmos, hurtbox.capsule, YELLOW);
        }
    }
}

fn draw_affine_capsule(
    gizmos: &mut Gizmos,
    capsule: FixedAffineCapsule,
    color: impl Into<Color> + Copy,
) {
    let color = color.into();
    let wireframe = capsule_wireframe();
    let transform = capsule.transform.to_mat4();
    let radius = capsule.radius.to_num::<f32>();
    let half_length = capsule.half_length.to_num::<f32>();
    let vertices: Vec<_> = wireframe
        .vertices
        .iter()
        .map(|vertex| {
            transform.transform_point3(
                vertex.direction * radius + Vec3::Y * vertex.center_sign * half_length,
            )
        })
        .collect();
    for &(start, end) in &wireframe.lines {
        gizmos.line(vertices[start], vertices[end], color);
    }
}

struct CapsuleWireVertex {
    direction: Vec3,
    center_sign: f32,
}

struct CapsuleWireframe {
    vertices: Vec<CapsuleWireVertex>,
    lines: Vec<(usize, usize)>,
}

fn capsule_wireframe() -> &'static CapsuleWireframe {
    const RESOLUTION: usize = 16;
    const HEMISPHERE_STEPS: usize = 4;
    static WIREFRAME: OnceLock<CapsuleWireframe> = OnceLock::new();

    WIREFRAME.get_or_init(|| {
        let mut vertices = Vec::new();
        let mut lines = Vec::new();
        let mut lower_ring = Vec::with_capacity(RESOLUTION);
        let mut upper_ring = Vec::with_capacity(RESOLUTION);

        for index in 0..RESOLUTION {
            let angle = index as f32 * std::f32::consts::TAU / RESOLUTION as f32;
            let (sin, cos) = angle.sin_cos();
            lower_ring.push(vertices.len());
            vertices.push(CapsuleWireVertex {
                direction: Vec3::new(cos, 0.0, sin),
                center_sign: -1.0,
            });
            upper_ring.push(vertices.len());
            vertices.push(CapsuleWireVertex {
                direction: Vec3::new(cos, 0.0, sin),
                center_sign: 1.0,
            });
        }

        let lower_pole = vertices.len();
        vertices.push(CapsuleWireVertex {
            direction: Vec3::NEG_Y,
            center_sign: -1.0,
        });
        let upper_pole = vertices.len();
        vertices.push(CapsuleWireVertex {
            direction: Vec3::Y,
            center_sign: 1.0,
        });

        for index in 0..RESOLUTION {
            let next = (index + 1) % RESOLUTION;
            lines.push((lower_ring[index], lower_ring[next]));
            lines.push((upper_ring[index], upper_ring[next]));
            lines.push((lower_ring[index], upper_ring[index]));

            let angle = index as f32 * std::f32::consts::TAU / RESOLUTION as f32;
            let (sin_azimuth, cos_azimuth) = angle.sin_cos();
            let mut lower_previous = lower_ring[index];
            let mut upper_previous = upper_ring[index];
            for step in 1..HEMISPHERE_STEPS {
                let latitude = step as f32 * std::f32::consts::FRAC_PI_2 / HEMISPHERE_STEPS as f32;
                let (sin_latitude, cos_latitude) = latitude.sin_cos();

                let lower = vertices.len();
                vertices.push(CapsuleWireVertex {
                    direction: Vec3::new(
                        cos_azimuth * cos_latitude,
                        -sin_latitude,
                        sin_azimuth * cos_latitude,
                    ),
                    center_sign: -1.0,
                });
                lines.push((lower_previous, lower));
                lower_previous = lower;

                let upper = vertices.len();
                vertices.push(CapsuleWireVertex {
                    direction: Vec3::new(
                        cos_azimuth * cos_latitude,
                        sin_latitude,
                        sin_azimuth * cos_latitude,
                    ),
                    center_sign: 1.0,
                });
                lines.push((upper_previous, upper));
                upper_previous = upper;
            }
            lines.push((lower_previous, lower_pole));
            lines.push((upper_previous, upper_pole));
        }

        CapsuleWireframe { vertices, lines }
    })
}

#[cfg(test)]
mod capsule_debug_tests {
    use super::*;

    #[test]
    fn animation_markers_ignore_visual_scale() {
        let global = GlobalTransform::from(Transform {
            translation: Vec3::new(3.0, 4.0, 5.0),
            rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            scale: Vec3::splat(10.0),
        });
        let (_, rotation, translation) = global.to_scale_rotation_translation();
        let marker_transform = Transform::from_translation(translation).with_rotation(rotation);

        let marker = marker_transform.transform_point(Vec3::new(2.0, 0.0, 0.0));
        assert!(marker.abs_diff_eq(Vec3::new(3.0, 4.0, 3.0), 1e-5));
    }

    #[test]
    fn capsule_wireframe_caps_extend_in_opposite_directions() {
        let wireframe = capsule_wireframe();
        let lower: Vec<_> = wireframe
            .vertices
            .iter()
            .filter(|vertex| vertex.center_sign < 0.0)
            .collect();
        let upper: Vec<_> = wireframe
            .vertices
            .iter()
            .filter(|vertex| vertex.center_sign > 0.0)
            .collect();

        assert!(lower.iter().all(|vertex| vertex.direction.y <= 0.0));
        assert!(upper.iter().all(|vertex| vertex.direction.y >= 0.0));
        assert!(lower.iter().any(|vertex| vertex.direction == Vec3::NEG_Y));
        assert!(upper.iter().any(|vertex| vertex.direction == Vec3::Y));
    }
}

pub fn attack_debug(
    query: Query<(
        &FighterHitboxes,
        &FighterBoneMatrices,
        &FighterTranslation,
        &FighterFacingDirection,
        &FighterAnimationFrame,
    )>,
    _attack_scripts: Res<Assets<FighterAttackScript>>,
    mut gizmos: Gizmos,
) {
    for (hitboxes, _matrices, _translation, _facing_direction, _frame) in query {
        if let Some(_script) = &hitboxes.attack_script {
            for solved_hitbox in hitboxes.active_hitboxes_solved.iter() {
                let start = solved_hitbox.capsule.start.to_vec3();
                let end = solved_hitbox.capsule.end.to_vec3();
                let center =
                    (solved_hitbox.capsule.start + solved_hitbox.capsule.end) * FGi32::lit("0.5");
                let length = (start - end).length();

                let attack_capsule = Capsule3d {
                    half_length: length * 0.5,
                    radius: solved_hitbox.capsule.radius.to_num(),
                };

                let aim_dir = (end - start).normalize_or(Vec3::Y);

                let mut capsule_draw_trf = Transform::IDENTITY;
                capsule_draw_trf.translation = center.to_vec3();
                capsule_draw_trf.align(Dir3::Y, aim_dir, Dir3::X, aim_dir.any_orthonormal_vector());

                gizmos.primitive_3d(
                    &attack_capsule,
                    capsule_draw_trf.to_isometry(),
                    bevy::color::palettes::css::RED,
                );

                gizmos.cross(start, 1.0, bevy::color::palettes::css::GREEN);
                gizmos.cross(end, 1.0, bevy::color::palettes::css::GREEN);
            }
        }
    }
}
