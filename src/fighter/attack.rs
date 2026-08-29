use crate::math::{int::FGi32, vec3::FGVec3};


pub struct FighterMoveManifest {
    script_path: String,
    
}

#[derive(Debug, Default, Clone)]
pub struct FighterDamage(FGi32);

impl FighterDamage {
    pub fn lit(text: &str) -> Self {
        Self(FGi32::lit(text))
    }
}

#[derive(Clone, Debug)]
pub enum KnockbackType {
    Normal,
    Fixed
}

impl KnockbackType {
    pub fn make_normal() -> Self {
        Self::Normal
    }

    pub fn make_fixed() -> Self {
        Self::Fixed
    }
}

#[derive(Debug, Default, Clone)]
pub struct Knockback(FGi32);

impl Knockback {
    pub fn lit(val: &str) -> Self {
        Self(FGi32::lit(val))
    }
}

#[derive(Debug, Clone)]
pub enum AttackAngle {
    Normal(FGi32),
    Sakurai
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

#[derive(Debug)]
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
    pub end_frame: u32
}