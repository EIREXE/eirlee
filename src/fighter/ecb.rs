use bevy::prelude::*;

/// Environment collision box: the diamond used to resolve a fighter against
/// the stage. Purely geometry — the collision response lives in
/// [`super::collision`].
#[derive(Component, Debug, Clone, Copy)]
#[require(FighterPreviousECB)]
pub struct FighterECB {
    pub vertical_half: f32,
    pub horizontal_half: f32,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct FighterPreviousECB {
    pub vertical_half: f32,
    pub horizontal_half: f32,
}

impl FighterECB {
    pub fn to_3d_lineloop(&self, trf: Transform) -> [Vec3; 4] {
        let rel_up = trf.transform_point(Vec3::new(0.0, self.vertical_half, 0.0));
        let rel_down = trf.transform_point(Vec3::new(0.0, -self.vertical_half, 0.0));
        let rel_left = trf.transform_point(Vec3::new(-self.horizontal_half, 0.0, 0.0));
        let rel_right = trf.transform_point(Vec3::new(self.horizontal_half, 0.0, 0.0));
        [rel_up, rel_right, rel_down, rel_left]
    }

    pub fn get_bottom_point(&self) -> Vec2 {
        Vec2::new(0.0, -self.vertical_half)
    }
}

impl FighterPreviousECB {
    pub fn to_3d_lineloop(&self, trf: Transform) -> [Vec3; 4] {
        let rel_up = trf.transform_point(Vec3::new(0.0, self.vertical_half, 0.0));
        let rel_down = trf.transform_point(Vec3::new(0.0, -self.vertical_half, 0.0));
        let rel_left = trf.transform_point(Vec3::new(-self.horizontal_half, 0.0, 0.0));
        let rel_right = trf.transform_point(Vec3::new(self.horizontal_half, 0.0, 0.0));
        [rel_up, rel_right, rel_down, rel_left]
    }

    pub fn get_bottom_point(&self) -> Vec2 {
        Vec2::new(0.0, -self.vertical_half)
    }
}
