use bevy::math::Vec2;
use bevy::prelude::*;
use fixed::types::I32F32;
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Sub};

use super::int::{FGWide, FGi32};

#[derive(Clone, Copy, PartialEq, Default, Debug, Reflect, Serialize, Deserialize, Hash)]
#[reflect(opaque)]
pub struct FGVec2 {
    pub x: FGi32,
    pub y: FGi32,
}

impl std::fmt::Display for FGVec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

impl FGVec2 {
    pub const ZERO: Self = Self::new(FGi32::ZERO, FGi32::ZERO);

    #[inline(always)]
    pub const fn new(x: FGi32, y: FGi32) -> Self {
        Self { x, y }
    }

    #[inline(always)]
    pub const fn lit(x: &str, y: &str) -> Self {
        Self {
            x: FGi32::lit(x),
            y: FGi32::lit(y),
        }
    }

    #[inline(always)]
    pub fn from_i32(x: i32, y: i32) -> Self {
        Self {
            x: FGi32::from_num(x),
            y: FGi32::from_num(y),
        }
    }

    #[inline]
    pub fn length_squared(&self) -> FGWide {
        let x = FGWide::from_num(self.x);
        let y = FGWide::from_num(self.y);
        x * x + y * y
    }

    #[inline]
    pub fn length(self) -> FGi32 {
        FGi32::from_num(self.length_squared().sqrt())
    }

    /// Returns the vector projection of `self` onto `rhs`.
    ///
    /// `rhs` must be normalized.
    ///
    /// # Panics
    ///
    /// Will panic if `rhs` is not normalized when `glam_assert` is enabled.
    #[inline]
    #[must_use]
    pub fn project_onto_normalized(self, rhs: Self) -> Self {
        assert!(rhs.is_normalized());

        let scale = self.dot(rhs);
        Self::new(
            FGi32::from_num(FGWide::from_num(rhs.x) * scale),
            FGi32::from_num(FGWide::from_num(rhs.y) * scale),
        )
    }

    const EPS: FGi32 = FGi32::from_bits(6);

    #[inline]
    #[must_use]
    pub fn is_normalized(self) -> bool {
        let diff = self.length() - FGi32::ONE;
        diff.abs() <= Self::EPS
    }

    #[inline]
    #[must_use]
    pub fn dot(self, rhs: Self) -> FGWide {
        FGWide::from_num(self.x) * FGWide::from_num(rhs.x)
            + FGWide::from_num(self.y) * FGWide::from_num(rhs.y)
    }

    #[inline]
    #[must_use]
    pub fn project_onto(self, rhs: Self) -> Self {
        let other_len_sq = rhs.dot(rhs);
        assert!(other_len_sq != FGWide::ZERO);
        let scale = self.dot(rhs) / other_len_sq;
        Self::new(
            FGi32::from_num(FGWide::from_num(rhs.x) * scale),
            FGi32::from_num(FGWide::from_num(rhs.y) * scale),
        )
    }

    #[inline]
    pub fn normalize_or_zero(&self) -> Self {
        self.normalize().unwrap_or(FGVec2::ZERO)
    }

    #[inline]
    pub fn direction_to(self, to: Self) -> Option<Self> {
        (to - self).normalize()
    }

    #[inline]
    #[must_use]
    pub fn distance(self, to: Self) -> FGi32 {
        (to - self).length()
    }

    #[inline]
    #[must_use]
    pub fn distance_squared(self, to: Self) -> FGWide {
        (to - self).length_squared()
    }

    #[inline]
    pub fn normalize(&self) -> Option<Self> {
        let x = I32F32::from_num(self.x);
        let y = I32F32::from_num(self.y);
        let length_squared = x * x + y * y;

        if length_squared.is_zero() {
            None
        } else {
            let length = length_squared.sqrt();
            Some(Self::new(
                FGi32::from_num(x / length),
                FGi32::from_num(y / length),
            ))
        }
    }

    #[inline]
    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.x.to_num(), self.y.to_num())
    }
}

impl Mul for FGVec2 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl Add for FGVec2 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for FGVec2 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Sub<&FGVec2> for &FGVec2 {
    type Output = FGVec2;
    #[inline]
    fn sub(self, rhs: &FGVec2) -> FGVec2 {
        (*self).sub(*rhs)
    }
}

impl Div for FGVec2 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl Mul<FGi32> for FGVec2 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: FGi32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<FGi32> for FGVec2 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: FGi32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl Div<FGi32> for &FGVec2 {
    type Output = FGVec2;

    #[inline]
    fn div(self, rhs: FGi32) -> Self::Output {
        (*self).div(rhs)
    }
}

impl AddAssign for FGVec2 {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl MulAssign for FGVec2 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl MulAssign<FGi32> for FGVec2 {
    fn mul_assign(&mut self, rhs: FGi32) {
        *self = Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl From<(FGi32, FGi32)> for FGVec2 {
    fn from(value: (FGi32, FGi32)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

#[cfg(test)]
mod is_normalized_tests {
    use super::*;
    use fixed::types::I16F16 as FGi32;

    // Normalization is computed in Q32.32, so the Q16.16 error comes from the
    // two final component conversions and length_squared()'s multiplications.
    const EPS_FIXED_BITS: i32 = 4;

    #[test]
    fn normalizes_small_axis_vector_without_losing_squared_precision() {
        let normalized = FGVec2::lit("0.01", "0").normalize().unwrap();

        assert_eq!(normalized, FGVec2::lit("1", "0"));
        assert!(normalized.is_normalized());
    }

    #[test]
    fn measure_normalize_error_bound() {
        let mut max_abs_diff_bits: i32 = 0;
        let mut worst_case: Option<(f64, f64, i32)> = None;

        let angle_steps = 3600; // 0.1 degree resolution
        let magnitudes: &[f64] = &[
            0.001, 0.01, 0.1, 0.5, 1.0, 2.0, 10.0, 100.0, 1000.0, 30000.0,
        ];

        for &mag in magnitudes {
            for i in 0..angle_steps {
                let theta = (i as f64) * std::f64::consts::TAU / (angle_steps as f64);
                let x = mag * theta.cos();
                let y = mag * theta.sin();

                let Some(fx) = FGi32::checked_from_num(x) else {
                    continue;
                };
                let Some(fy) = FGi32::checked_from_num(y) else {
                    continue;
                };

                let v = FGVec2 { x: fx, y: fy };

                if let Some(normalized) = v.normalize() {
                    let ls = FGi32::from_num(normalized.length_squared());
                    let diff_bits = ls.to_bits() - FGi32::ONE.to_bits();
                    let abs_diff_bits = diff_bits.abs();

                    if abs_diff_bits > max_abs_diff_bits {
                        max_abs_diff_bits = abs_diff_bits;
                        worst_case = Some((x, y, abs_diff_bits));
                    }
                }
            }
        }

        let ulp_value = FGi32::DELTA.to_num::<f64>();
        println!(
            "Max |length_squared - 1.0| = {} raw bits ({:e} real), worst case: {:?}",
            max_abs_diff_bits,
            max_abs_diff_bits as f64 * ulp_value,
            worst_case
        );

        assert!(
            max_abs_diff_bits <= EPS_FIXED_BITS,
            "measured error ({} bits) exceeds analytical bound ({} bits) — \
             revisit the epsilon derivation",
            max_abs_diff_bits,
            EPS_FIXED_BITS
        );

        println!(
            "Bound: {} bits, measured: {} bits (headroom: {} bits)",
            EPS_FIXED_BITS,
            max_abs_diff_bits,
            EPS_FIXED_BITS - max_abs_diff_bits
        );
    }

    #[test]
    fn is_normalized_behaves_correctly() {
        let angle_steps = 360;
        for i in 0..angle_steps {
            let theta = (i as f64) * std::f64::consts::TAU / (angle_steps as f64);
            for &mag in &[0.01, 1.0, 100.0, 10000.0] {
                let x = mag * theta.cos();
                let y = mag * theta.sin();
                let Some(fx) = FGi32::checked_from_num(x) else {
                    continue;
                };
                let Some(fy) = FGi32::checked_from_num(y) else {
                    continue;
                };
                let v = FGVec2 { x: fx, y: fy };

                if let Some(normalized) = v.normalize() {
                    assert!(
                        normalized.is_normalized(),
                        "normalized({}, {}) failed is_normalized(): length_squared = {:?}",
                        x,
                        y,
                        normalized.length_squared()
                    );
                }
            }
        }

        let non_unit = FGVec2 {
            x: FGi32::from_num(2.0),
            y: FGi32::from_num(0.0),
        };
        assert!(!non_unit.is_normalized());

        let non_unit_small = FGVec2 {
            x: FGi32::from_num(0.5),
            y: FGi32::from_num(0.0),
        };
        assert!(!non_unit_small.is_normalized());
    }

    #[test]
    fn dot_and_length_squared_use_wide_intermediates() {
        let vector = FGVec2::lit("200", "200");

        assert_eq!(vector.dot(vector), FGWide::from_num(80_000));
        assert_eq!(vector.length_squared(), FGWide::from_num(80_000));
    }
}
