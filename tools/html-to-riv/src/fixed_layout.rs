//! Private fixed CSS layout-length conversion to the pinned Chrome LayoutUnit.
//!
//! This is a derived semantic value alongside authored provenance, not a change
//! to that provenance's native float or ideal value. The caller establishes that
//! the source float is the resolved computed CSS px value at the Length-to-layout
//! boundary (including any prior unit/zoom/expression computation). This module
//! does not establish that upstream premise or apply to transforms, fonts or SVG.
use super::{computed_provenance::{self, NumericSize, NumericStyle},
    scalar_provenance::ScalarProvenance};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Unresolved {
    Source(computed_provenance::Unresolved),
    NonFinite,
    /// Chrome's saturated raw LayoutUnit cannot be encoded exactly as the
    /// ordinary f32 length field. In particular, i32::MAX / 64 is not f32-exact.
    NotRepresentable,
    InvalidEncoding,
}

#[derive(Clone, Debug)]
pub(crate) struct FixedLayoutLength {
    authored: ScalarProvenance,
    computed_bits: u32,
    raw_units: i32,
    emitted: f32,
}

/// The actual f32 multiplication occurs before saturation, including overflow
/// from finite inputs. Explicit branches avoid relying on Rust float-cast
/// saturation for the conversion's semantics. Integer zero emits positive zero.
fn convert(value: f32) -> Result<(i32, f32), Unresolved> {
    if !value.is_finite() { return Err(Unresolved::NonFinite); }
    let scaled = value * 64.0_f32;
    let raw = if f64::from(scaled) >= f64::from(i32::MAX) {
        i32::MAX
    } else if f64::from(scaled) <= f64::from(i32::MIN) {
        i32::MIN
    } else {
        scaled.trunc() as i32
    };
    // This f64 operation is exact for every i32 raw value. Comparing the
    // emitted float back to it prevents dropping the last LayoutUnit fraction.
    let exact = f64::from(raw) / 64.0;
    let emitted = exact as f32;
    if f64::from(emitted) != exact { return Err(Unresolved::NotRepresentable); }
    Ok((raw, emitted))
}

impl FixedLayoutLength {
    pub(crate) fn new(source: ScalarProvenance) -> Result<Self, Unresolved> {
        let computed_bits = source.native().to_bits();
        let (raw_units, emitted) = convert(source.native())?;
        Ok(Self { authored: source, computed_bits, raw_units, emitted })
    }
    pub(crate) fn authored(&self) -> &ScalarProvenance { &self.authored }
    pub(crate) fn computed_bits(&self) -> u32 { self.computed_bits }
    pub(crate) fn raw_units(&self) -> i32 { self.raw_units }
    pub(crate) fn emitted(&self) -> f32 { self.emitted }
    /// Recheck retained source bits, the explicit conversion and output bits.
    /// Field readers must additionally bind the emitted value to actual records.
    pub(crate) fn validate(&self) -> Result<(), Unresolved> {
        if self.computed_bits != self.authored.native().to_bits() {
            return Err(Unresolved::InvalidEncoding);
        }
        let (raw, emitted) = convert(f32::from_bits(self.computed_bits))?;
        if raw != self.raw_units || emitted.to_bits() != self.emitted.to_bits() {
            return Err(Unresolved::InvalidEncoding);
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) enum LayoutLength {
    Auto,
    Fixed(FixedLayoutLength),
    /// Percentages are coefficients, not resolved layout lengths. Preserve
    /// them unchanged; live percentage-result quantization is separate work.
    Percent(ScalarProvenance),
}
impl LayoutLength {
    pub(crate) fn from_numeric(source: &NumericSize) -> Result<Self, Unresolved> {
        let scalar = |s: &computed_provenance::Scalar| s.clone().map_err(Unresolved::Source);
        match source {
            NumericSize::Auto => Ok(Self::Auto),
            NumericSize::Pixels(s) => Ok(Self::Fixed(FixedLayoutLength::new(scalar(s)?)?)),
            NumericSize::Percent(s) => Ok(Self::Percent(scalar(s)?)),
        }
    }
    pub(crate) fn units(&self) -> Option<u32> {
        match self { Self::Auto => None, Self::Fixed(_) => Some(1), Self::Percent(_) => Some(2) }
    }
    pub(crate) fn value(&self) -> Option<f32> {
        match self { Self::Auto => None, Self::Fixed(v) => Some(v.emitted()), Self::Percent(v) => Some(v.native()) }
    }
    pub(crate) fn validate(&self) -> Result<(), Unresolved> {
        match self {
            Self::Auto => Ok(()), Self::Fixed(v) => v.validate(),
            Self::Percent(v) => if v.native().is_finite() { Ok(()) } else { Err(Unresolved::NonFinite) },
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LayoutStyle {
    pub width: LayoutLength,
    pub height: LayoutLength,
    pub min_width: LayoutLength,
    pub min_height: LayoutLength,
    pub max_width: LayoutLength,
    pub max_height: LayoutLength,
}
impl LayoutStyle {
    pub(crate) fn from_numeric(source: &NumericStyle) -> Result<Self, Unresolved> {
        Ok(Self {
            width: LayoutLength::from_numeric(&source.width)?,
            height: LayoutLength::from_numeric(&source.height)?,
            min_width: LayoutLength::from_numeric(&source.min_width)?,
            min_height: LayoutLength::from_numeric(&source.min_height)?,
            max_width: LayoutLength::from_numeric(&source.max_width)?,
            max_height: LayoutLength::from_numeric(&source.max_height)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source(text: &str) -> ScalarProvenance {
        ScalarProvenance::from_decimal(text, text.parse().unwrap()).unwrap()
    }
    #[test]
    fn adjacent_fixed_units_truncate_toward_zero_for_both_signs() {
        for n in [1, 4, 16, 31, 4096, 1_048_575] {
            for sign in [-1.0_f32, 1.0] {
                let exact = n as f32 / 64.0 * sign;
                for value in [exact.next_down(), exact, exact.next_up()] {
                    let converted = FixedLayoutLength::new(ScalarProvenance::exact_constant(value).unwrap()).unwrap();
                    // Independent integer truncation reference uses f64. In
                    // this bounded input range multiplication by64 is exact.
                    let expected_raw = (f64::from(value) * 64.0).trunc() as i32;
                    assert_eq!(converted.raw_units(), expected_raw);
                    assert_eq!(f64::from(converted.emitted()), f64::from(expected_raw) / 64.0);
                    converted.validate().unwrap();
                }
            }
        }
        assert_eq!(FixedLayoutLength::new(source("-0.251")).unwrap().emitted(), -0.25);
    }
    #[test]
    fn thin_and_accumulated_decimal_inputs_keep_authored_identity() {
        let thin_value = (1.0_f32/16.0).next_up();
        let thin_source = ScalarProvenance::from_decimal("0.0625000074505806", thin_value).unwrap();
        let thin = FixedLayoutLength::new(thin_source.clone()).unwrap();
        assert_eq!(thin.raw_units(), 4);assert_eq!(thin.emitted(), 1.0/16.0);
        assert_eq!(thin.computed_bits(), thin_value.to_bits());
        assert!(thin.authored().proves_equal(&thin_source));
        assert!(!thin.authored().proves_equal(&ScalarProvenance::exact_constant(thin.emitted()).unwrap()));
        let a=FixedLayoutLength::new(source("64.249")).unwrap();
        let b=FixedLayoutLength::new(source("0.251")).unwrap();
        assert_eq!(a.raw_units(),4111);assert_eq!(b.raw_units(),16);
        assert_eq!(a.emitted()+b.emitted(),64.484375);
        assert_eq!(a.authored().native()+b.authored().native(),64.5);
        assert!(a.authored().ideal_bounds().lower()>64.234375);
    }
    #[test]
    fn zero_subnormal_and_saturation_are_explicit() {
        for value in [0.0,-0.0,f32::from_bits(1),-f32::from_bits(1)] {
            let value=FixedLayoutLength::new(ScalarProvenance::exact_constant(value).unwrap()).unwrap();
            assert_eq!(value.raw_units(),0);assert_eq!(value.emitted().to_bits(),0);
            assert_eq!(value.computed_bits(),value.authored().native().to_bits());value.validate().unwrap();
        }
        let maximum_exact=33_554_432.0_f32.next_down();
        assert_eq!(convert(maximum_exact).unwrap(),(2_147_483_520,maximum_exact));
        assert_eq!(convert(33_554_432.0),Err(Unresolved::NotRepresentable));
        assert_eq!(convert(f32::MAX),Err(Unresolved::NotRepresentable));
        assert_eq!(convert(-33_554_432.0).unwrap(),(i32::MIN,-33_554_432.0));
        assert_eq!(convert(-f32::MAX).unwrap(),(i32::MIN,-33_554_432.0));
        for invalid in [f32::NAN,f32::INFINITY,f32::NEG_INFINITY] {
            assert_eq!(convert(invalid),Err(Unresolved::NonFinite));
        }
    }
    #[test]
    fn descriptor_preserves_percent_auto_and_rejects_unknown_source() {
        let percentage=source("32.501");let mut numeric=NumericStyle::default();
        numeric.width=NumericSize::Percent(Ok(percentage.clone()));
        numeric.height=NumericSize::Pixels(Ok(source("0.251")));
        let layout=LayoutStyle::from_numeric(&numeric).unwrap();
        assert_eq!(layout.width.units(),Some(2));assert_eq!(layout.width.value().unwrap().to_bits(),percentage.native().to_bits());
        assert!(matches!(&layout.width,LayoutLength::Percent(p)if p.proves_equal(&percentage)));
        assert_eq!(layout.height.value(),Some(0.25));assert_eq!(layout.height.units(),Some(1));
        assert_eq!(layout.max_height.value(),None);assert_eq!(layout.max_width.units(),None);
        assert_eq!(layout.min_height.value(),Some(0.));assert_eq!(layout.min_width.value(),Some(0.));
        for missing in [NumericSize::Pixels(Err(computed_provenance::Unresolved::MissingOriginal)),NumericSize::Percent(Err(computed_provenance::Unresolved::MissingOriginal))] {
            numeric.width=missing;assert!(matches!(LayoutStyle::from_numeric(&numeric),Err(Unresolved::Source(computed_provenance::Unresolved::MissingOriginal))));
        }
    }
    #[test]
    fn derived_field_mutations_cannot_relabel_the_source() {
        let length=FixedLayoutLength::new(source("64.249")).unwrap();
        let mut changed=length.clone();changed.raw_units+=1;assert_eq!(changed.validate(),Err(Unresolved::InvalidEncoding));
        let mut changed=length.clone();changed.emitted=length.authored().native();assert_eq!(changed.validate(),Err(Unresolved::InvalidEncoding));
        let mut changed=length.clone();changed.computed_bits=length.emitted().to_bits();assert_eq!(changed.validate(),Err(Unresolved::InvalidEncoding));
        let mut changed=FixedLayoutLength::new(source("0")).unwrap();changed.emitted=-0.0;assert_eq!(changed.validate(),Err(Unresolved::InvalidEncoding));
    }
}
