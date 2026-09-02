use std::collections::{HashMap, VecDeque};

use bevy::{asset::Handle, ecs::component::Component, reflect::Reflect};
use serde::{Deserialize, Serialize};

use crate::{
    fighter::{FighterAttributes, animation::AnimKind},
    math::{int::FGi32, vec3::FGVec3},
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

#[derive(Component)]
pub struct FighterAttackScriptAssets {
    pub scripts: HashMap<AttackKind, Handle<FighterAttackScript>>,
}
