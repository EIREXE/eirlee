//! Stage collision geometry.

use bevy::{math::InvalidDirectionError, prelude::*};

use crate::math::SegmentIntersection;

#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component)]
pub struct StageLine {
    pub segment: Segment2d,
    pub normal: Vec2,
}

pub struct StagePlaneIntersectResult {
    pub position: Vec2,
}

impl StageLine {
    pub fn new(start: Vec2, end: Vec2) -> Result<Self, InvalidDirectionError> {
        let segment = Segment2d::new(start, end);
        let normal = segment.left_normal().as_vec2();
        Ok(Self { segment, normal })
    }

    pub fn intersect_ray(&self, segment: Segment2d) -> Option<StagePlaneIntersectResult> {
        self.segment
            .segment_intersection(&segment)
            .map(|position| StagePlaneIntersectResult { position })
    }

    pub fn project(&self, vec: Vec2) -> Vec2 {
        vec - vec.project_onto_normalized(self.normal)
    }
}
