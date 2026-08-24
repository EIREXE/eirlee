use bevy::math::{Vec2, primitives::Segment2d};

use bevy::prelude::*;
use fixed::prelude::*;

use crate::math::vec::FGVec2;

type FGi32 = fixed::FixedI32<fixed::types::extra::U16>;

#[derive(Clone, Copy, PartialEq, Reflect)]
pub struct FGSegment2d {
    point1: FGVec2,
    point2: FGVec2
}

impl FGSegment2d {
    #[inline]
    pub fn new(point1: FGVec2, point2: FGVec2) -> Self {
        Self {
            point1, point2
        }
    }

    #[inline]
    pub fn point1(&self) -> FGVec2 {
        self.point1
    }

    #[inline]
    pub fn point2(&self) -> FGVec2 {
        self.point2
    }

    /// Compute the normalized direction pointing from the first endpoint to the second endpoint.
    ///
    /// For the non-panicking version, see [`FGVec2::try_direction`].
    ///
    /// # Panics
    ///
    /// Panics if a valid direction could not be computed, for example when the endpoints are coincident, NaN, or infinite.
    #[inline]
    pub fn direction(&self) -> FGVec2 {
        self.try_direction().expect("Failed to compute the direction of a line segment")
    }

    /// Try to compute the normalized direction pointing from the first endpoint to the second endpoint.
    ///
    /// Returns [`Err(InvalidDirectionError)`](InvalidDirectionError) if a valid direction could not be computed,
    /// for example when the endpoints are coincident, NaN, or infinite.
    #[inline]
    pub fn try_direction(&self) -> Option<FGVec2> {
        self.point1.direction_to(self.point2)
    }

    /// Returns the point on the [`Segment2d`] that is closest to the specified `point`.
    #[inline]
    pub fn closest_point(&self, point: FGVec2) -> FGVec2 {
        //       `point`
        //           x
        //          ^|
        //         / |
        //`offset`/  |
        //       /   |  `segment_vector`
        //      x----.-------------->x
        //      0    t               1
        let segment_vector = self.point2 - self.point1;
        let offset = point - self.point1;
        // The signed projection of `offset` onto `segment_vector`, scaled by the length of the segment.
        let projection_scaled = segment_vector.dot(offset);

        // `point` is too far "left" in the picture
        if projection_scaled <= 0.0 {
            return self.point1;
        }

        let length_squared = segment_vector.length_squared();
        // `point` is too far "right" in the picture
        if projection_scaled >= length_squared {
            return self.point2;
        }

        // Point lies somewhere in the middle, we compute the closest point by finding the parameter along the line.
        let t = projection_scaled / length_squared;
        self.point1 + segment_vector * t
    }

    /// Compute the normalized counterclockwise normal on the left-hand side of the line segment.
    ///
    /// For the non-panicking version, see [`Segment2d::try_left_normal`].
    ///
    /// # Panics
    ///
    /// Panics if a valid normal could not be computed, for example when the endpoints are coincident
    pub fn left_normal(&self) -> FGVec2 {
        self.point1.direction_to(self.point2).map(|o| { FGVec2::new(-o.y, o.x)}).expect("a valid normal could not be computed")
    }

    pub fn segment_intersection(&self, rhs: &FGSegment2d) -> Option<FGVec2> {
        let p0 = self.point1();
        let p1 = self.point2();
        let p2 = rhs.point1();
        let p3 = rhs.point2();
        
        let s1_x = p1.x - p0.x;
        let s1_y = p1.y - p0.y;
        let s2_x = p3.x - p2.x;
        let s2_y = p3.y - p2.y;

        let denom = -s2_x * s1_y + s1_x * s2_y;
        if denom == FGi32::ZERO {
            return None; // Parallel or collinear
        }

        let s = (-s1_y * (p0.x - p2.x) + s1_x * (p0.y - p2.y)) / denom;
        let t = ( s2_x * (p0.y - p2.y) - s2_y * (p0.x - p2.x)) / denom;

        if s >= FGi32::ZERO && s <= FGi32::ONE && t >= FGi32::ZERO && t <= FGi32::ONE {
            Some(FGVec2::new(p0.x + (t * s1_x), p0.y + (t * s1_y)))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closest_point_projects_and_clamps_to_segment() {
        let segment = FGSegment2d::new(FGVec2::lit("-4", "0"), FGVec2::lit("4", "0"));

        assert_eq!(segment.closest_point(FGVec2::lit("2", "1")), FGVec2::lit("2", "0"));
        assert_eq!(segment.closest_point(FGVec2::lit("-6", "1")), segment.point1());
        assert_eq!(segment.closest_point(FGVec2::lit("6", "1")), segment.point2());
    }
}
