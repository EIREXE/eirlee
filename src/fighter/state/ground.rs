//! Ground movement math shared by every grounded state. Deliberately free of
//! ECS types so it stays cheap to reason about and to unit-test — it has to be
//! bit-for-bit deterministic for rollback.

use crate::fighter::state::FighterStateContext;
use crate::fighter::{
    FighterAttributes, FighterECB, FighterPreviousTranslation, FighterTranslation, FighterVelocity,
};
use crate::game_settings::GameSettings;
use crate::stage::StagePoly;
use crate::stage::line::{StageCollision, StageLineID, StagePolyLineSegment, StagePolyLineSegmentType};

use bevy::prelude::*;

#[derive(Reflect, Hash, Clone, Debug)]
pub struct GroundedStateCommon {
    pub current_line_id: StageLineID,
}

pub fn apply_grounded_friction(friction: f32, ground_vel: f32) -> f32 {
    if friction.abs() > ground_vel.abs() {
        -ground_vel
    } else {
        (-ground_vel.signum()) * friction.abs()
    }
}

pub fn move_accelerate() {}

pub fn compute_ground_accel(
    accel: f32,
    target_vel: f32,
    gr_vel: f32,
    attribs: &FighterAttributes,
    game_settings: &GameSettings,
) -> f32 {
    if target_vel == 0.0 {
        apply_grounded_friction(attribs.ground_friction, gr_vel)
    } else {
        let mut accel = accel;
        let ground_max_horizontal_velocity =
            game_settings.figher_common.ground_max_horizontal_velocity;
        if !(gr_vel * accel < 0.0) {
            // accelerating, not reversing
            if accel > 0.0 {
                if gr_vel + accel > target_vel {
                    accel = -attribs.ground_friction;
                    if gr_vel + accel < target_vel {
                        accel = target_vel - gr_vel;
                    }
                    if gr_vel + accel > ground_max_horizontal_velocity {
                        accel = ground_max_horizontal_velocity - gr_vel;
                    }
                }
            } else if gr_vel + accel < target_vel {
                accel = attribs.ground_friction;
                if gr_vel + accel > target_vel {
                    accel = target_vel - gr_vel;
                }
                if gr_vel + accel < -ground_max_horizontal_velocity {
                    accel = -ground_max_horizontal_velocity - gr_vel;
                }
            }
        }
        accel
    }
}

pub enum GroundedMotionResult {
    InGround,
    InAir,
}

pub fn apply_grounded_motion(
    velocity: &FighterVelocity,
    translation: &mut FighterTranslation,
    prev_translation: &mut FighterPreviousTranslation,
) {
    // Vertical velocity should be 0 on the ground
    prev_translation.0 = translation.0;
    translation.0 += Vec2::new(velocity.0.x ,0.0);
}

pub fn collide_with_stage_grounded(
    state_context: &mut FighterStateContext,
    ground_common: &mut GroundedStateCommon,
    can_walk_off: bool
) -> GroundedMotionResult {
    let translation = &mut state_context.translation;
    let ecb = &state_context.ecb;
    let stage_collision = &state_context.stage_collision;

    let stage_poly = stage_collision
        .stage_polys
        .get(ground_common.current_line_id.polygon);

    if let Some(stage_poly) = stage_poly {
        let ecb_bottom_pos = translation.0 + ecb.get_bottom_point();
        let left_segment_idx = ground_common
            .current_line_id
            .segment
            .checked_sub(1)
            .unwrap_or(0);
        let right_segment_idx =
            (ground_common.current_line_id.segment + 1) % stage_poly.segments.len();

        // First, we check the current segment, if that fails, we check the surrounding ones
        let grounded_ecb_bottom_pos = [
            ground_common.current_line_id.segment.clone(),
            left_segment_idx,
            right_segment_idx,
        ]
        .iter()
        .find_map(|segment_idx| {
            let segment = stage_poly.get_segment(*segment_idx)?;
            if segment.segment_type != StagePolyLineSegmentType::Floor {
                return None;
            }

            let line_dir = segment.segment.direction();
            let projected_point = segment.segment.point1() + (ecb_bottom_pos - segment.segment.point1()).project_onto(line_dir.as_vec2());
            let closest_point = segment.segment.closest_point(ecb_bottom_pos);
            let distance = closest_point.distance(projected_point);
            
            if distance < 0.01 {
                return Some((projected_point, *segment_idx, segment.normal));
            }

            None
        });

        if let Some((bp, new_segment_idx, normal)) = grounded_ecb_bottom_pos {
            translation.0 = bp - ecb.get_bottom_point();
            state_context.velocity.0 = state_context.velocity.0 - state_context.velocity.0.project_onto_normalized(normal);
            ground_common.current_line_id.segment = new_segment_idx;
            GroundedMotionResult::InGround
        } else {
            GroundedMotionResult::InAir
        }
    } else {
        GroundedMotionResult::InAir
    }
}
