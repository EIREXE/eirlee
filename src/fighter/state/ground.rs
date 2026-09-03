//! Ground movement math shared by every grounded state. Deliberately free of
//! ECS types so it stays cheap to reason about and to unit-test — it has to be
//! bit-for-bit deterministic for rollback.

use crate::fighter::state::{
    FighterState, FighterStateContext, dash, ground_attack, turn, wait, walk,
};
use crate::fighter::{
    FighterAttributes, FighterPreviousTranslation, FighterTranslation, FighterVelocity,
};
use crate::game_settings::GameSettings;
use crate::math::int::FGi32;
use crate::math::vec::FGVec2;
use crate::stage::line::{StageLineID, StagePolyLineSegmentType};

use bevy::prelude::*;

#[derive(Reflect, Hash, Clone, Debug)]
pub struct GroundedStateCommon {
    pub current_line_id: StageLineID,
}

pub fn apply_grounded_friction(friction: FGi32, ground_vel: FGi32) -> FGi32 {
    if friction.abs() > ground_vel.abs() {
        -ground_vel
    } else {
        (-ground_vel.signum()) * friction.abs()
    }
}

pub fn move_accelerate() {}

pub fn compute_ground_accel(
    accel: FGi32,
    target_vel: FGi32,
    gr_vel: FGi32,
    attribs: &FighterAttributes,
    game_settings: &GameSettings,
) -> FGi32 {
    if target_vel == FGi32::ZERO {
        apply_grounded_friction(attribs.ground_friction, gr_vel)
    } else {
        let mut accel = accel;
        let ground_max_horizontal_velocity =
            game_settings.fighter_common.ground_max_horizontal_velocity;
        if !(gr_vel * accel < FGi32::ZERO) {
            // accelerating, not reversing
            if accel > FGi32::ZERO {
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
    translation.0 += FGVec2::new(velocity.0.x, FGi32::ZERO);
}

pub fn collide_with_stage_grounded(
    state_context: &mut FighterStateContext,
    ground_common: &mut GroundedStateCommon,
    _can_walk_off: bool,
) -> GroundedMotionResult {
    let translation = &mut state_context.translation;
    let ecb = &state_context.ecb;
    let stage_collision = &state_context.stage_collision;

    let stage_poly = stage_collision
        .stage_polys
        .get(ground_common.current_line_id.polygon);

    if let Some(stage_poly) = stage_poly {
        let ecb_bottom_pos = translation.0 + ecb.get_bottom_point();
        let current_segment_idx = ground_common.current_line_id.segment;

        // First, we check the current segment, if that fails, we check the surrounding ones
        let grounded_ecb_bottom_pos = [
            Some(current_segment_idx),
            stage_poly.get_prev_segment_index(current_segment_idx),
            stage_poly.get_next_segment_index(current_segment_idx),
        ]
        .into_iter()
        .flatten()
        .find_map(|segment_idx| {
            let segment = stage_poly.get_segment(segment_idx)?;
            if segment.segment_type != StagePolyLineSegmentType::Floor {
                return None;
            }

            let line_dir = segment.segment.direction();
            let projected_point = segment.segment.point1()
                + (ecb_bottom_pos - segment.segment.point1()).project_onto(line_dir);
            let closest_point = segment.segment.closest_point(ecb_bottom_pos);
            let distance = closest_point.distance(projected_point);

            if distance < 0.01 {
                return Some((projected_point, segment_idx, segment.normal));
            }

            None
        });

        if let Some((bp, new_segment_idx, normal)) = grounded_ecb_bottom_pos {
            translation.0 = bp - ecb.get_bottom_point();
            state_context.velocity.0 =
                state_context.velocity.0 - state_context.velocity.0.project_onto_normalized(normal);
            ground_common.current_line_id.segment = new_segment_idx;
            GroundedMotionResult::InGround
        } else {
            GroundedMotionResult::InAir
        }
    } else {
        GroundedMotionResult::InAir
    }
}

pub fn grounded_movement_common_interrupts(
    state_context: &FighterStateContext,
    ground_common: &GroundedStateCommon,
) -> Option<FighterState> {
    super::jump::check_input(state_context, ground_common)
}

pub fn grounded_movement_standstill_common_interrupts(
    state_context: &FighterStateContext,
    ground_common: &GroundedStateCommon,
) -> Option<FighterState> {
    if let Some(state) = ground_attack::check_interrupt(state_context, &ground_common) {
        Some(state)
    } else if let Some(state) = dash::check_input(state_context, &ground_common) {
        Some(state)
    } else if let Some(state) = turn::check_input(state_context, &ground_common) {
        Some(state)
    } else if let Some(state) = walk::check_input(state_context, &ground_common) {
        Some(state)
    } else if let Some(state) = wait::check_input(state_context, &ground_common) {
        Some(state)
    } else {
        None
    }
}
