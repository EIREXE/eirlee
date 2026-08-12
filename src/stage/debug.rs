use bevy::{color::palettes::css::RED, prelude::*};

use crate::stage::StageLine;

pub fn debug_draw_scene(mut gizmos: Gizmos, planes: Query<&StageLine>) {
    for plane in planes {
        let from = plane.segment.point1();
        let to = plane.segment.point2();
        let from_3d = Vec3::new(from.x, from.y, 0.0);
        let end_3d = Vec3::new(to.x, to.y, 0.0);
        let normal_3d = Vec3::new(plane.normal.x, plane.normal.y, 0.0);
        gizmos.line(from_3d, end_3d, RED);
        let midpoint = (from_3d + end_3d) * 0.5;
        gizmos.arrow(midpoint, midpoint + normal_3d, RED);
    }
}
