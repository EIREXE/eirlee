pub type FGi32 = fixed::FixedI32<fixed::types::extra::U16>;
pub type FGWide = fixed::FixedI128<fixed::types::extra::U32>;

/// Angle conversions for the game's Q16.16 fixed-point values.
pub trait FGi32Ext {
    fn to_radians(self) -> FGi32;
    fn sin_cos(self) -> (FGi32, FGi32);
}

impl FGi32Ext for FGi32 {
    fn to_radians(self) -> FGi32 {
        FGi32::from_num(FGWide::from_num(self) * FGWide::lit("0.01745329251994329576923690768489"))
    }

    fn sin_cos(self) -> (FGi32, FGi32) {
        const PI: FGi32 = FGi32::lit("3.14159265358979323846");
        const HALF_PI: FGi32 = FGi32::lit("1.57079632679489661923");
        const TWO_PI: FGi32 = FGi32::lit("6.28318530717958647692");
        const CORDIC_GAIN_INVERSE: FGi32 = FGi32::lit("0.60725293500888125617");
        const ANGLES: [FGi32; 16] = [
            FGi32::lit("0.78539816339744830962"),
            FGi32::lit("0.46364760900080611621"),
            FGi32::lit("0.24497866312686415417"),
            FGi32::lit("0.12435499454676143503"),
            FGi32::lit("0.06241880999595734847"),
            FGi32::lit("0.03123983343026827625"),
            FGi32::lit("0.01562372862047683080"),
            FGi32::lit("0.00781234106010111130"),
            FGi32::lit("0.00390623013196697183"),
            FGi32::lit("0.00195312251647881869"),
            FGi32::lit("0.00097656218955931943"),
            FGi32::lit("0.00048828121119489829"),
            FGi32::lit("0.00024414062014936177"),
            FGi32::lit("0.00012207031189367021"),
            FGi32::lit("0.00006103515617420878"),
            FGi32::lit("0.00003051757811552610"),
        ];

        let mut angle = self % TWO_PI;
        if angle > PI {
            angle -= TWO_PI;
        } else if angle < -PI {
            angle += TWO_PI;
        }
        let mut sign = FGi32::ONE;
        if angle > HALF_PI {
            angle -= PI;
            sign = FGi32::NEG_ONE;
        } else if angle < -HALF_PI {
            angle += PI;
            sign = FGi32::NEG_ONE;
        }

        let mut x = CORDIC_GAIN_INVERSE;
        let mut y = FGi32::ZERO;
        for (index, step) in ANGLES.into_iter().enumerate() {
            let scale = FGi32::from_bits(1 << (16 - index));
            if angle >= FGi32::ZERO {
                let next_x = x - y * scale;
                y += x * scale;
                x = next_x;
                angle -= step;
            } else {
                let next_x = x + y * scale;
                y -= x * scale;
                x = next_x;
                angle += step;
            }
        }
        let snap_axis = |value: FGi32| {
            const EPSILON: FGi32 = FGi32::from_bits(4);
            if value.abs() <= EPSILON {
                FGi32::ZERO
            } else if (value.abs() - FGi32::ONE).abs() <= EPSILON {
                if value < FGi32::ZERO {
                    FGi32::NEG_ONE
                } else {
                    FGi32::ONE
                }
            } else {
                value
            }
        };
        (snap_axis(y * sign), snap_axis(x * sign))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_degrees_convert_to_radians() {
        assert_eq!(
            FGi32::lit("180").to_radians(),
            FGi32::lit("3.1415863037109375")
        );
    }
}
