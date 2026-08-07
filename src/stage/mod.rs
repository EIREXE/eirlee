use bevy::{color::palettes::css::{GREEN, RED}, prelude::*};

#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component)]
pub struct StagePlane {
    pub plane: Plane2d,
    pub position: Vec2,
    half_size: f32,
    basis_x: Vec2,
}

pub struct StagePlaneIntersectResult {
    pub position: Vec2
}

impl StagePlane {
    pub fn new(normal: Dir2, half_size: f32, position: Vec2) -> Self {
        let orthogonal = normal.perp();
        let basis_x = orthogonal * half_size;

        Self {
            plane: Plane2d::new(*normal),
            position,
            half_size,
            basis_x,
        }
    }
    pub fn intersect_ray(&self, ray: Ray2d) -> Option<StagePlaneIntersectResult> {
        if let Some(result) = ray.plane_intersection_point(self.position, self.plane) {
            return Some(StagePlaneIntersectResult {
                position: result
            });
        }
        None
    }

    pub fn project(&self, vec: Vec2) -> Vec2 {
        vec - vec.project_onto_normalized(self.plane.normal.as_vec2())
    }
}

pub fn debug_draw_scene(mut gizmos: Gizmos, planes: Query<&StagePlane>) {
    for plane in planes {
        let plane_pos_3d = Vec3::new(plane.position.x, plane.position.y, 0.0);
        let plane_normal_3d = Vec3::new(plane.plane.normal.x, plane.plane.normal.y, 0.0);
        let plane_basis_x_3d = Vec3::new(plane.basis_x.x, plane.basis_x.y, 0.0);
        gizmos.rect(
            Transform::from_translation(plane_pos_3d).looking_to(plane_normal_3d, Dir3::Z).to_isometry(),
            Vec2::new(plane.half_size * 2.0, 1.0),
            GREEN,
        );
        gizmos.line(plane_pos_3d, plane_pos_3d + plane_basis_x_3d, RED);
    }
}

/// set up a simple 3D scene
pub fn test_scene() -> impl SceneList {
    let up = Dir2::Y;
    bsn_list! [
        (
            #Cube
            Mesh3d(asset_value(Cuboid::new(5.0, 1.0, 1.0)))
            MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(124, 144, 255)))
            Transform::from_xyz(0.0, 0.5, 0.0)
        ),
        (
            #StagePlane
            template_value(StagePlane::new(up, 2.5, Vec2::new(0.0, 1.0)))
        ),
        (
            PointLight {
                shadow_maps_enabled: true,
            }
            Transform::from_xyz(4.0, 8.0, 4.0)
        ),
        (
            Camera3d
            template_value(Transform::from_xyz(0.0, 1.5, 9.0))
            /*FreeCamera {
                sensitivity: 0.2,
                friction: 25.0,
                walk_speed: 3.0,
                run_speed: 9.0,
            }*/
        )
    ]
}
