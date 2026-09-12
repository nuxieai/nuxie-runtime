//! Conditional scalar proof for the ordinary wrapped-line distance gate.
//!
//! The caller must separately bind zero target translation, zero local anchor,
//! an orthogonal zero coordinate, distance 65536, mode exactly, strength one,
//! identity linear transforms, and the following doubling/upper-clamp records.
//! This module proves arithmetic only; it does not admit CSS or inspect records.
use super::wrapping_sizes::MachineInterval;

const D: f32 = 65536.;
const U: f64 = 1. / 16777216.;
const TINY: f64 = f32::from_bits(1) as f64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Unresolved {
    NonPositiveDead,
    SquareNotNormal,
    LengthNotNormal,
    EarlyReturnThreshold,
    QuotientNotNormal,
    ProductNotNormal,
    InterpolationOverflow,
    InsufficientGateSeparation,
    DoublingOverflow,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Proof {
    pub(super) dead: MachineInterval,
    pub(super) square: MachineInterval,
    pub(super) length: MachineInterval,
    pub(super) quotient: MachineInterval,
    /// Correlated bounds, not the product of independent operand intervals.
    pub(super) projected: MachineInterval,
    pub(super) subtraction_error: f64,
    pub(super) interpolation_error: f64,
    pub(super) interpolated: MachineInterval,
}

fn normal(lo: f32, hi: f32, reason: Unresolved) -> Result<MachineInterval, Unresolved> {
    if !lo.is_normal() || !hi.is_normal() || lo <= 0. || lo > hi {
        return Err(reason);
    }
    MachineInterval::new(lo, hi).map_err(|_| reason)
}

// Every bound arithmetic operation rounds outwards, including binary64 work;
// a final binary32 cast alone would not enclose earlier binary64 rounding.
fn add_up(a: f64, b: f64) -> f64 { (a + b).next_up() }
fn mul_up(a: f64, b: f64) -> f64 { (a * b).next_up() }
fn error(magnitude: f64) -> f64 { add_up(mul_up(U, magnitude), TINY) }
fn down32(value: f64) -> f32 {
    let rounded = value as f32;
    if (rounded as f64) > value { rounded.next_down() } else { rounded }
}
fn up32(value: f64) -> f32 {
    let rounded = value as f32;
    if (rounded as f64) < value { rounded.next_up() } else { rounded }
}

/// Enclose all positive machine inputs in `dead`. A zero gate follows the
/// native early-return branch and remains exactly zero under the caller's
/// zero bindings; it is deliberately outside this positive-branch lemma.
pub(super) fn prove(dead: MachineInterval) -> Result<Proof, Unresolved> {
    let lo = dead.lower();
    let hi = dead.upper();
    if lo <= 0. { return Err(Unresolved::NonPositiveDead); }

    // These are the actual monotone binary32 operations, in native order.
    // Square equals x.mul_add(x, 0) or y*y when the other coordinate is zero.
    let square = normal(lo * lo, hi * hi, Unresolved::SquareNotNormal)?;
    let length = normal(square.lower().sqrt(), square.upper().sqrt(), Unresolved::LengthNotNormal)?;
    if length.lower() <= 0.001_f32 { return Err(Unresolved::EarlyReturnThreshold); }
    let quotient = normal(D / length.upper(), D / length.lower(), Unresolved::QuotientNotNormal)?;
    // Independent bounds suffice to establish normal finite multiplication,
    // but are intentionally not used as the useful projected-value enclosure.
    normal(lo * quotient.lower(), hi * quotient.upper(), Unresolved::ProductNotNormal)?;

    // Normal binary32 relative rounding: q=d²(1+δq), l=sqrt(q)(1+δl),
    // t=(D/l)(1+δt), p=d*t(1+δp). Bounding sqrt(1+δq)
    // between 1-u and 1+u yields this conservative correlated enclosure.
    let a = 1. - U; // exact binary64 values
    let b = 1. + U;
    let low = (((D as f64 * a).next_down() * a).next_down()
        / (b * b).next_up()).next_down();
    let high = (((D as f64 * b).next_up() * b).next_up()
        / (a * a).next_down()).next_up();
    let projected = normal(down32(low), up32(high), Unresolved::ProductNotNormal)?;

    // Immutable Vec2D::lerp uses fl(p-d).mul_add(1,d). The subtraction
    // rounding survives even at strength one: it is not an exact copy of p.
    let diff_lo = (projected.lower() as f64 - hi as f64).next_down();
    let diff_hi = (projected.upper() as f64 - lo as f64).next_up();
    let magnitude = diff_lo.abs().max(diff_hi.abs());
    let subtraction_error = error(magnitude);
    let fma_magnitude = add_up(projected.upper() as f64, subtraction_error);
    let interpolation_error = error(fma_magnitude);
    if add_up(magnitude, subtraction_error) > f32::MAX as f64
        || add_up(fma_magnitude, interpolation_error) > f32::MAX as f64 {
        return Err(Unresolved::InterpolationOverflow);
    }
    let z_lo = down32(((projected.lower() as f64 - subtraction_error).next_down()
        - interpolation_error).next_down());
    let z_hi = up32(add_up(fma_magnitude, interpolation_error));
    if z_lo <= D / 2. { return Err(Unresolved::InsufficientGateSeparation); }
    if !(z_hi * 2.).is_finite() { return Err(Unresolved::DoublingOverflow); }
    let interpolated = normal(z_lo, z_hi, Unresolved::InterpolationOverflow)?;
    // Multiplication by two is exact here and >D. The bound-shape upper
    // clamp therefore selects the ordinary exact D value for every input.
    Ok(Proof { dead, square, length, quotient, projected, subtraction_error,
        interpolation_error, interpolated })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn point(value: f32) -> MachineInterval { MachineInterval::new(value, value).unwrap() }
    fn actual(dead: f32) -> (f32, f32) {
        let p = dead * (D / (dead * dead).sqrt());
        (p, (p - dead).mul_add(1., dead))
    }
    fn contains(interval: MachineInterval, value: f32) {
        assert!(interval.lower() <= value && value <= interval.upper(), "{interval:?} excludes {value:?}");
    }

    #[test]
    fn threshold_is_checked_after_native_square_and_sqrt() {
        for value in [0.001_f32.next_down(), 0.001_f32] {
            assert_eq!(prove(point(value)).unwrap_err(), Unresolved::EarlyReturnThreshold);
        }
        let value = 0.001_f32.next_up();
        assert!(prove(point(value)).is_ok());
        assert_eq!(prove(point(0.)).unwrap_err(), Unresolved::NonPositiveDead);
        assert_eq!(prove(point(f32::MIN_POSITIVE)).unwrap_err(), Unresolved::SquareNotNormal);
        assert_eq!(prove(point(f32::MAX)).unwrap_err(), Unresolved::SquareNotNormal);
    }

    #[test]
    fn finite_normalization_does_not_hide_lerp_cancellation() {
        let dead = 2_f32.powi(42);
        let (projected, interpolated) = actual(dead);
        assert_eq!(projected, D);
        assert_eq!(interpolated, 0.);
        assert_eq!(prove(point(dead)).unwrap_err(), Unresolved::InsufficientGateSeparation);
        assert!(prove(point(2_f32.powi(38))).is_ok());
        assert_eq!(prove(point(2_f32.powi(39))).unwrap_err(), Unresolved::InsufficientGateSeparation);
    }

    #[test]
    fn correlated_bounds_cover_wide_ranges_and_rounding_boundaries() {
        let range = MachineInterval::new(0.001_f32.next_up(), 2_f32.powi(38)).unwrap();
        let proof = prove(range).unwrap();
        assert_eq!(proof.dead, range);
        assert!(proof.subtraction_error > 0. && proof.interpolation_error > 0.);
        let mut samples = vec![];
        for center in [range.lower(), 0.5, 1., 31., D / 2., D, D * 2., range.upper()] {
            for bits in center.to_bits().saturating_sub(128)..=center.to_bits() + 128 {
                let d = f32::from_bits(bits);
                if d >= range.lower() && d <= range.upper() { samples.push(d); }
            }
        }
        for exponent in -9..=38 { samples.push(2_f32.powi(exponent)); }
        for d in samples {
            let q = d * d;
            let l = q.sqrt();
            let t = D / l;
            let (p, z) = actual(d);
            contains(proof.square, q);
            contains(proof.length, l);
            contains(proof.quotient, t);
            contains(proof.projected, p);
            contains(proof.interpolated, z);
            assert_eq!((z * 2.).min(D), D);
        }
        // Independent multiplication would discard correlation by many orders.
        assert!(range.upper() * proof.quotient.upper() > proof.projected.upper() * 1e10);
    }
}
