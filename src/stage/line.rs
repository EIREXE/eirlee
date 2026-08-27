//! Stage collision geometry.


use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::math::{segment::FGSegment2d, vec::FGVec2};

#[derive(Reflect, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum StagePolyLineSegmentType {
    Floor,
    Wall,
    Ceiling,
}

#[derive(Reflect, Clone)]
pub struct StagePolyLineSegment {
    pub segment: FGSegment2d,
    pub normal: FGVec2,
    pub segment_type: StagePolyLineSegmentType,
}

#[derive(Reflect, Clone, Default, Serialize, Deserialize, Copy)]
pub enum StagePolyType {
    #[default]
    Closed,
    Platform,
}

#[derive(Reflect, Hash, Debug, Clone, Copy, Default)]
pub struct StageLineID {
    pub polygon: usize,
    pub segment: usize,
}

#[derive(Reflect, Clone, Default)]
pub struct StagePoly {
    poly_type: StagePolyType,
    pub segments: Vec<StagePolyLineSegment>,
}

pub struct StagePlaneIntersectResult {
    pub position: FGVec2,
    pub segment_idx: usize,
    pub normal: FGVec2,
}

#[derive(Resource, Reflect, Clone, Default)]
#[reflect(Resource)]
pub struct StageCollision {
    pub stage_polys: Vec<StagePoly>,
}

impl StagePoly {
    pub fn intersect_ray(&self, ray_segment: FGSegment2d) -> Option<StagePlaneIntersectResult> {
        let ray_start = ray_segment.point1();
        let (segment_idx, segment, position) = self
            .segments
            .iter()
            .enumerate()
            .filter_map(|(segment_idx, segment)| {
                segment
                    .segment
                    .segment_intersection(&ray_segment)
                    .map(|position| (segment_idx, segment, position))
            })
            .min_by_key(|(_, _, position)| {
                let offset = *position - ray_start;
                let x = i128::from(offset.x.to_bits());
                let y = i128::from(offset.y.to_bits());
                x * x + y * y
            })?;

        Some(StagePlaneIntersectResult {
            position,
            segment_idx,
            normal: segment.normal,
        })
    }

    pub fn get_segment(&self, segment_idx: usize) -> Option<&StagePolyLineSegment> {
        self.segments.get(segment_idx)
    }

    pub fn get_next_segment(&self, segment_idx: usize) -> Option<&StagePolyLineSegment> {
        self.get_next_segment_index(segment_idx)
            .and_then(|index| self.segments.get(index))
    }

    pub fn get_prev_segment(&self, segment_idx: usize) -> Option<&StagePolyLineSegment> {
        self.get_prev_segment_index(segment_idx)
            .and_then(|index| self.segments.get(index))
    }

    pub fn get_next_segment_index(&self, segment_idx: usize) -> Option<usize> {
        if segment_idx >= self.segments.len() {
            return None;
        }

        let next = segment_idx + 1;
        if next < self.segments.len() {
            Some(next)
        } else if matches!(&self.poly_type, StagePolyType::Closed) {
            Some(0)
        } else {
            None
        }
    }

    pub fn get_prev_segment_index(&self, segment_idx: usize) -> Option<usize> {
        if segment_idx >= self.segments.len() {
            return None;
        }

        if segment_idx > 0 {
            Some(segment_idx - 1)
        } else if matches!(&self.poly_type, StagePolyType::Closed) {
            self.segments.len().checked_sub(1)
        } else {
            None
        }
    }

    pub fn build(poly_type: StagePolyType, points: &[(FGVec2, StagePolyLineSegmentType)]) -> Self {
        let segment_count = if points.len() < 2 {
            0
        } else if matches!(&poly_type, StagePolyType::Closed) {
            points.len()
        } else {
            points.len() - 1
        };
        let windows = points
            .iter()
            .zip(points.iter().cycle().skip(1))
            .take(segment_count);

        let segments = windows
            .map(|segment_window| {
                let (first_point, segment_type) = segment_window.0;
                let (second_point, _) = segment_window.1;
                let segment = FGSegment2d::new(*first_point, *second_point);
                StagePolyLineSegment {
                    segment_type: *segment_type,
                    segment,
                    normal: segment.left_normal(),
                }
            })
            .collect();

        Self {
            poly_type,
            segments,
        }
    }
}

impl StagePolyLineSegment {
    pub fn project(&self, vec: FGVec2) -> FGVec2 {
        vec - vec.project_onto_normalized(self.normal)
    }
}

#[cfg(test)]
mod tests {
    use crate::math::int::FGi32;

    use super::*;

    fn stage_segment(point1: FGVec2, point2: FGVec2) -> StagePolyLineSegment {
        let segment = FGSegment2d::new(point1, point2);
        StagePolyLineSegment {
            segment,
            normal: segment.left_normal(),
            segment_type: StagePolyLineSegmentType::Floor,
        }
    }

    #[test]
    fn platform_is_open_and_closed_poly_wraps() {
        let points = [
            (FGVec2::lit("0", "0"), StagePolyLineSegmentType::Floor),
            (FGVec2::lit("1", "0"), StagePolyLineSegmentType::Floor),
            (FGVec2::lit("2", "0"), StagePolyLineSegmentType::Floor),
        ];
        let platform = StagePoly::build(StagePolyType::Platform, &points);
        let closed = StagePoly::build(StagePolyType::Closed, &points);

        assert_eq!(platform.segments.len(), 2);
        assert_eq!(platform.get_next_segment_index(0), Some(1));
        assert_eq!(platform.get_next_segment_index(1), None);
        assert_eq!(platform.get_prev_segment_index(0), None);
        assert_eq!(platform.get_prev_segment_index(1), Some(0));

        assert_eq!(closed.segments.len(), 3);
        assert_eq!(closed.get_next_segment_index(2), Some(0));
        assert_eq!(closed.get_prev_segment_index(0), Some(2));
    }

    #[test]
    fn ray_intersection_returns_nearest_hit() {
        let stage_poly = StagePoly {
            poly_type: StagePolyType::Platform,
            segments: vec![
                stage_segment(FGVec2::lit("-1", "0"), FGVec2::lit("1", "0")),
                stage_segment(FGVec2::lit("-1", "1"), FGVec2::lit("1", "1")),
            ],
        };

        let hit = stage_poly
            .intersect_ray(FGSegment2d::new(
                FGVec2::lit("0", "2"),
                FGVec2::lit("0", "-1"),
            ))
            .unwrap();

        assert_eq!(hit.segment_idx, 1);
        assert_eq!(hit.position, FGVec2::lit("0", "1"));
    }

    #[test]
    fn ray_intersection_test() {
        let stage_poly = StagePoly::build(
            crate::stage::line::StagePolyType::Platform,
            &[
                (FGVec2::lit("-4.5", "0.0"), StagePolyLineSegmentType::Floor),
                (FGVec2::lit("4.5", "0.0"), StagePolyLineSegmentType::Floor),
            ],
        );

        let res = stage_poly.intersect_ray(FGSegment2d::new(
            FGVec2::lit("0.0", "1.0"),
            FGVec2::lit("0.0", "-1.0"),
        ));
        assert!(res.is_some());
        assert_eq!(res.unwrap().position, FGVec2::ZERO);
    }

    #[test]
    fn ray_intersection_test_side() {
        let stage_poly = StagePoly::build(
            crate::stage::line::StagePolyType::Platform,
            &[
                (FGVec2::lit("-4.5", "0.0"), StagePolyLineSegmentType::Floor),
                (FGVec2::lit("4.5", "0.0"), StagePolyLineSegmentType::Floor),
            ],
        );

        let res = stage_poly.intersect_ray(FGSegment2d::new(
            FGVec2::lit("2.0", "1.0"),
            FGVec2::lit("2.0", "-1.0"),
        ));
        assert!(res.is_some());
        assert!(res.unwrap().position.distance(FGVec2::lit("2.0", "0.0")) < FGi32::from_bits(4));
    }
    #[test]
    fn ray_intersection_test_diagonal() {
        let stage_poly = StagePoly::build(
            crate::stage::line::StagePolyType::Platform,
            &[
                (FGVec2::lit("0.0", "0.0"), StagePolyLineSegmentType::Floor),
                (FGVec2::lit("2.0", "2.0"), StagePolyLineSegmentType::Floor),
            ],
        );

        let res = stage_poly.intersect_ray(FGSegment2d::new(
            FGVec2::lit("1.0", "2.0"),
            FGVec2::lit("1.0", "-2.0"),
        ));
        assert!(res.is_some());
        assert_eq!(res.unwrap().position, FGVec2::lit("1.0", "1.0"));
    }
}
