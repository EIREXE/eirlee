//! Stage collision geometry.

use std::default;

use bevy::{math::InvalidDirectionError, prelude::*};

use crate::math::segment::SegmentIntersection;

#[derive(Reflect, Clone, Copy, PartialEq)]
pub enum StagePolyLineSegmentType {
    Floor, Wall, Ceiling
}


#[derive(Reflect, Clone)]
pub struct StagePolyLineSegment {
    pub segment: Segment2d,
    pub normal: Vec2,
    pub segment_type: StagePolyLineSegmentType
}

#[derive(Reflect, Clone, Default)]
pub enum StagePolyType {
    #[default]
    Closed,
    Platform
}

#[derive(Reflect, Hash, Debug, Clone, Copy, Default)]
pub struct StageLineID {
    pub polygon: usize,
    pub segment: usize
}

#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component)]
pub struct StagePoly {
    poly_type: StagePolyType,
    pub segments: Vec<StagePolyLineSegment>
}

pub struct StagePlaneIntersectResult {
    pub position: Vec2,
    pub segment_idx: usize,
    pub normal: Vec2
}

#[derive(Resource, Reflect, Clone, Default)]
#[reflect(Resource)]
pub struct StageCollision {
    pub stage_polys: Vec<StagePoly>
}

impl StagePoly {
    pub fn intersect_ray(&self, ray_segment: Segment2d) -> Option<StagePlaneIntersectResult> {
        for (i, segment) in self.segments.iter().enumerate() {
            if let Some(intersection) = segment.segment.segment_intersection(&ray_segment) {
                println!("DIST3 {} {}", intersection, segment.segment.closest_point(intersection));
                return Some(StagePlaneIntersectResult {
                    position: intersection,
                    segment_idx: i,
                    normal: segment.normal
                });
            }
        }
        None
    }

    pub fn get_segment(&self, segment_idx: usize) -> Option<&StagePolyLineSegment> {
        self.segments.get(segment_idx)
    }

    pub fn get_next_segment(&self, segment_idx: usize) -> Option<&StagePolyLineSegment> {
        self.segments.get(segment_idx % segment_idx + 1)
    }

    pub fn get_prev_segment(&self, segment_idx: usize) -> Option<&StagePolyLineSegment> {
        self.segments.get(segment_idx % segment_idx + 1)
    }

    pub fn build(poly_type: StagePolyType, points: &[(Vec2, StagePolyLineSegmentType)]) -> Self {
        let windows = points.iter().zip(points.iter().cycle().skip(1));

        let segments = windows.map(|segment_window| {
            let (first_point, segment_type) = segment_window.0;
            let (second_point, _) = segment_window.1;
            let segment = Segment2d::new(*first_point, *second_point);
            StagePolyLineSegment {
                segment_type: *segment_type,
                segment,
                normal: segment.left_normal().as_vec2()
            }
        }).collect();

        Self {
            poly_type,
            segments
        }
    }
}

impl StagePolyLineSegment {
    pub fn project(&self, vec: Vec2) -> Vec2 {
        vec - vec.project_onto_normalized(self.normal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ray_intersection_test() {
        let stage_poly = StagePoly::build(crate::stage::line::StagePolyType::Platform, &[
            (Vec2::new(-4.5, 0.0), StagePolyLineSegmentType::Floor),
            (Vec2::new(4.5, 0.0), StagePolyLineSegmentType::Floor),
        ]);

        let res = stage_poly.intersect_ray(Segment2d::new(Vec2::new(0.0, 1.0), Vec2::new(0.0, -1.0)));
        assert!(res.is_some());
        assert_eq!(res.unwrap().position, Vec2::ZERO);
    }

    #[test]
    fn ray_intersection_test_side() {
        let stage_poly = StagePoly::build(crate::stage::line::StagePolyType::Platform, &[
            (Vec2::new(-4.5, 0.0), StagePolyLineSegmentType::Floor),
            (Vec2::new(4.5, 0.0), StagePolyLineSegmentType::Floor),
        ]);

        let res = stage_poly.intersect_ray(Segment2d::new(Vec2::new(2.0, 1.0), Vec2::new(2.0, -1.0)));
        assert!(res.is_some());
        assert_eq!(res.unwrap().position, Vec2::new(2.0, 0.0));
    }
    #[test]
    fn ray_intersection_test_diagonal() {
        let stage_poly = StagePoly::build(crate::stage::line::StagePolyType::Platform, &[
            (Vec2::new(0.0, 0.0), StagePolyLineSegmentType::Floor),
            (Vec2::new(2.0, 2.0), StagePolyLineSegmentType::Floor),
        ]);

        let res = stage_poly.intersect_ray(Segment2d::new(Vec2::new(1.0, 2.0), Vec2::new(1.0, -2.0)));
        assert!(res.is_some());
        assert_eq!(res.unwrap().position, Vec2::new(1.0, 1.0));
    }
}