//! Presentation-only state and tuning for the match camera.

use bevy::prelude::*;
use bevy_ggrs::GgrsFrameTiming;

use crate::{
    debug_tools::DebugSettings,
    fighter::{manifest::FighterManifest, motion::sample_presentation_translation},
    stage::manifest::StageCameraProfile,
};

const EMPTY_FRAME_HALF_SIZE: f32 = 40.0;
const SUBJECT_COUNT_SCALES: [f32; 4] = [1.5, 1.32, 1.16, 1.0];
const VERTICAL_PAN_REFERENCE_OFFSET: f32 = -30.0;
const EXTENT_APPROACH_PER_SECOND: f32 = 30.0;
const MELEE_DOWNWARD_EXPANSION_START_DEPTH: f32 = 80.0;
const MELEE_DOWNWARD_EXPANSION_END_DEPTH: f32 = 5000.0;
const MELEE_DOWNWARD_EXPANSION_MIN: f32 = 10.0;
const MELEE_DOWNWARD_EXPANSION_MAX: f32 = 400.0;
const MELEE_INTEREST_FOLLOW_MIN: f32 = 0.05;
const MELEE_INTEREST_FOLLOW_MAX: f32 = 0.1;
const MELEE_INTEREST_FOLLOW_START_SPREAD: f32 = 120.0;
const MELEE_INTEREST_FOLLOW_END_SPREAD: f32 = 900.0;
const MELEE_EYE_FOLLOW: f32 = 0.15;
const MELEE_FOV_FOLLOW: f32 = 0.1;

/// The rectangle contributed by one tracked subject on the gameplay plane.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CameraSubjectBounds {
    pub center: Vec2,
    pub left_extent: f32,
    pub right_extent: f32,
    pub bottom_extent: f32,
    pub top_extent: f32,
}

/// Presentation-only fighter camera extents that ease toward the manifest
/// values. This prevents a facing change from instantly expanding the frame.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct FighterCameraExtents {
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    initialized: bool,
}

/// The aggregate rectangle the camera must keep in view on the gameplay plane.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CameraFrameBounds {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
}

impl CameraFrameBounds {
    fn around(center: Vec2, half_size: f32) -> Self {
        Self {
            left: center.x - half_size,
            right: center.x + half_size,
            bottom: center.y - half_size,
            top: center.y + half_size,
        }
    }

    fn width(self) -> f32 {
        self.right - self.left
    }

    fn height(self) -> f32 {
        self.top - self.bottom
    }
}

/// A solved camera pose before it is smoothed and applied to the render camera.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CameraPose {
    pub interest: Vec2,
    pub eye: Vec2,
    pub depth: f32,
    pub fov: f32,
}

/// Runtime presentation state for the match camera.
///
/// This intentionally remains outside rollback: it follows already-simulated
/// fighter positions and must not affect gameplay.
#[derive(Component, Clone, Copy, Debug)]
pub struct MatchCamera {
    pub current: CameraPose,
    pub target: CameraPose,
    pub frame: CameraFrameBounds,
    pub shake_offset: Vec2,
    pub needs_snap: bool,
}

impl Default for MatchCamera {
    fn default() -> Self {
        Self {
            current: CameraPose::default(),
            target: CameraPose::default(),
            frame: CameraFrameBounds::default(),
            shake_offset: Vec2::ZERO,
            needs_snap: true,
        }
    }
}

/// Owns the presentation-only match-camera systems.
#[derive(Default)]
pub struct MatchCameraPlugin;

impl Plugin for MatchCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_match_camera,
                debug_draw_camera.run_if(crate::debug_tools::camera_debug_enabled),
            )
                .chain()
                .run_if(in_state(crate::AppState::InMatch)),
        );
    }
}

/// Produces the aggregate rectangle to frame, constraining target contributions
/// to the explicit stage camera rectangle.
pub fn frame_subjects(
    subjects: impl IntoIterator<Item = CameraSubjectBounds>,
    profile: &StageCameraProfile,
    current_depth: f32,
) -> CameraFrameBounds {
    let subjects: Vec<_> = subjects.into_iter().collect();
    let scale = profile.subject_scale
        * SUBJECT_COUNT_SCALES[subjects
            .len()
            .saturating_sub(1)
            .min(SUBJECT_COUNT_SCALES.len() - 1)];

    let mut frame = if subjects.is_empty() {
        CameraFrameBounds::around(profile.origin, EMPTY_FRAME_HALF_SIZE)
    } else {
        let mut frame = CameraFrameBounds {
            left: f32::INFINITY,
            right: f32::NEG_INFINITY,
            bottom: f32::INFINITY,
            top: f32::NEG_INFINITY,
        };

        for subject in subjects {
            let center = subject.center.clamp(
                Vec2::new(profile.left, profile.bottom),
                Vec2::new(profile.right, profile.top),
            );
            let left_extent = subject.left_extent * scale;
            let right_extent = subject.right_extent * scale;
            let bottom_extent = subject.bottom_extent * scale;
            let top_extent = subject.top_extent * scale;
            frame.left = frame
                .left
                .min((center.x - left_extent).clamp(profile.left, profile.right));
            frame.right = frame
                .right
                .max((center.x + right_extent).clamp(profile.left, profile.right));
            frame.bottom = frame
                .bottom
                .min((center.y - bottom_extent).clamp(profile.bottom, profile.top));
            frame.top = frame
                .top
                .max((center.y + top_extent).clamp(profile.bottom, profile.top));
        }
        frame
    };

    frame.bottom -= melee_downward_expansion(current_depth);
    frame
}

fn melee_downward_expansion(depth: f32) -> f32 {
    let fraction = ((depth.abs() - MELEE_DOWNWARD_EXPANSION_START_DEPTH)
        / (MELEE_DOWNWARD_EXPANSION_END_DEPTH - MELEE_DOWNWARD_EXPANSION_START_DEPTH))
        .clamp(0.0, 1.0);
    MELEE_DOWNWARD_EXPANSION_MIN
        + (MELEE_DOWNWARD_EXPANSION_MAX - MELEE_DOWNWARD_EXPANSION_MIN) * fraction
}

/// Solves a perspective camera pose that contains `frame` at the supplied
/// aspect ratio. The profile supplies Melee-style stage bias and pan limits.
pub fn solve_pose(
    frame: CameraFrameBounds,
    profile: &StageCameraProfile,
    aspect_ratio: f32,
) -> CameraPose {
    let aspect_ratio = aspect_ratio.max(f32::EPSILON);
    let vertical_half_fov = profile.vertical_fov_degrees.to_radians() * 0.5;
    let spread = frame.width().max(frame.height());
    let vertical_bias = ((spread - 60.0) / 60.0).clamp(0.0, 1.0) * 0.0682;
    let vertical_center = profile.origin.y
        + ((frame.bottom - profile.origin.y) + (frame.top - profile.origin.y))
            * (0.5 - vertical_bias);
    let vertical_angle = (-(vertical_center + VERTICAL_PAN_REFERENCE_OFFSET)
        * profile.vertical_pan_degrees_per_unit)
        .to_radians()
        .clamp(
            -profile.max_downward_pan_degrees.to_radians(),
            profile.max_upward_pan_degrees.to_radians(),
        )
        + profile.vertical_pan_degrees.to_radians();
    let up_angle = vertical_half_fov + vertical_angle;
    let down_angle = vertical_half_fov - vertical_angle;
    let vertical_depth = frame.height() / (up_angle.tan() + down_angle.tan()).max(f32::EPSILON);
    let vertical_eye_offset = vertical_depth * vertical_angle.tan();
    let interest_y = vertical_eye_offset + frame.top - vertical_depth * up_angle.tan();

    let horizontal_center = (frame.left + frame.right) * 0.5;
    let horizontal_angle = (-(horizontal_center - profile.origin.x)
        * profile.horizontal_pan_degrees_per_unit)
        .to_radians()
        .clamp(
            -profile.max_horizontal_pan_degrees.to_radians(),
            profile.max_horizontal_pan_degrees.to_radians(),
        );
    let right_tangent = aspect_ratio * (vertical_half_fov - horizontal_angle).tan();
    let left_tangent = aspect_ratio * (vertical_half_fov + horizontal_angle).tan();
    let horizontal_depth = frame.width() / (right_tangent + left_tangent).max(f32::EPSILON);
    let horizontal_eye_offset = aspect_ratio * horizontal_depth * horizontal_angle.tan();
    let interest_x = frame.right - horizontal_depth * right_tangent - horizontal_eye_offset;

    let depth = horizontal_depth
        .max(vertical_depth)
        .clamp(profile.min_depth, profile.max_depth);
    let interest = Vec2::new(interest_x, interest_y);

    correct_pose_to_camera_bounds(
        CameraPose {
            eye: interest + Vec2::new(horizontal_eye_offset, -vertical_eye_offset),
            interest,
            depth,
            fov: profile.vertical_fov_degrees.to_radians(),
        },
        profile,
        aspect_ratio,
    )
}

fn correct_pose_to_camera_bounds(
    mut pose: CameraPose,
    profile: &StageCameraProfile,
    aspect_ratio: f32,
) -> CameraPose {
    let Some(corners) = view_corners_on_plane(pose, aspect_ratio) else {
        return pose;
    };
    let mut left = f32::INFINITY;
    let mut right = f32::NEG_INFINITY;
    let mut bottom = f32::INFINITY;
    let mut top = f32::NEG_INFINITY;
    for corner in corners {
        left = left.min(corner.x);
        right = right.max(corner.x);
        bottom = bottom.min(corner.y);
        top = top.max(corner.y);
    }
    let correction = Vec2::new(
        axis_correction(left, right, profile.left, profile.right),
        axis_correction(bottom, top, profile.bottom, profile.top),
    );
    pose.eye += correction;
    pose.interest += correction;
    pose
}

fn axis_correction(minimum: f32, maximum: f32, lower_bound: f32, upper_bound: f32) -> f32 {
    let lower_overlap = (lower_bound - minimum).max(0.0);
    let upper_overlap = (upper_bound - maximum).min(0.0);
    match (lower_overlap > 0.0, upper_overlap < 0.0) {
        (true, true) => (lower_overlap + upper_overlap) * 0.5,
        (true, false) => lower_overlap,
        (false, true) => upper_overlap,
        (false, false) => 0.0,
    }
}

fn view_corners_on_plane(pose: CameraPose, aspect_ratio: f32) -> Option<[Vec2; 4]> {
    let eye = Vec3::new(pose.eye.x, pose.eye.y, pose.depth);
    let interest = Vec3::new(pose.interest.x, pose.interest.y, 0.0);
    let forward = (interest - eye).try_normalize()?;
    let right = forward.cross(Vec3::Y).try_normalize()?;
    let up = right.cross(forward).try_normalize()?;
    let vertical_tangent = (pose.fov * 0.5).tan();
    let horizontal_tangent = vertical_tangent * aspect_ratio;
    [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)]
        .map(|(horizontal, vertical)| {
            let ray = (forward
                + right * horizontal_tangent * horizontal
                + up * vertical_tangent * vertical)
                .try_normalize()?;
            let distance = -eye.z / ray.z;
            (distance >= 0.0).then(|| (eye + ray * distance).truncate())
        })
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .and_then(|corners| corners.try_into().ok())
}

fn update_match_camera(
    time: Res<Time>,
    mut fighters: Query<
        (
            &crate::fighter::motion::FighterTranslation,
            &crate::fighter::motion::FighterPreviousTranslation,
            &crate::fighter::Fighter,
            &crate::fighter::FighterFacingDirection,
            &mut FighterCameraExtents,
        ),
        With<crate::player::Player>,
    >,
    camera_profiles: Query<&StageCameraProfile, With<MatchCamera>>,
    mut cameras: Query<(&mut Transform, &mut Projection, &mut MatchCamera), With<MatchCamera>>,
    manifests: Res<Assets<FighterManifest>>,
    timing: Res<GgrsFrameTiming>,
    debug_settings: Res<DebugSettings>,
) {
    let Ok(profile) = camera_profiles.single() else {
        return;
    };
    let subjects = fighters
        .iter_mut()
        .map(
            |(translation, previous_translation, fighter, facing, mut extents)| {
                let manifest = manifests
                    .get(&fighter.manifest)
                    .expect("Fighter manifest should be valid");
                fighter_subject_bounds(
                    sample_presentation_translation(
                        *previous_translation,
                        *translation,
                        timing.overstep_fraction(),
                        debug_settings.motion_sampling,
                    ),
                    manifest.camera,
                    *facing,
                    profile,
                    &mut extents,
                    time.delta_secs(),
                )
            },
        )
        .collect::<Vec<_>>();

    for (mut transform, mut projection, mut camera) in &mut cameras {
        let Projection::Perspective(projection) = projection.as_mut() else {
            continue;
        };
        let frame_depth = if camera.needs_snap {
            transform.translation.z.abs()
        } else {
            camera.current.depth
        };
        camera.frame = frame_subjects(subjects.iter().copied(), profile, frame_depth);
        camera.target = solve_pose(camera.frame, profile, projection.aspect_ratio);

        if camera.needs_snap {
            camera.current = camera.target;
            camera.needs_snap = false;
        } else {
            let spread = camera.frame.width().max(camera.frame.height());
            let interest_rate = if spread <= MELEE_INTEREST_FOLLOW_START_SPREAD {
                MELEE_INTEREST_FOLLOW_MIN
            } else if spread >= MELEE_INTEREST_FOLLOW_END_SPREAD {
                MELEE_INTEREST_FOLLOW_MAX
            } else {
                MELEE_INTEREST_FOLLOW_MIN
                    + (MELEE_INTEREST_FOLLOW_MAX - MELEE_INTEREST_FOLLOW_MIN)
                        * (spread - MELEE_INTEREST_FOLLOW_START_SPREAD)
                        / (MELEE_INTEREST_FOLLOW_END_SPREAD - MELEE_INTEREST_FOLLOW_START_SPREAD)
            } * profile.tracking_smoothness;
            let interest_alpha = sixty_hz_lerp_alpha(interest_rate, time.delta_secs());
            let eye_alpha = sixty_hz_lerp_alpha(
                MELEE_EYE_FOLLOW * profile.tracking_smoothness,
                time.delta_secs(),
            );
            let fov_alpha = sixty_hz_lerp_alpha(MELEE_FOV_FOLLOW, time.delta_secs());
            camera.current.interest = camera
                .current
                .interest
                .lerp(camera.target.interest, interest_alpha);
            camera.current.eye = camera.current.eye.lerp(camera.target.eye, eye_alpha);
            camera.current.depth += (camera.target.depth - camera.current.depth) * eye_alpha;
            camera.current.fov += (camera.target.fov - camera.current.fov) * fov_alpha;
        }

        projection.fov = camera.current.fov;
        transform.translation = Vec3::new(
            camera.current.eye.x + camera.shake_offset.x,
            camera.current.eye.y + camera.shake_offset.y,
            camera.current.depth,
        );
        transform.look_at(
            Vec3::new(camera.current.interest.x, camera.current.interest.y, 0.0),
            Vec3::Y,
        );
    }
}

fn sixty_hz_lerp_alpha(per_frame_factor: f32, delta_secs: f32) -> f32 {
    1.0 - (1.0 - per_frame_factor.clamp(0.0, 1.0)).powf(delta_secs * 60.0)
}

fn fighter_subject_bounds(
    translation: Vec2,
    profile: crate::fighter::FighterCameraProfile,
    facing: crate::fighter::FighterFacingDirection,
    stage_profile: &StageCameraProfile,
    extents: &mut FighterCameraExtents,
    delta_secs: f32,
) -> CameraSubjectBounds {
    let (left, right) = match facing {
        crate::fighter::FighterFacingDirection::Left => (
            profile.forward_extent.to_num::<f32>() * stage_profile.fighter_forward_extent_scale,
            profile.backward_extent.to_num(),
        ),
        crate::fighter::FighterFacingDirection::Right => (
            profile.backward_extent.to_num(),
            profile.forward_extent.to_num::<f32>() * stage_profile.fighter_forward_extent_scale,
        ),
    };
    let target = FighterCameraExtents {
        left,
        right,
        bottom: profile.downward_extent.to_num(),
        top: profile.upward_extent.to_num(),
        initialized: true,
    };
    if !extents.initialized {
        *extents = target;
    } else {
        let maximum_delta = EXTENT_APPROACH_PER_SECOND * delta_secs;
        extents.left = approach(extents.left, target.left, maximum_delta);
        extents.right = approach(extents.right, target.right, maximum_delta);
        extents.bottom = approach(extents.bottom, target.bottom, maximum_delta);
        extents.top = approach(extents.top, target.top, maximum_delta);
    }

    CameraSubjectBounds {
        center: translation + Vec2::Y * profile.vertical_origin_offset.to_num::<f32>(),
        left_extent: extents.left,
        right_extent: extents.right,
        bottom_extent: extents.bottom,
        top_extent: extents.top,
    }
}

fn approach(current: f32, target: f32, maximum_delta: f32) -> f32 {
    current + (target - current).clamp(-maximum_delta, maximum_delta)
}

fn debug_draw_camera(
    mut gizmos: Gizmos,
    fighters: Query<
        (
            &crate::fighter::motion::FighterTranslation,
            &crate::fighter::motion::FighterPreviousTranslation,
            &crate::fighter::Fighter,
            &crate::fighter::FighterFacingDirection,
            &FighterCameraExtents,
        ),
        With<crate::player::Player>,
    >,
    cameras: Query<(&MatchCamera, &StageCameraProfile)>,
    manifests: Res<Assets<FighterManifest>>,
    timing: Res<GgrsFrameTiming>,
    debug_settings: Res<DebugSettings>,
) {
    for (camera, profile) in &cameras {
        draw_rectangle(
            &mut gizmos,
            CameraFrameBounds {
                left: profile.left,
                right: profile.right,
                bottom: profile.bottom,
                top: profile.top,
            },
            Color::srgb(1.0, 0.5, 0.0),
        );
        draw_rectangle(&mut gizmos, camera.frame, Color::srgb(1.0, 1.0, 0.0));
        for (translation, previous_translation, fighter, _facing, extents) in &fighters {
            let manifest = manifests
                .get(&fighter.manifest)
                .expect("Fighter manifest should be valid");
            let center = sample_presentation_translation(
                *previous_translation,
                *translation,
                timing.overstep_fraction(),
                debug_settings.motion_sampling,
            ) + Vec2::Y * manifest.camera.vertical_origin_offset.to_num::<f32>();
            draw_rectangle(
                &mut gizmos,
                CameraFrameBounds {
                    left: center.x - extents.left,
                    right: center.x + extents.right,
                    bottom: center.y - extents.bottom,
                    top: center.y + extents.top,
                },
                Color::srgb(0.0, 1.0, 0.0),
            );
        }
        gizmos.circle(
            Vec3::new(camera.current.interest.x, camera.current.interest.y, 0.0),
            1.0,
            Color::srgb(0.0, 1.0, 1.0),
        );
    }
}

fn draw_rectangle(gizmos: &mut Gizmos, frame: CameraFrameBounds, color: Color) {
    let bottom_left = Vec3::new(frame.left, frame.bottom, 0.0);
    let bottom_right = Vec3::new(frame.right, frame.bottom, 0.0);
    let top_left = Vec3::new(frame.left, frame.top, 0.0);
    let top_right = Vec3::new(frame.right, frame.top, 0.0);
    gizmos.line(bottom_left, bottom_right, color);
    gizmos.line(bottom_right, top_right, color);
    gizmos.line(top_right, top_left, color);
    gizmos.line(top_left, bottom_left, color);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> StageCameraProfile {
        StageCameraProfile {
            left: -100.0,
            right: 100.0,
            bottom: -50.0,
            top: 100.0,
            origin: Vec2::new(0.0, 20.0),
            vertical_fov_degrees: 30.0,
            min_depth: 20.0,
            max_depth: 200.0,
            subject_scale: 1.0,
            fighter_forward_extent_scale: 1.0,
            tracking_smoothness: 3.0,
            vertical_pan_degrees: 0.0,
            horizontal_pan_degrees_per_unit: 0.0,
            vertical_pan_degrees_per_unit: 0.0,
            max_horizontal_pan_degrees: 17.5,
            max_upward_pan_degrees: 5.0,
            max_downward_pan_degrees: 7.0,
        }
    }

    fn fighter_profile() -> crate::fighter::FighterCameraProfile {
        crate::fighter::FighterCameraProfile {
            vertical_origin_offset: crate::math::int::FGi32::lit("1.5"),
            forward_extent: crate::math::int::FGi32::lit("4.0"),
            backward_extent: crate::math::int::FGi32::lit("2.0"),
            upward_extent: crate::math::int::FGi32::lit("3.0"),
            downward_extent: crate::math::int::FGi32::lit("1.0"),
            visibility_radius: crate::math::int::FGi32::lit("3.0"),
        }
    }

    #[test]
    fn fighter_camera_bounds_mirror_with_facing_direction() {
        let fighter_profile = fighter_profile();
        let stage_profile = profile();
        let translation = Vec2::new(10.0, 20.0);
        let mut right_extents = FighterCameraExtents::default();
        let right = fighter_subject_bounds(
            translation,
            fighter_profile,
            crate::fighter::FighterFacingDirection::Right,
            &stage_profile,
            &mut right_extents,
            1.0 / 60.0,
        );
        let mut left_extents = FighterCameraExtents::default();
        let left = fighter_subject_bounds(
            translation,
            fighter_profile,
            crate::fighter::FighterFacingDirection::Left,
            &stage_profile,
            &mut left_extents,
            1.0 / 60.0,
        );

        assert_eq!(right.center, Vec2::new(10.0, 21.5));
        assert_eq!(right.left_extent, 2.0);
        assert_eq!(right.right_extent, 4.0);
        assert_eq!(left.left_extent, 4.0);
        assert_eq!(left.right_extent, 2.0);
        assert_eq!(right.bottom_extent, 1.0);
        assert_eq!(right.top_extent, 3.0);
    }

    #[test]
    fn fighter_forward_extent_uses_the_stage_scale() {
        let mut stage_profile = profile();
        stage_profile.fighter_forward_extent_scale = 1.5;
        let mut extents = FighterCameraExtents::default();
        let bounds = fighter_subject_bounds(
            Vec2::ZERO,
            fighter_profile(),
            crate::fighter::FighterFacingDirection::Right,
            &stage_profile,
            &mut extents,
            1.0 / 60.0,
        );

        assert_eq!(bounds.left_extent, 2.0);
        assert_eq!(bounds.right_extent, 6.0);
    }

    #[test]
    fn fighter_extents_approach_facing_changes_at_half_a_unit_per_sixtieth() {
        let stage_profile = profile();
        let mut extents = FighterCameraExtents::default();
        fighter_subject_bounds(
            Vec2::ZERO,
            fighter_profile(),
            crate::fighter::FighterFacingDirection::Right,
            &stage_profile,
            &mut extents,
            1.0 / 60.0,
        );
        let bounds = fighter_subject_bounds(
            Vec2::ZERO,
            fighter_profile(),
            crate::fighter::FighterFacingDirection::Left,
            &stage_profile,
            &mut extents,
            1.0 / 60.0,
        );

        assert_eq!(bounds.left_extent, 2.5);
        assert_eq!(bounds.right_extent, 3.5);
    }

    #[test]
    fn empty_frame_is_centered_on_the_stage_origin() {
        let profile = profile();
        let frame = frame_subjects([], &profile, profile.min_depth);

        assert_eq!(frame.left, -40.0);
        assert_eq!(frame.right, 40.0);
        assert_eq!(frame.bottom, -30.0);
        assert_eq!(frame.top, 60.0);
    }

    #[test]
    fn subject_bounds_are_clamped_to_camera_limits() {
        let profile = profile();
        let frame = frame_subjects(
            [CameraSubjectBounds {
                center: Vec2::new(300.0, 300.0),
                left_extent: 20.0,
                right_extent: 20.0,
                bottom_extent: 20.0,
                top_extent: 20.0,
            }],
            &profile,
            profile.min_depth,
        );

        assert_eq!(frame.right, profile.right);
        assert_eq!(frame.top, profile.top);
        assert!(frame.left >= profile.left);
        assert!(frame.bottom >= profile.bottom);
    }

    #[test]
    fn four_subjects_use_the_base_scale() {
        let profile = profile();
        let subject = CameraSubjectBounds {
            center: Vec2::ZERO,
            left_extent: 10.0,
            right_extent: 10.0,
            bottom_extent: 10.0,
            top_extent: 10.0,
        };
        let one = frame_subjects([subject], &profile, profile.min_depth);
        let four = frame_subjects([subject; 4], &profile, profile.min_depth);

        assert_eq!(one.width(), 30.0);
        assert_eq!(four.width(), 20.0);
    }

    #[test]
    fn depth_grows_with_subject_separation_and_obeys_limits() {
        let profile = profile();
        let near = solve_pose(
            CameraFrameBounds {
                left: -1.0,
                right: 1.0,
                bottom: 19.0,
                top: 21.0,
            },
            &profile,
            16.0 / 9.0,
        );
        let far = solve_pose(
            CameraFrameBounds {
                left: -100.0,
                right: 100.0,
                bottom: -50.0,
                top: 100.0,
            },
            &profile,
            16.0 / 9.0,
        );

        assert_eq!(near.depth, profile.min_depth);
        assert!(far.depth > near.depth);
        assert_eq!(far.depth, profile.max_depth);
    }

    #[test]
    fn narrow_aspect_ratio_requires_more_depth_for_horizontal_spread() {
        let profile = profile();
        let frame = CameraFrameBounds {
            left: -50.0,
            right: 50.0,
            bottom: 19.0,
            top: 21.0,
        };

        let narrow = solve_pose(frame, &profile, 4.0 / 3.0);
        let wide = solve_pose(frame, &profile, 16.0 / 9.0);

        assert!(narrow.depth > wide.depth);
    }

    #[test]
    fn downward_expansion_matches_melees_global_curve() {
        assert_eq!(melee_downward_expansion(80.0), 10.0);
        assert_eq!(melee_downward_expansion(5000.0), 400.0);
        assert!((melee_downward_expansion(1000.0) - 82.92683).abs() < 0.0001);

        let profile = profile();
        let subject = CameraSubjectBounds {
            center: Vec2::new(0.0, 0.0),
            left_extent: 0.0,
            right_extent: 0.0,
            bottom_extent: 0.0,
            top_extent: 0.0,
        };
        let near = frame_subjects([subject], &profile, profile.min_depth);
        let far = frame_subjects([subject], &profile, profile.max_depth);

        assert!(far.bottom < near.bottom);
        assert!(far.bottom >= profile.bottom);
    }

    #[test]
    fn pan_coefficients_are_degrees_per_world_unit() {
        let mut profile = profile();
        profile.left = -1_000.0;
        profile.right = 1_000.0;
        profile.bottom = -1_000.0;
        profile.top = 1_000.0;
        profile.max_depth = 1_000.0;
        profile.horizontal_pan_degrees_per_unit = 0.1;
        let pose = solve_pose(
            CameraFrameBounds {
                left: 90.0,
                right: 110.0,
                bottom: 19.0,
                top: 21.0,
            },
            &profile,
            16.0 / 9.0,
        );

        let expected_offset = (pose.eye.x - pose.interest.x).abs();
        assert!(expected_offset > 5.0);
        assert!(expected_offset < 7.0);
    }

    #[test]
    fn solved_view_is_corrected_inside_camera_bounds() {
        let mut profile = profile();
        profile.max_depth = 1_000.0;
        profile.vertical_pan_degrees = -10.0;
        profile.horizontal_pan_degrees_per_unit = 0.1;
        profile.vertical_pan_degrees_per_unit = 0.1;
        let pose = solve_pose(
            CameraFrameBounds {
                left: 70.0,
                right: 95.0,
                bottom: 70.0,
                top: 95.0,
            },
            &profile,
            16.0 / 9.0,
        );

        for corner in view_corners_on_plane(pose, 16.0 / 9.0).unwrap() {
            assert!(corner.x >= profile.left - 0.001);
            assert!(corner.x <= profile.right + 0.001);
            assert!(corner.y >= profile.bottom - 0.001);
            assert!(corner.y <= profile.top + 0.001);
        }
    }

    #[test]
    fn frame_independent_smoothing_matches_melee_at_sixty_hz() {
        assert!((sixty_hz_lerp_alpha(0.09, 1.0 / 60.0) - 0.09).abs() < f32::EPSILON);
        assert!(sixty_hz_lerp_alpha(0.09, 1.0 / 30.0) > 0.09);
    }
}
