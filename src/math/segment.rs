use bevy::prelude::*;

use crate::math::{
    int::{FGWide, FGi32},
    vec::FGVec2,
};

#[derive(Clone, Copy, PartialEq, Reflect)]
pub struct FGSegment2d {
    point1: FGVec2,
    point2: FGVec2,
}

impl FGSegment2d {
    #[inline]
    pub fn new(point1: FGVec2, point2: FGVec2) -> Self {
        Self { point1, point2 }
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
        self.try_direction()
            .expect("Failed to compute the direction of a line segment")
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
        FGVec2::new(
            FGi32::from_num(
                FGWide::from_num(self.point1.x) + FGWide::from_num(segment_vector.x) * t,
            ),
            FGi32::from_num(
                FGWide::from_num(self.point1.y) + FGWide::from_num(segment_vector.y) * t,
            ),
        )
    }

    /// Compute the normalized counterclockwise normal on the left-hand side of the line segment.
    ///
    /// For the non-panicking version, see [`Segment2d::try_left_normal`].
    ///
    /// # Panics
    ///
    /// Panics if a valid normal could not be computed, for example when the endpoints are coincident
    pub fn left_normal(&self) -> FGVec2 {
        self.point1
            .direction_to(self.point2)
            .map(|o| FGVec2::new(-o.y, o.x))
            .expect("a valid normal could not be computed")
    }

    pub fn segment_intersection(&self, rhs: &FGSegment2d) -> Option<FGVec2> {
        let p0 = self.point1();
        let p1 = self.point2();
        let p2 = rhs.point1();
        let p3 = rhs.point2();

        let p0_x = FGWide::from_num(p0.x);
        let p0_y = FGWide::from_num(p0.y);
        let p1_x = FGWide::from_num(p1.x);
        let p1_y = FGWide::from_num(p1.y);
        let p2_x = FGWide::from_num(p2.x);
        let p2_y = FGWide::from_num(p2.y);
        let p3_x = FGWide::from_num(p3.x);
        let p3_y = FGWide::from_num(p3.y);

        let s1_x = p1_x - p0_x;
        let s1_y = p1_y - p0_y;
        let s2_x = p3_x - p2_x;
        let s2_y = p3_y - p2_y;

        let denom = -s2_x * s1_y + s1_x * s2_y;
        if denom == FGWide::ZERO {
            return None; // Parallel or collinear
        }
        let s = (-s1_y * (p0_x - p2_x) + s1_x * (p0_y - p2_y)) / denom;
        let t = (s2_x * (p0_y - p2_y) - s2_y * (p0_x - p2_x)) / denom;

        if s >= FGWide::ZERO && s <= FGWide::ONE && t >= FGWide::ZERO && t <= FGWide::ONE {
            if self.left_normal().dot(rhs.point2() - rhs.point1()) >= FGi32::ZERO {
                return None;
            }
            Some(FGVec2::new(
                FGi32::from_num(p0_x + t * s1_x),
                FGi32::from_num(p0_y + t * s1_y),
            ))
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

        assert_eq!(
            segment.closest_point(FGVec2::lit("2", "1")),
            FGVec2::lit("2", "0")
        );
        assert_eq!(
            segment.closest_point(FGVec2::lit("-6", "1")),
            segment.point1()
        );
        assert_eq!(
            segment.closest_point(FGVec2::lit("6", "1")),
            segment.point2()
        );
    }

    #[test]
    fn segment_intersection_avoids_cross_product_overflow() {
        let segment = FGSegment2d::new(FGVec2::lit("56", "-3.5"), FGVec2::lit("56", "-200"));
        let rhs = FGSegment2d::new(
            FGVec2::lit("-111.26334", "-130.22586"),
            FGVec2::lit("-112.36334", "-133.02586"),
        );

        assert_eq!(segment.segment_intersection(&rhs), None);
    }

    #[test]
    fn segment_intersection_handles_large_crossing_segments() {
        let horizontal = FGSegment2d::new(FGVec2::lit("-200", "0"), FGVec2::lit("200", "0"));
        let vertical = FGSegment2d::new(FGVec2::lit("0", "-200"), FGVec2::lit("0", "200"));

        assert_eq!(
            horizontal.segment_intersection(&vertical),
            Some(FGVec2::ZERO)
        );
    }
}
