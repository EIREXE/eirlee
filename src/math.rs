use bevy::math::{Vec2, primitives::Segment2d};

pub trait SegmentIntersection {
    fn segment_intersection(&self, rhs: &Segment2d) -> Option<Vec2>;
}

impl SegmentIntersection for Segment2d {
    fn segment_intersection(&self, rhs: &Segment2d) -> Option<Vec2> {
        let p0 = self.point1();
        let p1 = self.point2();
        let p2 = rhs.point1();
        let p3 = rhs.point2();
        
        let s1_x = p1.x - p0.x;
        let s1_y = p1.y - p0.y;
        let s2_x = p3.x - p2.x;
        let s2_y = p3.y - p2.y;

        let denom = -s2_x * s1_y + s1_x * s2_y;
        if denom.abs() < f32::EPSILON {
            return None; // Parallel or collinear
        }

        let s = (-s1_y * (p0.x - p2.x) + s1_x * (p0.y - p2.y)) / denom;
        let t = ( s2_x * (p0.y - p2.y) - s2_y * (p0.x - p2.x)) / denom;

        if s >= 0.0 && s <= 1.0 && t >= 0.0 && t <= 1.0 {
            Some(Vec2::new(p0.x + (t * s1_x), p0.y + (t * s1_y)))
        } else {
            None
        }
    }
}