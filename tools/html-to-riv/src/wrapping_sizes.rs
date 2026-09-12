//! Machine size bounds for the definite, independent wrapping-slot profile.
//!
//! This proves only dimension resolution. The caller must bind the final slot
//! shape and the unchanged parent content basis separately. No line membership,
//! CSS/native agreement, or public wrapping admission follows from these bounds.
use super::{computed_provenance::NumericSize, flex_descriptor::RecordLength};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Unresolved {
    Domain,
    Metadata,
    NativeBinding,
    AutomaticSize,
    AutomaticMinimum,
    IntermediateOverflow,
    EmptySlots,
    NoPositiveCrossLowerBound,
}

/// Closed enclosure of nonnegative finite binary32 machine values. Zero is a
/// valid endpoint, including for a positive authored percentage that underflows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct MachineInterval {
    lower: f32,
    upper: f32,
}

impl MachineInterval {
    pub(super) fn new(lower: f32, upper: f32) -> Result<Self, Unresolved> {
        if !lower.is_finite() || !upper.is_finite() || lower < 0. || lower > upper {
            return Err(Unresolved::Domain);
        }
        Ok(Self { lower, upper })
    }

    pub(super) fn lower(self) -> f32 { self.lower }
    pub(super) fn upper(self) -> f32 { self.upper }

    fn percentage(self, coefficient: f32) -> Result<Self, Unresolved> {
        // Immutable dimension resolution: util/resolve.rs RIVE_YOGA_PERCENT_TAG.
        // Each multiply is monotone for nonnegative finite inputs. Evaluating
        // the closed machine endpoints in the same f32 order encloses every
        // representable parent size, without adding a guessed error epsilon.
        // Check the first product independently: a later bound cannot justify
        // an overflowing native intermediate, even if its clamp would be finite.
        let lo_product = coefficient * self.lower;
        let hi_product = coefficient * self.upper;
        if !lo_product.is_finite() || !hi_product.is_finite() {
            return Err(Unresolved::IntermediateOverflow);
        }
        Self::new(lo_product * 0.01_f32, hi_product * 0.01_f32)
    }
}

fn absent(raw: &RecordLength) -> bool {
    raw.units.is_none() && raw.value.is_none()
}

fn resolve(
    source: &NumericSize, raw: &RecordLength, parent: MachineInterval,
) -> Result<MachineInterval, Unresolved> {
    let (scalar, units) = match source {
        NumericSize::Pixels(Ok(value)) => (value, 1),
        NumericSize::Percent(Ok(value)) => (value, 2),
        NumericSize::Auto => return Err(Unresolved::AutomaticSize),
        _ => return Err(Unresolved::Metadata),
    };
    let native = scalar.native();
    if !scalar.is_nonnegative() || !native.is_finite() || native < 0. {
        return Err(Unresolved::Metadata);
    }
    if raw.units != Some(units)
        || !raw.value.is_some_and(|value| value.to_bits() == native.to_bits()) {
        return Err(Unresolved::NativeBinding);
    }
    if units == 1 { MachineInterval::new(native, native) }
    else { parent.percentage(native) }
}

/// Authored metadata and the matching final ordinary record values. The three
/// percentages all resolve against `parent`, never against the preferred size
/// or an already clamped wrapper. Explicit minima are required; native automatic
/// minima introduce an intrinsic/flex premise outside this resolver.
pub(super) fn used_size(
    preferred: (&NumericSize, &RecordLength),
    minimum: (&NumericSize, &RecordLength),
    maximum: (&NumericSize, &RecordLength),
    parent: MachineInterval,
) -> Result<MachineInterval, Unresolved> {
    let preferred = resolve(preferred.0, preferred.1, parent)?;
    if matches!(minimum.0, NumericSize::Auto) {
        return Err(Unresolved::AutomaticMinimum);
    }
    let minimum = resolve(minimum.0, minimum.1, parent)?;
    let maximum = if matches!(maximum.0, NumericSize::Auto) {
        if !absent(maximum.1) { return Err(Unresolved::NativeBinding); }
        None
    } else {
        Some(resolve(maximum.0, maximum.1, parent)?)
    };
    // Immutable maybe_clamp selects existing floats, min(maximum) then
    // max(minimum). It adds no rounding, and a conflicting minimum wins.
    // Correlated parent percentages remain enclosed by endpoint monotonicity.
    let clamp = |value: f32, min: f32, max: Option<f32>| {
        max.map_or(value, |max| value.min(max)).max(min)
    };
    MachineInterval::new(
        clamp(preferred.lower, minimum.lower, maximum.map(|v| v.lower)),
        clamp(preferred.upper, minimum.upper, maximum.map(|v| v.upper)),
    )
}

/// Sufficient per-slot envelope for arbitrary nonempty line partitions. This is
/// not a gate certificate: separation, carry errors and masks remain unproved.
pub(super) fn cross_extents(slots: &[MachineInterval]) -> Result<MachineInterval, Unresolved> {
    let first = *slots.first().ok_or(Unresolved::EmptySlots)?;
    let lower = slots.iter().fold(first.lower, |bound, slot| bound.min(slot.lower));
    let upper = slots.iter().fold(first.upper, |bound, slot| bound.max(slot.upper));
    if lower == 0. { return Err(Unresolved::NoPositiveCrossLowerBound); }
    MachineInterval::new(lower, upper)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::scalar_provenance::ScalarProvenance;

    fn value(number: f32, percent: bool) -> (NumericSize, RecordLength) {
        let scalar = Ok(ScalarProvenance::exact_constant(number).unwrap());
        (if percent { NumericSize::Percent(scalar) } else { NumericSize::Pixels(scalar) },
         RecordLength { units: Some(if percent { 2 } else { 1 }), value: Some(number) })
    }
    fn no_max() -> (NumericSize, RecordLength) {
        (NumericSize::Auto, RecordLength { units: None, value: None })
    }
    fn run(p: &(NumericSize, RecordLength), lo: &(NumericSize, RecordLength),
        hi: &(NumericSize, RecordLength), parent: MachineInterval) -> Result<MachineInterval, Unresolved> {
        used_size((&p.0, &p.1), (&lo.0, &lo.1), (&hi.0, &hi.1), parent)
    }

    #[test]
    fn machine_percentage_enclosure_covers_rounding_transitions_and_underflow() {
        // Enumerate consecutive representable parent values around several
        // transitions, across mixed-unit and conflicting min/max combinations.
        for center in [0_u32, 1, 0x007f_ffff, 0x0080_0000, 1_f32.to_bits(),
            31.25_f32.to_bits(), 8192_f32.to_bits(), 16384_f32.to_bits()] {
            let start = center.saturating_sub(32);
            let end = center + 32;
            let parent = MachineInterval::new(f32::from_bits(start), f32::from_bits(end)).unwrap();
            for coefficient in [0., 0.1, 33.333332, 50., 100., 175.] {
                for (minimum, maximum) in [(value(0., false), no_max()),
                    (value(1., false), value(75., true)),
                    (value(80., true), value(2., false)),
                    (value(120., true), value(75., true))] {
                    let preferred = value(coefficient, true);
                    let result = run(&preferred, &minimum, &maximum, parent).unwrap();
                    for bits in start..=end {
                        let owner = f32::from_bits(bits);
                        let native = |v: &RecordLength| if v.units == Some(2) {
                            (v.value.unwrap() * owner) * 0.01_f32
                        } else { v.value.unwrap() };
                        let mut actual = native(&preferred.1);
                        if !absent(&maximum.1) { actual = actual.min(native(&maximum.1)); }
                        actual = actual.max(native(&minimum.1));
                        assert!(result.lower <= actual && actual <= result.upper,
                            "parent={owner:?}, actual={actual:?}, enclosure={result:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn bounds_use_original_parent_and_minimum_wins() {
        let parent = MachineInterval::new(200., 400.).unwrap();
        let result = run(&value(50., true), &value(80., true), &value(100., false), parent).unwrap();
        assert_eq!(result, MachineInterval::new(160., 320.).unwrap());
        let point = run(&value(35., false), &value(50., true), &value(75., true), parent).unwrap();
        assert_eq!(point, MachineInterval::new(100., 200.).unwrap());
    }

    #[test]
    fn overflow_cannot_be_hidden_by_maximum_or_zero_parent_endpoint() {
        let parent = MachineInterval::new(0., f32::MAX).unwrap();
        assert_eq!(run(&value(100., true), &value(0., false), &value(1., false), parent),
            Err(Unresolved::IntermediateOverflow));
        assert_eq!(run(&value(1., false), &value(0., false), &value(100., true), parent),
            Err(Unresolved::IntermediateOverflow));
    }

    #[test]
    fn metadata_cannot_hide_missing_units_changed_values_or_automatic_minima() {
        let parent = MachineInterval::new(0., 16384.).unwrap();
        let minimum = value(0., false);
        let maximum = no_max();
        let mut preferred = value(20., false);
        preferred.1.value = Some(20_f32.next_up());
        assert_eq!(run(&preferred, &minimum, &maximum, parent), Err(Unresolved::NativeBinding));
        preferred = value(20., false);
        preferred.1.units = None;
        assert_eq!(run(&preferred, &minimum, &maximum, parent), Err(Unresolved::NativeBinding));
        assert_eq!(run(&value(20., false), &no_max(), &maximum, parent), Err(Unresolved::AutomaticMinimum));
        assert_eq!(run(&no_max(), &minimum, &maximum, parent), Err(Unresolved::AutomaticSize));
        assert_eq!(run(&value(20., false), &minimum,
            &(NumericSize::Auto, value(10., false).1), parent), Err(Unresolved::NativeBinding));
    }

    #[test]
    fn positive_authored_percentage_does_not_imply_positive_machine_line_extent() {
        let parent = MachineInterval::new(f32::from_bits(1), 8192.).unwrap();
        let percent = run(&value(50., true), &value(0., false), &no_max(), parent).unwrap();
        assert_eq!(percent.lower(), 0.);
        assert_eq!(cross_extents(&[percent, MachineInterval::new(30., 30.).unwrap()]),
            Err(Unresolved::NoPositiveCrossLowerBound));
        let bounded = run(&value(50., true), &value(1., false), &no_max(), parent).unwrap();
        assert_eq!(cross_extents(&[bounded, MachineInterval::new(30., 30.).unwrap()]).unwrap().lower(), 1.);
        assert_eq!(cross_extents(&[]), Err(Unresolved::EmptySlots));
        for (lo, hi) in [(f32::NAN, 1.), (0., f32::INFINITY), (-1., 2.), (2., 1.)] {
            assert_eq!(MachineInterval::new(lo, hi), Err(Unresolved::Domain));
        }
    }
}
