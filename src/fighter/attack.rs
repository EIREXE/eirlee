use std::collections::{HashMap, VecDeque};

use bevy::prelude::*;
use bevy_ggrs::prelude::GgrsSchedule;
use serde::{Deserialize, Serialize};

use crate::{
    fighter::{
        Fighter, FighterAttributes, FighterFacingDirection, FighterTranslation,
        animation::AnimKind,
        baked_animation::{FighterBoneMatrices, FixedMat4},
        hurtbox::FixedAffineCapsule,
        manifest::FighterManifest,
    },
    math::{
        int::{FGi32, FGi32Ext},
        vec3::FGVec3,
    },
    schedule::GameplaySet,
    scripting::FighterAttackScript,
};
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FighterDamage(FGi32);

impl FighterDamage {
    pub fn lit(text: &str) -> Self {
        Self(FGi32::lit(text))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum KnockbackType {
    Normal,
    Fixed,
}

impl KnockbackType {
    pub fn make_normal() -> Self {
        Self::Normal
    }

    pub fn make_fixed() -> Self {
        Self::Fixed
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Knockback(FGi32);

impl Knockback {
    pub fn lit(val: &str) -> Self {
        Self(FGi32::lit(val))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttackAngle {
    Normal(FGi32),
    Sakurai,
}

impl Default for AttackAngle {
    fn default() -> Self {
        AttackAngle::Normal(FGi32::lit("0"))
    }
}

impl AttackAngle {
    pub fn make_normal(angle: FGi32) -> Self {
        Self::Normal(angle)
    }

    pub fn make_sakurai() -> Self {
        Self::Sakurai
    }
}

#[derive(Serialize, Deserialize, Reflect, PartialEq, Eq, Hash, Clone, Debug)]
pub enum AttackKind {
    NAir,
    Jab,
    UpTilt,
}

impl AttackKind {
    pub fn get_animation(&self) -> AnimKind {
        match self {
            AttackKind::NAir => AnimKind::AttackJab1,
            AttackKind::Jab => AnimKind::AttackJab1,
            AttackKind::UpTilt => AnimKind::AttackUpTilt,
        }
    }
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(opaque)]
pub struct AttackHitbox {
    pub id: u32,
    pub bone: String,
    pub damage: FighterDamage,
    pub offset: FGVec3,
    pub radius: FGi32,
    pub angle: AttackAngle,
    pub knockback_type: KnockbackType,
    pub knockback: Knockback,
    pub knockback_growth: FGi32,
    pub start_frame: u32,
    pub end_frame: u32,
}

pub struct StaleMoveQueue(pub VecDeque<AttackKind>);

impl AttackHitbox {
    /// Knockback calculation
    pub fn calculate_knockback(
        &self,
        attack: AttackKind,

        receiver_attribs: &FighterAttributes,
        receiver_damage: FighterDamage,
        total_damage_received_this_frame: FighterDamage,
        interrupted_smash_charge: bool,
        receiver_crouching: bool,
    ) -> Knockback {
        /// Weight that produces a weight modifier of exactly 1.0. Weight-independent
        /// attacks pass this value in place of the target's real weight.
        const BASELINE_WEIGHT: FGi32 = FGi32::lit("100.0");

        /// Applied to the weight-scaled damage term.
        const KB_GAIN: FGi32 = FGi32::lit("1.4");

        /// Constant floor added to the damage term before knockback growth scales it.
        /// At 0% and 0 damage, knockback reduces to KB_OFFSET * s + b.
        const KB_OFFSET: FGi32 = FGi32::lit("18.0");

        /// Knockback growth is authored in percent (110 means 1.1x).
        const KB_GROWTH_SCALE: FGi32 = FGi32::lit("0.01");
        const DAMAGE_TERM_DIVISOR: FGi32 = FGi32::lit("20.0");
        const DAMAGE_BONUS: FGi32 = FGi32::lit("2.0"); // effective damage floor in the damage term
        const CHARGE_SMASH_INTERRUPTION_MODIFIER: FGi32 = FGi32::lit("1.2");
        const CROUCH_CANCEL_MODIFIER: FGi32 = FGi32::lit("0.666667");

        let attack_damage_unstalled = self.damage.0;

        let modifier = {
            let mut out = FGi32::ONE;
            if interrupted_smash_charge {
                out *= CHARGE_SMASH_INTERRUPTION_MODIFIER
            } else if receiver_crouching {
                out *= CROUCH_CANCEL_MODIFIER
            }
            out
        };

        let s = KB_GROWTH_SCALE * self.knockback_growth;
        let p = receiver_damage.0.floor() + total_damage_received_this_frame.0;
        let d = attack_damage_unstalled;
        let damage_term = p / DAMAGE_TERM_DIVISOR * (d + DAMAGE_BONUS);

        let scaled = damage_term * (FGi32::lit("2.0") * BASELINE_WEIGHT)
            / (receiver_attribs.weight + BASELINE_WEIGHT);
        let kb = (scaled * KB_GAIN + KB_OFFSET) * s + self.knockback.0;

        Knockback(modifier * kb)
    }
}

pub fn update_active_hitbox_list(script: &FighterAttackScript, list: &mut Vec<usize>, frame: u32) {
    *list = script
        .hitboxes
        .iter()
        .enumerate()
        .filter(|(_, hb)| hb.start_frame <= frame && hb.end_frame > frame)
        .map(|(i, _)| i)
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
pub struct FighterSolvedHurtbox {
    pub hurtbox_idx: usize,
    pub capsule: FixedAffineCapsule,
}

#[derive(Component)]
pub struct FighterAttackScriptAssets {
    pub scripts: HashMap<AttackKind, Handle<FighterAttackScript>>,
}
#[derive(Component, Clone, Debug, Default)]
pub struct FighterSolvedHurtboxes(pub Vec<FighterSolvedHurtbox>);

pub fn solve_hurtboxes(
    mut query: Query<(
        &FighterBoneMatrices,
        &FighterTranslation,
        &FighterFacingDirection,
        &mut FighterSolvedHurtboxes,
        &Fighter,
    )>,
    manifests: Res<Assets<FighterManifest>>,
) {
    for (matrices, translation, facing_direction, mut solved_hurtboxes, fighter) in &mut query {
        let manifest = manifests
            .get(&fighter.manifest)
            .expect("Manifest should be valid");
        let fighter_trf = translation.get_3d_transform(facing_direction);
        solved_hurtboxes.0.clear();
        for (hurtbox_idx, hurtbox) in manifest.hurtboxes.iter().enumerate() {
            let bone_matrix = matrices.get_current(&hurtbox.bone);

            if let Some(bone_matrix) = bone_matrix {
                let to_global = fighter_trf.mul(bone_matrix);
                let hurtbox_trf = FixedMat4::IDENTITY
                    .with_rotation(FGVec3::new(
                        hurtbox.rotation.x.to_radians(),
                        hurtbox.rotation.y.to_radians(),
                        hurtbox.rotation.z.to_radians(),
                    ))
                    .with_translation(hurtbox.offset);
                solved_hurtboxes.0.push(FighterSolvedHurtbox {
                    hurtbox_idx,
                    capsule: FixedAffineCapsule::new(
                        to_global.mul(hurtbox_trf),
                        hurtbox.half_length,
                        hurtbox.radius,
                    ),
                });
            }
        }
    }
}

pub struct FighterAttackPlugin;

impl Plugin for FighterAttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            GgrsSchedule,
            solve_hurtboxes
                .after(crate::fighter::baked_animation::update_fighter_bone_matrices)
                .in_set(GameplaySet::Animation),
        );
    }
}
