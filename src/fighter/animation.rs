use bevy::prelude::*;
use strum_macros::EnumIter;

use crate::fighter::{FighterFacingDirection, FighterVisual};

/// The simulation-owned animation clock. Rendering may interpolate freely, but
/// gameplay samples baked data only with this rollback-tracked value.
#[derive(Component, Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct FighterAnimationFrame {
    pub kind: AnimKind,
    pub frame: u32,
    pub repeat: bool,
}

impl FighterAnimationFrame {
    pub const fn new(kind: AnimKind, repeat: bool) -> Self {
        Self {
            kind,
            frame: 0,
            repeat,
        }
    }

    pub fn reset(&mut self, kind: AnimKind, repeat: bool) {
        self.kind = kind;
        self.frame = 0;
        self.repeat = repeat;
    }

    pub fn advance(&mut self) {
        self.frame = self.frame.saturating_add(1);
    }
}

pub fn advance_fighter_animation_frames(mut fighters: Query<&mut FighterAnimationFrame>) {
    for mut animation in &mut fighters {
        animation.advance();
    }
}

pub fn apply_animation(
    children: Query<
        (
            &mut Transform,
            &FighterFacingDirection,
            &FighterAnimationFrame,
        ),
        With<FighterVisual>,
    >,
) {
    for (mut trf, facing_direction, frame) in children {
        // Melee model scale is in decimeters
        trf.rotation = Quat::IDENTITY;
        if let AnimKind::Turn = frame.kind {
            trf.rotate_local_y(
                std::f32::consts::PI * -0.5 * facing_direction.to_sign().to_num::<f32>(),
            );
        } else {
            trf.rotate_local_y(
                std::f32::consts::PI * 0.5 * facing_direction.to_sign().to_num::<f32>(),
            );
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize, Reflect, EnumIter,
)]
pub enum AnimKind {
    Wait,
    Walk,
    Run,
    Dash,
    JumpSquat,
    JumpForward,
    JumpBack,
    DoubleJump,
    Landing,
    AirDodge,
    Fall,
    Turn,

    // Attacks
    AttackJab1,
    AttackUpTilt,
    AttackForwardTilt,
    AttackDownTilt,
}
