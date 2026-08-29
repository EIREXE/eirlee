use bevy::prelude::*;

use crate::math::{int::FGi32, vec::FGVec2};

/// Environment collision box: the diamond used to resolve a fighter against
/// the stage. Purely geometry — the collision response lives in
/// [`super::collision`].
#[derive(Component, Debug, Clone, Copy)]
#[require(FighterPreviousECB)]
pub struct FighterECB {
    pub vertical_half: FGi32,
    pub horizontal_half: FGi32,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct FighterPreviousECB {
    pub vertical_half: FGi32,
    pub horizontal_half: FGi32,
}

impl From<FighterECB> for FighterPreviousECB {
    fn from(ecb: FighterECB) -> Self {
        Self {
            vertical_half: ecb.vertical_half,
            horizontal_half: ecb.horizontal_half,
        }
    }
}

pub fn snapshot_fighter_ecb(fighters: Query<(&FighterECB, &mut FighterPreviousECB)>) {
    for (ecb, mut previous_ecb) in fighters {
        *previous_ecb = (*ecb).into();
    }
}

impl FighterECB {
    pub fn to_3d_lineloop(&self, trf: Transform) -> [Vec3; 4] {
        let rel_up = trf.transform_point(Vec3::new(
            0.0,
            (self.vertical_half + self.vertical_half).to_num(),
            0.0,
        ));
        let rel_down = trf.transform_point(Vec3::new(0.0, 0.0, 0.0));
        let rel_left = trf.transform_point(Vec3::new(
            (-self.horizontal_half).to_num(),
            self.vertical_half.to_num(),
            0.0,
        ));
        let rel_right = trf.transform_point(Vec3::new(
            self.horizontal_half.to_num(),
            self.vertical_half.to_num(),
            0.0,
        ));
        [rel_up, rel_right, rel_down, rel_left]
    }

    pub fn get_bottom_point(&self) -> FGVec2 {
        FGVec2::new(FGi32::ZERO, FGi32::ZERO)
    }
}

impl FighterPreviousECB {
    pub fn to_3d_lineloop(&self, trf: Transform) -> [Vec3; 4] {
        let rel_up = trf.transform_point(Vec3::new(
            0.0,
            (self.vertical_half + self.vertical_half).to_num(),
            0.0,
        ));
        let rel_down = trf.transform_point(Vec3::new(0.0, 0.0, 0.0));
        let rel_left = trf.transform_point(Vec3::new(
            (-self.horizontal_half).to_num(),
            self.vertical_half.to_num(),
            0.0,
        ));
        let rel_right = trf.transform_point(Vec3::new(
            self.horizontal_half.to_num(),
            self.vertical_half.to_num(),
            0.0,
        ));
        [rel_up, rel_right, rel_down, rel_left]
    }

    pub fn get_bottom_point(&self) -> FGVec2 {
        FGVec2::new(FGi32::ZERO, FGi32::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn previous_ecb_copies_current_dimensions() {
        let current = FighterECB {
            vertical_half: FGi32::lit("0.5"),
            horizontal_half: FGi32::lit("0.25"),
        };
        let previous = FighterPreviousECB::from(current);

        assert_eq!(previous.vertical_half, current.vertical_half);
        assert_eq!(previous.horizontal_half, current.horizontal_half);
    }
}
