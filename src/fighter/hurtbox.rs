//! Fixed-point convex geometry used by fighter hurtboxes.

use crate::{
    fighter::baked_animation::FixedMat4,
    math::{
        int::{FGWide, FGi32},
        vec3::FGVec3,
    },
};

/// A capsule after an arbitrary affine transform. Non-uniform scale turns the
/// spherical caps and circular cylinder into ellipsoidal geometry.
#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub struct FixedAffineCapsule {
    pub transform: FixedMat4,
    pub half_length: FGi32,
    pub radius: FGi32,
}

/// An ordinary world-space capsule, used for swept attack volumes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FixedCapsule {
    pub start: FGVec3,
    pub end: FGVec3,
    pub radius: FGi32,
}

impl FixedCapsule {
    pub fn new(start: FGVec3, end: FGVec3, radius: FGi32) -> Self {
        Self { start, end, radius }
    }
}

impl FixedAffineCapsule {
    pub fn new(transform: FixedMat4, half_length: FGi32, radius: FGi32) -> Self {
        Self {
            transform,
            half_length,
            radius,
        }
    }

    pub fn transform_point(self, point: FGVec3) -> FGVec3 {
        self.transform.transform_point(point)
    }

    /// Tests the exact affine hurtbox against an ordinary world-space capsule.
    pub fn intersects(self, rhs: FixedCapsule) -> bool {
        let mut direction = rhs.start - self.transform.get_translation();
        if direction == FGVec3::ZERO {
            direction = FGVec3::new(FGi32::ONE, FGi32::ZERO, FGi32::ZERO);
        }
        let mut simplex = [FGVec3::ZERO; 4];
        let mut simplex_len = 0;
        for _ in 0..20 {
            let point = self.support(direction) - rhs.support(-direction);
            if point.dot(direction) < FGWide::ZERO {
                return false;
            }
            for index in (1..=simplex_len.min(3)).rev() {
                simplex[index] = simplex[index - 1];
            }
            simplex[0] = point;
            simplex_len += 1;
            if update_simplex(&mut simplex, &mut simplex_len, &mut direction) {
                return true;
            }
            if direction == FGVec3::ZERO {
                return true;
            }
        }
        // A capped GJK iteration means the origin remains enclosed by the
        // current simplex to fixed-point precision, so conservatively collide.
        true
    }

}

trait SupportMap {
    fn support(self, direction: FGVec3) -> FGVec3;
}

impl SupportMap for FixedAffineCapsule {
    fn support(self, direction: FGVec3) -> FGVec3 {
        let local_direction = FGVec3::new(
            dot_column(self.transform, 0, direction),
            dot_column(self.transform, 1, direction),
            dot_column(self.transform, 2, direction),
        );
        let round = local_direction.normalized().unwrap_or(FGVec3::new(
            FGi32::ONE,
            FGi32::ZERO,
            FGi32::ZERO,
        )) * self.radius;
        let endpoint = if local_direction.y >= FGi32::ZERO {
            self.half_length
        } else {
            -self.half_length
        };
        self.transform_point(round + FGVec3::new(FGi32::ZERO, endpoint, FGi32::ZERO))
    }
}

impl SupportMap for FixedCapsule {
    fn support(self, direction: FGVec3) -> FGVec3 {
        let endpoint = if self.start.dot(direction) >= self.end.dot(direction) {
            self.start
        } else {
            self.end
        };
        endpoint
            + direction
                .normalized()
                .unwrap_or(FGVec3::new(FGi32::ONE, FGi32::ZERO, FGi32::ZERO))
                * self.radius
    }
}

fn dot_column(matrix: FixedMat4, column: usize, vector: FGVec3) -> FGi32 {
    FGi32::from_num(
        FGWide::from_num(matrix.cols[column][0]) * FGWide::from_num(vector.x)
            + FGWide::from_num(matrix.cols[column][1]) * FGWide::from_num(vector.y)
            + FGWide::from_num(matrix.cols[column][2]) * FGWide::from_num(vector.z),
    )
}

fn cross(lhs: FGVec3, rhs: FGVec3) -> FGVec3 {
    FGVec3::new(
        FGi32::from_num(
            FGWide::from_num(lhs.y) * FGWide::from_num(rhs.z)
                - FGWide::from_num(lhs.z) * FGWide::from_num(rhs.y),
        ),
        FGi32::from_num(
            FGWide::from_num(lhs.z) * FGWide::from_num(rhs.x)
                - FGWide::from_num(lhs.x) * FGWide::from_num(rhs.z),
        ),
        FGi32::from_num(
            FGWide::from_num(lhs.x) * FGWide::from_num(rhs.y)
                - FGWide::from_num(lhs.y) * FGWide::from_num(rhs.x),
        ),
    )
}

fn triple_cross(lhs: FGVec3, middle: FGVec3, rhs: FGVec3) -> FGVec3 {
    cross(cross(lhs, middle), rhs)
}

fn update_simplex(
    simplex: &mut [FGVec3; 4],
    simplex_len: &mut usize,
    direction: &mut FGVec3,
) -> bool {
    let a = simplex[0];
    let ao = -a;
    match *simplex_len {
        2 => {
            let ab = simplex[1] - a;
            if ab.dot(ao) > FGWide::ZERO {
                *direction = triple_cross(ab, ao, ab);
            } else {
                *simplex_len = 1;
                *direction = ao;
            }
            false
        }
        3 => {
            let b = simplex[1];
            let c = simplex[2];
            let ab = b - a;
            let ac = c - a;
            let abc = cross(ab, ac);
            if cross(abc, ac).dot(ao) > FGWide::ZERO {
                if ac.dot(ao) > FGWide::ZERO {
                    simplex[1] = c;
                    *simplex_len = 2;
                    *direction = triple_cross(ac, ao, ac);
                } else {
                    *simplex_len = 2;
                    *direction = triple_cross(ab, ao, ab);
                }
            } else if cross(ab, abc).dot(ao) > FGWide::ZERO {
                *simplex_len = 2;
                *direction = triple_cross(ab, ao, ab);
            } else if abc.dot(ao) > FGWide::ZERO {
                *direction = abc;
            } else {
                simplex.swap(1, 2);
                *direction = -abc;
            }
            false
        }
        4 => {
            let b = simplex[1];
            let c = simplex[2];
            let d = simplex[3];
            for (first, second, opposite) in [(b, c, d), (c, d, b), (d, b, c)] {
                let mut normal = cross(first - a, second - a);
                if normal.dot(opposite - a) > FGWide::ZERO {
                    normal = -normal;
                }
                if normal.dot(ao) > FGWide::ZERO {
                    simplex[0] = a;
                    simplex[1] = first;
                    simplex[2] = second;
                    *simplex_len = 3;
                    *direction = normal;
                    return false;
                }
            }
            true
        }
        _ => {
            *direction = ao;
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn affine_capsule(x: &str, scale_y: &str) -> FixedAffineCapsule {
        let mut transform = FixedMat4::IDENTITY;
        transform.cols[1][1] = FGi32::lit(scale_y);
        transform.translate(FGVec3::lit(x, "0", "0"));
        FixedAffineCapsule::new(transform, FGi32::lit("1"), FGi32::lit("1"))
    }

    fn capsule(x: &str) -> FixedCapsule {
        FixedCapsule::new(
            FGVec3::lit(x, "-1", "0"),
            FGVec3::lit(x, "1", "0"),
            FGi32::lit("1"),
        )
    }

    #[test]
    fn detects_touching_and_separated_capsules() {
        assert!(affine_capsule("0", "1").intersects(capsule("2")));
        assert!(!affine_capsule("0", "1").intersects(capsule("2.01")));
    }

    #[test]
    fn affine_scale_changes_capsule_intersection() {
        let sphere = FixedAffineCapsule::new(
            FixedMat4::IDENTITY,
            FGi32::ZERO,
            FGi32::lit("1"),
        );
        let mut squashed_transform = FixedMat4::IDENTITY;
        squashed_transform.cols[1][1] = FGi32::lit("0.5");
        let squashed = FixedAffineCapsule::new(
            squashed_transform,
            FGi32::ZERO,
            FGi32::lit("1"),
        );
        let target = FixedCapsule::new(
            FGVec3::lit("0", "1.4", "0"),
            FGVec3::lit("0", "1.4", "0"),
            FGi32::lit("0.5"),
        );

        assert!(sphere.intersects(target));
        assert!(!squashed.intersects(target));
    }
}
