use bevy::{color::palettes::css::RED, prelude::*};

use crate::stage::{StagePoly, line::StageCollision};

pub fn debug_draw_scene(mut gizmos: Gizmos, stage_collision: Res<StageCollision>) {
    for poly in stage_collision.stage_polys.iter() {
        for segment in poly.segments.iter() {
            let from = segment.segment.point1();
            let to = segment.segment.point2();
            let from_3d = Vec3::new(from.x, from.y, 0.0);
            let end_3d = Vec3::new(to.x, to.y, 0.0);
            let normal_3d = Vec3::new(segment.normal.x, segment.normal.y, 0.0);
            gizmos.line(from_3d, end_3d, RED);
            let midpoint = (from_3d + end_3d) * 0.5;
            gizmos.arrow(midpoint, midpoint + normal_3d, RED);
        }

    }
}
