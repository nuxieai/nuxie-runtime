//! Private metadata for exact CSS values alongside unchanged native floats.
//!
//! Construction always needs original numeric provenance or an explicitly exact
//! compiler constant. Decimal enclosures and arithmetic round outward in f64.
//! This carrier neither changes native values nor admits a compiler feature.
use std::sync::Arc;

const PREFIX_DIGITS: usize = 18;
const EQUALITY_DIGITS: usize = 128;
const MIN_F64: f64 = f64::from_bits(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScalarError {
    MissingOriginal,
    InvalidDecimal,
    NonFiniteNative,
    IdealRangeOverflow,
    InvalidInterval,
    InvalidDivisor,
}

/// An outward enclosure of the mathematical value, not a native f32 range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IdealBounds {
    lower: f64,
    upper: f64,
}
impl IdealBounds {
    pub(crate) fn new(lower: f64, upper: f64) -> Result<Self, ScalarError> {
        if !lower.is_finite() || !upper.is_finite() || lower > upper {
            return Err(ScalarError::InvalidInterval);
        }
        Ok(Self { lower, upper })
    }
    fn point(value: f64) -> Self { Self { lower: value, upper: value } }
    pub(crate) fn lower(self) -> f64 { self.lower }
    pub(crate) fn upper(self) -> f64 { self.upper }
    fn is_zero(self) -> bool { self.lower == 0. && self.upper == 0. }
    fn negate(self) -> Self { Self { lower: -self.upper, upper: -self.lower } }

    pub(crate) fn multiply(self, rhs: Self) -> Result<Self, ScalarError> {
        if self.is_zero() || rhs.is_zero() { return Ok(Self::point(0.)); }
        let mut lower = f64::INFINITY;
        let mut upper = f64::NEG_INFINITY;
        for a in [self.lower, self.upper] {
            for b in [rhs.lower, rhs.upper] {
                // Multiplication by exactly zero is exact, including subnormals.
                let (lo, hi) = if a == 0. || b == 0. { (0., 0.) }
                    else { outward(a * b)? };
                lower = lower.min(lo);
                upper = upper.max(hi);
            }
        }
        // Avoid manufacturing the opposite sign when a tiny product underflows.
        if (self.lower >= 0. && rhs.lower >= 0.) || (self.upper <= 0. && rhs.upper <= 0.) {
            lower = lower.max(0.);
        }
        if (self.lower >= 0. && rhs.upper <= 0.) || (self.upper <= 0. && rhs.lower >= 0.) {
            upper = upper.min(0.);
        }
        Self::new(lower, upper)
    }

    pub(crate) fn divide_positive(self, divisor: f64) -> Result<Self, ScalarError> {
        if !divisor.is_finite() || divisor <= 0. { return Err(ScalarError::InvalidDivisor); }
        if self.is_zero() { return Ok(self); }
        let mut lower = if self.lower == 0. { 0. } else { outward(self.lower / divisor)?.0 };
        let mut upper = if self.upper == 0. { 0. } else { outward(self.upper / divisor)?.1 };
        if self.lower >= 0. { lower = lower.max(0.); }
        if self.upper <= 0. { upper = upper.min(0.); }
        Self::new(lower, upper)
    }

    /// May return infinity when no finite f64 error upper bound is available.
    pub(crate) fn error_from(self, native: f32) -> f64 {
        if !native.is_finite() { return f64::INFINITY; }
        let native = f64::from(native);
        if self.lower == native && self.upper == native { return 0.; }
        (native - self.lower).abs().next_up().max((native - self.upper).abs().next_up())
    }
}

fn outward(rounded: f64) -> Result<(f64, f64), ScalarError> {
    if !rounded.is_finite() { return Err(ScalarError::IdealRangeOverflow); }
    let pair = (rounded.next_down(), rounded.next_up());
    if !pair.0.is_finite() || !pair.1.is_finite() { return Err(ScalarError::IdealRangeOverflow); }
    Ok(pair)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sign { Negative, Zero, Positive }
impl Sign {
    fn product(self, rhs: Self) -> Self {
        if self == Self::Zero || rhs == Self::Zero { Self::Zero }
        else if self == rhs { Self::Positive } else { Self::Negative }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Dyadic { negative: bool, odd: u128, exponent: i128 }
#[derive(Clone, Debug, PartialEq, Eq)]
struct DecimalKey { negative: bool, digits: Box<str>, exponent: i128 }
#[derive(Debug)]
enum Identity {
    Dyadic(Dyadic),
    Decimal(DecimalKey),
    // A shared opaque identity proves copies refer to the same ideal value;
    // independently constructed opaque identities never prove equality.
    Opaque,
}

#[derive(Clone, Debug)]
pub(crate) struct ScalarProvenance {
    native: f32,
    ideal: IdealBounds,
    sign: Sign,
    identity: Arc<Identity>,
}
impl ScalarProvenance {
    /// `original` is the original numeric token spelling, excluding its unit.
    /// `native` is the actual existing parser/computation result, not a reparse
    /// performed by this module. In particular, serialization need not roundtrip.
    pub(crate) fn from_decimal(original: &str, native: f32) -> Result<Self, ScalarError> {
        finite_native(native)?;
        let parsed = parse_decimal(original)?;
        Ok(Self { native, ideal: parsed.bounds, sign: parsed.sign, identity: Arc::new(parsed.identity) })
    }
    pub(crate) fn from_optional_decimal(original: Option<&str>, native: f32) -> Result<Self, ScalarError> {
        Self::from_decimal(original.ok_or(ScalarError::MissingOriginal)?, native)
    }
    /// Only use for a mathematically exact compiler-defined value (for example
    /// root font size16 or an initial factor1), never to fill missing provenance.
    pub(crate) fn exact_constant(native: f32) -> Result<Self, ScalarError> {
        finite_native(native)?;
        let sign = if native == 0. { Sign::Zero } else if native < 0. { Sign::Negative } else { Sign::Positive };
        Ok(Self { native, ideal: IdealBounds::point(f64::from(native)), sign,
            identity: Arc::new(Identity::Dyadic(f32_dyadic(native))) })
    }
    /// Update the actual float after native normalization/parsing while retaining
    /// the original ideal semantics. This is not an arithmetic operation.
    pub(crate) fn with_native(&self, native: f32) -> Result<Self, ScalarError> {
        finite_native(native)?;
        Ok(Self { native, ..self.clone() })
    }
    pub(crate) fn native(&self) -> f32 { self.native }
    pub(crate) fn ideal_bounds(&self) -> IdealBounds { self.ideal }
    pub(crate) fn is_exact_zero(&self) -> bool { self.sign == Sign::Zero }
    pub(crate) fn is_nonnegative(&self) -> bool { self.sign != Sign::Negative }
    pub(crate) fn absolute_error_upper(&self) -> f64 { self.ideal.error_from(self.native) }

    /// Proves equality of ideal values only. The caller must separately compare
    /// native floats when a lowering requires both to be equal. False means
    /// unknown or different; overlapping bounds never constitute a proof.
    pub(crate) fn proves_equal(&self, rhs: &Self) -> bool {
        if self.is_exact_zero() && rhs.is_exact_zero() { return true; }
        if Arc::ptr_eq(&self.identity, &rhs.identity) { return true; }
        match (&*self.identity, &*rhs.identity) {
            (Identity::Dyadic(a), Identity::Dyadic(b)) => a == b,
            (Identity::Decimal(a), Identity::Decimal(b)) => a == b,
            _ => false,
        }
    }

    /// The native expression is exactly one f32 multiplication, as for em/rem.
    pub(crate) fn multiply(&self, rhs: &Self) -> Result<Self, ScalarError> {
        let native = self.native * rhs.native;
        finite_native(native)?;
        Ok(Self { native, ideal: self.ideal.multiply(rhs.ideal)?, sign: self.sign.product(rhs.sign),
            identity: Arc::new(Identity::Opaque) })
    }
    pub(crate) fn divide_100(&self) -> Result<Self, ScalarError> {
        let native = self.native / 100.0_f32;
        finite_native(native)?;
        Ok(Self { native, ideal: self.ideal.divide_positive(100.)?, sign: self.sign,
            identity: Arc::new(Identity::Opaque) })
    }
    /// Compiler font-size percentages, and percentage-padding coefficients,
    /// divide before multiplication. Do not substitute preferred_percent here.
    pub(crate) fn font_percent(&self, parent: &Self) -> Result<Self, ScalarError> {
        self.divide_100()?.multiply(parent)
    }
    /// Evaluates a concrete owner scalar, not a whole viewport domain. Keep a
    /// computed percentage as a coefficient until domain analysis resolves it.
    /// Immutable preferred/min/max dimensions: percent * owner * 0.01f.
    /// Check the first product too; a final small multiplier cannot hide overflow.
    pub(crate) fn preferred_percent(&self, owner: &Self) -> Result<Self, ScalarError> {
        self.multiply(owner)?.multiply(&Self::from_decimal("0.01", 0.01_f32)?)
    }
}
fn finite_native(value: f32) -> Result<(), ScalarError> {
    if value.is_finite() { Ok(()) } else { Err(ScalarError::NonFiniteNative) }
}
fn normalized_dyadic(negative: bool, mut integer: u128, mut exponent: i128) -> Dyadic {
    if integer == 0 { return Dyadic { negative: false, odd: 0, exponent: 0 }; }
    let shift = integer.trailing_zeros();
    integer >>= shift;
    exponent += i128::from(shift);
    Dyadic { negative, odd: integer, exponent }
}
fn f32_dyadic(value: f32) -> Dyadic {
    let bits = value.to_bits();
    let exponent = (bits >> 23) & 255;
    let mantissa = bits & 0x7fffff;
    let (integer, exponent) = if exponent == 0 { (mantissa, -149) }
        else { (mantissa | (1 << 23), i128::from(exponent) - 127 - 23) };
    normalized_dyadic(bits >> 31 != 0, u128::from(integer), exponent)
}
fn decimal_identity(negative: bool, digits: &[u8], exponent: i128) -> Identity {
    let text = std::str::from_utf8(digits).expect("decimal digits");
    if let Ok(mut integer) = text.parse::<u128>() {
        if exponent < 0 {
            let mut remaining = -exponent;
            while remaining != 0 && integer % 5 == 0 { integer /= 5; remaining -= 1; }
            if remaining == 0 { return Identity::Dyadic(normalized_dyadic(negative, integer, exponent)); }
        } else {
            let mut remaining = exponent;
            while remaining != 0 {
                let Some(next) = integer.checked_mul(5) else { break; };
                integer = next; remaining -= 1;
            }
            if remaining == 0 { return Identity::Dyadic(normalized_dyadic(negative, integer, exponent)); }
        }
    }
    Identity::Decimal(DecimalKey { negative, digits: text.into(), exponent })
}

struct ParsedDecimal { bounds: IdealBounds, sign: Sign, identity: Identity }
struct Digits {
    prefix: u64,
    significant: usize,
    trailing_zeroes: usize,
    discarded_nonzero: bool,
    key: [u8; EQUALITY_DIGITS],
}
impl Digits {
    fn new() -> Self { Self { prefix: 0, significant: 0, trailing_zeroes: 0, discarded_nonzero: false, key: [0; EQUALITY_DIGITS] } }
    fn push(&mut self, digit: u8) {
        if self.significant == 0 && digit == b'0' { return; }
        if self.significant < PREFIX_DIGITS { self.prefix = self.prefix * 10 + u64::from(digit - b'0'); }
        else { self.discarded_nonzero |= digit != b'0'; }
        if self.significant < EQUALITY_DIGITS { self.key[self.significant] = digit; }
        self.significant += 1;
        self.trailing_zeroes = if digit == b'0' { self.trailing_zeroes + 1 } else { 0 };
    }
}
fn integer_lower(value: u64) -> f64 {
    let rounded = value as f64;
    if (rounded as u64) > value { rounded.next_down() } else { rounded }
}
fn integer_upper(value: u64) -> f64 {
    let rounded = value as f64;
    if (rounded as u64) < value { rounded.next_up() } else { rounded }
}

fn parse_decimal(original: &str) -> Result<ParsedDecimal, ScalarError> {
    let bytes = original.as_bytes();
    let mut index = 0;
    let negative = bytes.first() == Some(&b'-');
    if matches!(bytes.first(), Some(b'+' | b'-')) { index += 1; }
    let mut digits = Digits::new();
    let integer_start = index;
    while index < bytes.len() && bytes[index].is_ascii_digit() { digits.push(bytes[index]); index += 1; }
    let integer_count = index - integer_start;
    let mut fractional_count = 0;
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        let start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() { digits.push(bytes[index]); index += 1; }
        fractional_count = index - start;
        if fractional_count == 0 { return Err(ScalarError::InvalidDecimal); }
    }
    if integer_count == 0 && fractional_count == 0 { return Err(ScalarError::InvalidDecimal); }
    let mut exponent = 0_i128;
    let mut saturated = false;
    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        index += 1;
        let exponent_negative = bytes.get(index) == Some(&b'-');
        if matches!(bytes.get(index), Some(b'+' | b'-')) { index += 1; }
        let start = index;
        // Any exponent larger than token length+2048 cannot be cancelled by
        // the decimal point or omitted digits. Saturation never grants equality.
        let cap = original.len() as u128 + 2048;
        let mut magnitude = 0_u128;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            let next = magnitude.saturating_mul(10).saturating_add(u128::from(bytes[index] - b'0'));
            saturated |= next > cap;
            magnitude = next.min(cap);
            index += 1;
        }
        if index == start { return Err(ScalarError::InvalidDecimal); }
        exponent = magnitude as i128 * if exponent_negative { -1 } else { 1 };
    }
    if index != bytes.len() { return Err(ScalarError::InvalidDecimal); }
    if digits.significant == 0 {
        return Ok(ParsedDecimal { bounds: IdealBounds::point(0.), sign: Sign::Zero,
            identity: Identity::Dyadic(normalized_dyadic(false, 0, 0)) });
    }
    let retained = digits.significant.min(PREFIX_DIGITS);
    let scale = exponent - fractional_count as i128 + (digits.significant - retained) as i128;
    let sign = if negative { Sign::Negative } else { Sign::Positive };
    let mut bounds = if scale < -400 {
        // prefix <=1e18 and scale <=-401 imply magnitude <1e-383,
        // strictly below the smallest positive binary64 subnormal.
        IdealBounds { lower: 0., upper: MIN_F64 }
    } else {
        if scale > 400 { return Err(ScalarError::IdealRangeOverflow); }
        let mut value = IdealBounds { lower: integer_lower(digits.prefix),
            upper: integer_upper(digits.prefix + u64::from(digits.discarded_nonzero)) };
        // Scale the prefix itself: this avoids underflowing a standalone tiny
        // power of ten before multiplication by a large significant prefix.
        if scale >= 0 {
            for _ in 0..scale { value = value.multiply(IdealBounds::point(10.))?; }
        } else {
            for _ in scale..0 { value = value.divide_positive(10.)?; }
        }
        value
    };
    if negative { bounds = bounds.negate(); }
    let canonical_count = digits.significant - digits.trailing_zeroes;
    let identity = if !saturated && canonical_count <= EQUALITY_DIGITS {
        decimal_identity(negative, &digits.key[..canonical_count], exponent - fractional_count as i128 + digits.trailing_zeroes as i128)
    } else { Identity::Opaque };
    Ok(ParsedDecimal { bounds, sign, identity })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn scalar(text: &str, native: f32) -> ScalarProvenance { ScalarProvenance::from_decimal(text, native).unwrap() }
    fn contains(bounds: IdealBounds, value: f64) { assert!(bounds.lower() <= value && value <= bounds.upper(), "{bounds:?} does not enclose {value}"); }

    #[test]
    fn missing_or_invalid_original_never_becomes_exact_native() {
        assert_eq!(ScalarProvenance::from_optional_decimal(None, 1.).unwrap_err(), ScalarError::MissingOriginal);
        for text in ["", " ", "1px", "1.", ".", "+", "1e", "1e-", "1 2", "NaN", "inf", "0x1", "--0"] {
            assert_eq!(ScalarProvenance::from_decimal(text, 0.).unwrap_err(), ScalarError::InvalidDecimal, "{text}");
        }
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(ScalarProvenance::from_decimal("1", value).unwrap_err(), ScalarError::NonFiniteNative);
            assert_eq!(ScalarProvenance::exact_constant(value).unwrap_err(), ScalarError::NonFiniteNative);
            assert_eq!(IdealBounds::new(0., 1.).unwrap().error_from(value), f64::INFINITY);
        }
    }
    #[test]
    fn normalization_can_exceed_half_an_ulp_and_native_bits_stay_unchanged() {
        let value = scalar("100.71428680419922", 100.714_f32);
        assert_eq!(value.native().to_bits(), 100.714_f32.to_bits());
        contains(value.ideal_bounds(), 100.71428680419922);
        assert!(value.absolute_error_upper() > 0.000289);
        assert!(value.absolute_error_upper() < 0.000291);
        let normalized_again = value.with_native(100.713_f32).unwrap();
        assert_eq!(normalized_again.native().to_bits(), 100.713_f32.to_bits());
        assert_eq!(value.ideal_bounds(), normalized_again.ideal_bounds());
        assert!(value.proves_equal(&normalized_again));
    }
    #[test]
    fn canonical_decimal_equality_is_stronger_than_native_equality() {
        let a = scalar(".1", 0.1);
        for text in ["+0.10", "00100e-3", "1e-1", "0.1000000000000000000000"] { assert!(a.proves_equal(&scalar(text, 0.1))); }
        let distinct = scalar("0.1000000001", 0.1);
        assert_eq!(a.native().to_bits(), distinct.native().to_bits());
        assert!(!a.proves_equal(&distinct));
        assert!(scalar("0001.000e0", 1.).proves_equal(&ScalarProvenance::exact_constant(1.).unwrap()));
        assert!(scalar(".125", 0.125).proves_equal(&ScalarProvenance::exact_constant(0.125).unwrap()));
        assert!(!a.proves_equal(&ScalarProvenance::exact_constant(0.1).unwrap()));
    }
    #[test]
    fn tiny_nonzero_and_negative_zero_remain_distinct() {
        for text in ["0", "-0", "-0.000e999999999999", "+00e-99999999999"] {
            let zero = scalar(text, -0.);
            assert!(zero.is_exact_zero() && zero.is_nonnegative());
            assert_eq!(zero.native().to_bits(), (-0_f32).to_bits());
        }
        let tiny = scalar("1e-99999999999999999999999", 0.);
        assert!(!tiny.is_exact_zero()); assert!(tiny.is_nonnegative());
        assert_eq!(tiny.ideal_bounds().lower(), 0.); assert_eq!(tiny.ideal_bounds().upper(), MIN_F64);
        let negative = scalar("-1e-99999999999999999999999", -0.);
        assert!(!negative.is_nonnegative()); assert!(!negative.is_exact_zero());
        assert_eq!(negative.ideal_bounds().lower(), -MIN_F64);
        assert!(!tiny.proves_equal(&scalar("1e-99999999999999999999998", 0.)));
    }
    #[test]
    fn long_digits_have_bounded_storage_and_do_not_overstate_equality() {
        let mut text = String::from("0."); text.push_str(&"0".repeat(20000)); text.push('1');
        let tiny = scalar(&text, 0.); assert!(!tiny.is_exact_zero());
        assert_eq!(tiny.ideal_bounds().upper(), MIN_F64);
        let text = format!("0.{}", "123456789".repeat(1000));
        let a = scalar(&text, 0.12345679); let b = scalar(&text, 0.12345679);
        assert!(!a.proves_equal(&b)); assert!(a.proves_equal(&a.clone()));
        contains(a.ideal_bounds(), 0.12345678912345679);
        assert_eq!(ScalarProvenance::from_decimal("1e999999999999999999999999", 0.).unwrap_err(), ScalarError::IdealRangeOverflow);
    }
    #[test]
    fn font_chain_matches_public_native_control_and_exposes_accumulated_error() {
        let coefficient = scalar("1.1", 1.1_f32);
        let mut font = ScalarProvenance::exact_constant(16.).unwrap();
        for _ in 0..100 { font = coefficient.multiply(&font).unwrap(); }
        assert_eq!(font.native().to_bits(), 220490.171875_f32.to_bits());
        contains(font.ideal_bounds(), 220489.79743715632);
        assert!(font.absolute_error_upper() > 0.37443784 && font.absolute_error_upper() < 0.374438);
        assert!(font.proves_equal(&font.clone()));
    }
    #[test]
    fn percent_paths_keep_the_native_operation_order() {
        let percent = scalar("33.333333", 33.333332_f32);
        let owner = scalar("123.456", 123.456_f32);
        let preferred = percent.preferred_percent(&owner).unwrap();
        let font = percent.font_percent(&owner).unwrap();
        assert_eq!(preferred.native().to_bits(), ((percent.native()*owner.native())*0.01_f32).to_bits());
        assert_eq!(font.native().to_bits(), ((percent.native()/100_f32)*owner.native()).to_bits());
        contains(preferred.ideal_bounds(), 33.333333*123.456/100.);
        contains(font.ideal_bounds(), 33.333333*123.456/100.);
        // Inherited computed scalar copies keep their existing meaning.
        assert!(preferred.proves_equal(&preferred.clone()));
        assert!(!preferred.proves_equal(&font)); // unknown, not asserted unequal
    }
    #[test]
    fn intermediate_native_overflow_cannot_be_hidden_by_later_percentage_scale() {
        let huge = ScalarProvenance::exact_constant(f32::MAX).unwrap();
        let two = ScalarProvenance::exact_constant(2.).unwrap();
        assert_eq!(huge.preferred_percent(&two).unwrap_err(), ScalarError::NonFiniteNative);
        assert_eq!(huge.multiply(&two).unwrap_err(), ScalarError::NonFiniteNative);
        assert_eq!(huge.native(), f32::MAX);
    }
    #[test]
    fn ideal_bounds_validate_and_round_outward_for_both_signs() {
        assert_eq!(IdealBounds::new(2., 1.).unwrap_err(), ScalarError::InvalidInterval);
        assert_eq!(IdealBounds::new(f64::NAN, 1.).unwrap_err(), ScalarError::InvalidInterval);
        let a = IdealBounds::new(-3., 2.).unwrap(); let b = IdealBounds::new(-4., 5.).unwrap();
        let result = a.multiply(b).unwrap(); assert!(result.lower() <= -15. && result.upper() >= 12.);
        assert_eq!(a.divide_positive(0.).unwrap_err(), ScalarError::InvalidDivisor);
        contains(a.divide_positive(10.).unwrap(), -0.3);
        let tiny = IdealBounds::new(0., MIN_F64).unwrap().multiply(IdealBounds::new(0., MIN_F64).unwrap()).unwrap();
        assert_eq!(tiny.lower(), 0.); assert!(tiny.upper() > 0.);
    }
}
