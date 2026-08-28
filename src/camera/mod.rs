//! Presentation-only state and tuning for the match camera.

use bevy::prelude::*;

use crate::stage::manifest::StageCameraProfile;

const EMPTY_FRAME_HALF_SIZE: f32 = 40.0;
const SUBJECT_COUNT_SCALES: [f32; 4] = [1.5, 1.32, 1.16, 1.0];

/// The rectangle contributed by one tracked subject on the gameplay plane.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CameraSubjectBounds {
    pub center: Vec2,
    pub left_extent: f32,
    pub right_extent: f32,
    pub bottom_extent: f32,
    pub top_extent: f32,
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
            (update_match_camera, debug_draw_camera).run_if(in_state(crate::AppState::InMatch)),
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

    let depth_fraction = ((current_depth - profile.min_depth)
        / (profile.max_depth - profile.min_depth).max(f32::EPSILON))
    .clamp(0.0, 1.0);
    frame.bottom =
        (frame.bottom - profile.max_downward_expansion * depth_fraction).max(profile.bottom);
    frame
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
    let vertical_angle =
        (-(vertical_center - profile.origin.y) * profile.vertical_pan_coefficient).clamp(
            -profile.max_downward_pan_degrees.to_radians(),
            profile.max_upward_pan_degrees.to_radians(),
        ) + profile.vertical_tilt_degrees.to_radians();
    let up_angle = vertical_half_fov + vertical_angle;
    let down_angle = vertical_half_fov - vertical_angle;
    let vertical_depth = frame.height() / (up_angle.tan() + down_angle.tan()).max(f32::EPSILON);
    let vertical_eye_offset = vertical_depth * vertical_angle.tan();
    let interest_y = vertical_eye_offset + frame.top - vertical_depth * up_angle.tan();

    let horizontal_half_fov = (vertical_half_fov.tan() * aspect_ratio).atan();
    let horizontal_center = (frame.left + frame.right) * 0.5;
    let horizontal_angle =
        (-(horizontal_center - profile.origin.x) * profile.horizontal_pan_coefficient).clamp(
            -profile.max_horizontal_pan_degrees.to_radians(),
            profile.max_horizontal_pan_degrees.to_radians(),
        );
    let right_angle = horizontal_half_fov - horizontal_angle;
    let left_angle = horizontal_half_fov + horizontal_angle;
    let horizontal_depth = frame.width() / (right_angle.tan() + left_angle.tan()).max(f32::EPSILON);
    let horizontal_eye_offset = horizontal_depth * horizontal_angle.tan();
    let interest_x = frame.right - horizontal_depth * right_angle.tan() - horizontal_eye_offset;

    let depth = horizontal_depth
        .max(vertical_depth)
        .clamp(profile.min_depth, profile.max_depth);
    let interest = Vec2::new(interest_x, interest_y).clamp(
        Vec2::new(profile.left, profile.bottom),
        Vec2::new(profile.right, profile.top),
    );

    CameraPose {
        eye: interest + Vec2::new(horizontal_eye_offset, -vertical_eye_offset),
        interest,
        depth,
        fov: profile.vertical_fov_degrees.to_radians(),
    }
}

fn update_match_camera(
    time: Res<Time>,
    fighters: Query<
        (
            &crate::fighter::motion::FighterTranslation,
            &crate::fighter::FighterCameraProfile,
            &crate::fighter::FighterFacingDirection,
        ),
        With<crate::player::Player>,
    >,
    mut cameras: Query<(
        &mut Transform,
        &mut Projection,
        &mut MatchCamera,
        &StageCameraProfile,
    )>,
) {
    let subjects = fighters.iter().map(|(translation, profile, facing)| {
        fighter_subject_bounds(
            Vec2::new(translation.x.to_num(), translation.y.to_num()),
            *profile,
            *facing,
        )
    });

    for (mut transform, mut projection, mut camera, profile) in &mut cameras {
        let Projection::Perspective(projection) = projection.as_mut() else {
            continue;
        };
        camera.frame = frame_subjects(subjects.clone(), profile, camera.current.depth);
        camera.target = solve_pose(camera.frame, profile, projection.aspect_ratio);

        if camera.needs_snap {
            camera.current = camera.target;
            camera.needs_snap = false;
        } else {
            let spread = camera.frame.width().max(camera.frame.height());
            let interest_rate =
                profile.tracking_smoothness * (1.0 + ((spread - 120.0) / 780.0).clamp(0.0, 1.0));
            let interest_alpha = 1.0 - (-interest_rate * time.delta_secs()).exp();
            let eye_alpha = 1.0 - (-(profile.tracking_smoothness * 3.0) * time.delta_secs()).exp();
            camera.current.interest = camera
                .current
                .interest
                .lerp(camera.target.interest, interest_alpha);
            camera.current.eye = camera.current.eye.lerp(camera.target.eye, eye_alpha);
            camera.current.depth += (camera.target.depth - camera.current.depth) * eye_alpha;
            camera.current.fov += (camera.target.fov - camera.current.fov) * interest_alpha;
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

fn fighter_subject_bounds(
    translation: Vec2,
    profile: crate::fighter::FighterCameraProfile,
    facing: crate::fighter::FighterFacingDirection,
) -> CameraSubjectBounds {
    let (left_extent, right_extent) = match facing {
        crate::fighter::FighterFacingDirection::Left => (
            profile.forward_extent.to_num(),
            profile.backward_extent.to_num(),
        ),
        crate::fighter::FighterFacingDirection::Right => (
            profile.backward_extent.to_num(),
            profile.forward_extent.to_num(),
        ),
    };

    CameraSubjectBounds {
        center: translation + Vec2::Y * profile.vertical_origin_offset.to_num::<f32>(),
        left_extent,
        right_extent,
        bottom_extent: profile.downward_extent.to_num(),
        top_extent: profile.upward_extent.to_num(),
    }
}

fn debug_draw_camera(
    mut gizmos: Gizmos,
    fighters: Query<
        (
            &crate::fighter::motion::FighterTranslation,
            &crate::fighter::FighterCameraProfile,
            &crate::fighter::FighterFacingDirection,
        ),
        With<crate::player::Player>,
    >,
    cameras: Query<(&MatchCamera, &StageCameraProfile)>,
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
        for (translation, fighter_profile, facing) in &fighters {
            let subject = fighter_subject_bounds(
                Vec2::new(translation.x.to_num(), translation.y.to_num()),
                *fighter_profile,
                *facing,
            );
            draw_rectangle(
                &mut gizmos,
                CameraFrameBounds {
                    left: subject.center.x - subject.left_extent,
                    right: subject.center.x + subject.right_extent,
                    bottom: subject.center.y - subject.bottom_extent,
                    top: subject.center.y + subject.top_extent,
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
            tracking_smoothness: 3.0,
            vertical_tilt_degrees: 0.0,
            horizontal_pan_coefficient: 0.0,
            vertical_pan_coefficient: 0.0,
            max_horizontal_pan_degrees: 17.5,
            max_upward_pan_degrees: 5.0,
            max_downward_pan_degrees: 7.0,
            max_downward_expansion: 40.0,
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
        let profile = fighter_profile();
        let translation = Vec2::new(10.0, 20.0);
        let right = fighter_subject_bounds(
            translation,
            profile,
            crate::fighter::FighterFacingDirection::Right,
        );
        let left = fighter_subject_bounds(
            translation,
            profile,
            crate::fighter::FighterFacingDirection::Left,
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
    fn empty_frame_is_centered_on_the_stage_origin() {
        let profile = profile();
        let frame = frame_subjects([], &profile, profile.min_depth);

        assert_eq!(frame.left, -40.0);
        assert_eq!(frame.right, 40.0);
        assert_eq!(frame.bottom, -20.0);
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
    fn downward_expansion_increases_with_depth_without_crossing_stage_bottom() {
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
}
