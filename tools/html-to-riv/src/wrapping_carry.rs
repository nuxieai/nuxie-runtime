//! Conditional bounds for the two wrapped-line maximum passes.
//!
//! These are arithmetic facts, not a record certificate or CSS admission. The
//! caller must bind the `wrapping.rs` scalar graph: identity linear transforms,
//! root-zero helpers, exact 0/65536 gates identifying the actual line partition,
//! local nonnegative max clamps and strength-one TranslationConstraints. Slot
//! measurements must enclose the actual bottom-minus-top values; their errors
//! bound distance from the corresponding nonnegative line-layout extents.
use super::wrapping_sizes::MachineInterval;

const D: f64 = 65536.;
const U: f64 = 1. / 16777216.;
const TINY: f64 = f32::from_bits(1) as f64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Unresolved {
    Empty,
    InvalidError,
    ArithmeticOverflow,
    /// A directional pass might enter a different-line reset above 65536.
    ResetRange { index: usize, backward: bool },
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Measured {
    pub(super) machine: MachineInterval,
    pub(super) error: f64,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Bound {
    /// Every actual value is nonnegative and no larger than this endpoint.
    pub(super) upper: f64,
    /// Absolute error against the maximum of the true extents in the
    /// corresponding same-line prefix/suffix (or whole line for final maxima).
    pub(super) error: f64,
}

#[derive(Clone, Debug)]
pub(super) struct Proof {
    pub(super) forward: Vec<Bound>,
    pub(super) backward: Vec<Bound>,
    pub(super) line_maxima: Vec<Bound>,
}

fn add_up(a: f64, b: f64) -> f64 { (a + b).next_up() }
fn error(magnitude: f64) -> f64 { add_up((U * magnitude).next_up(), TINY) }
fn finite(value: f64) -> Result<f64, Unresolved> {
    if !value.is_finite() || value < 0. || value > f32::MAX as f64 {
        Err(Unresolved::ArithmeticOverflow)
    } else { Ok(value) }
}

/// The b-selected branch is fl(fl(b-a)+a), not an exact copy of b.
/// For nonnegative inputs bounded by M, |b-a|<=M. Rounding that subtraction
/// contributes e1; reconstruction has magnitude at most M+e1 and error e2.
/// The a-selected branch copies a. Max is 1-Lipschitz in the infinity norm,
/// so operand errors contribute their maximum, rather than their sum.
fn maximum(a: Bound, b: Bound) -> Result<Bound, Unresolved> {
    let magnitude = a.upper.max(b.upper);
    let e1 = error(magnitude);
    finite(add_up(magnitude, e1))?;
    let e2 = error(add_up(magnitude, e1));
    let local = add_up(e1, e2);
    Ok(Bound {
        upper: finite(add_up(magnitude, local))?,
        error: add_up(a.error.max(b.error), local),
    })
}

fn step(carry: Bound, measured: Bound, index: usize, backward: bool) -> Result<Bound, Unresolved> {
    if carry.upper > D { return Err(Unresolved::ResetRange { index, backward }); }
    // At gate zero diff/relu copy the nonnegative carry exactly. At gate D,
    // fl(carry-D)<=0 and relu returns exact zero: no residue can leak from
    // the preceding line. Its ideal reset value is also zero. The following
    // bound takes the union of both cases; it need not know a line partition.
    maximum(carry, measured)
}

/// Prove all partitions for these logical-order measurements. Count is the
/// slice length; there is no fixture-size specialization or assumed viewport
/// minimum. A failure is an unresolved certificate, not impossibility evidence.
pub(super) fn prove(measured: &[Measured]) -> Result<Proof, Unresolved> {
    if measured.is_empty() { return Err(Unresolved::Empty); }
    let measured = measured.iter().map(|m| {
        if !m.error.is_finite() || m.error < 0. { return Err(Unresolved::InvalidError); }
        Ok(Bound { upper: m.machine.upper() as f64, error: m.error })
    }).collect::<Result<Vec<_>, _>>()?;
    let n = measured.len();
    let mut forward = Vec::with_capacity(n);
    forward.push(measured[0]);
    for i in 1..n { forward.push(step(forward[i-1], measured[i], i-1, false)?); }
    let mut backward = measured.clone();
    for i in (0..n-1).rev() { backward[i] = step(backward[i+1], measured[i], i+1, true)?; }
    let line_maxima = if n == 1 {
        // Both handles identify the same measured node; fl(a-a)=0 and
        // fl(0+a)=a exactly. Do not invent a reconstruction error here.
        measured
    } else {
        forward.iter().zip(&backward).map(|(&a,&b)| maximum(a,b))
            .collect::<Result<Vec<_>, _>>()?
    };
    if line_maxima.iter().any(|b| !b.error.is_finite()) {
        return Err(Unresolved::InvalidError);
    }
    Ok(Proof { forward, backward, line_maxima })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn measured(value: f32, error: f64) -> Measured {
        Measured { machine: MachineInterval::new(value, value).unwrap(), error }
    }
    fn actual_max(a: f32, b: f32) -> f32 { (b-a).max(0.)+a }
    fn check(bound: Bound, actual: f32, ideal: f64) {
        assert!(actual >= 0. && actual as f64 <= bound.upper, "{actual} > {bound:?}");
        assert!((actual as f64-ideal).abs() <= bound.error, "{actual} vs {ideal}: {bound:?}");
    }

    #[test]
    fn reconstruction_overshoot_is_charged() {
        let a = 3. * 2_f32.powi(-24);
        let b = 1. + 3. * 2_f32.powi(-23);
        let actual = actual_max(a,b);
        assert_eq!(actual, 1. + 4. * 2_f32.powi(-23));
        assert!(actual > b);
        let bound = maximum(Bound { upper:a as f64,error:0. }, Bound { upper:b as f64,error:0. }).unwrap();
        check(bound,actual,b as f64);
    }

    #[test]
    fn reset_range_is_checked_on_every_entering_carry() {
        let exact = prove(&[measured(D as f32,0.)]).unwrap();
        assert_eq!(exact.line_maxima[0].upper,D);
        assert_eq!(exact.line_maxima[0].error,0.);
        // There is no subsequent reset of final maxima, so this is admitted
        // conditionally even though the conservative final bound exceeds D.
        let pair = prove(&[measured(D as f32,0.);2]).unwrap();
        assert!(pair.line_maxima[0].upper > D);
        assert_eq!(prove(&[measured(D as f32,0.);3]).unwrap_err(),
            Unresolved::ResetRange { index:1,backward:false });
        assert_eq!(prove(&[measured(1.,0.),measured((D as f32).next_up(),0.)]).unwrap_err(),
            Unresolved::ResetRange { index:1,backward:true });
        let safe = (D as f32)-1.;
        assert!(prove(&[measured(safe,0.);32]).is_ok());
        assert_eq!(((D as f32)-D as f32).max(0.),0.);
        assert!((((D as f32).next_up())-D as f32).max(0.) > 0.);
    }

    #[test]
    fn all_partitions_enclose_native_scalar_recurrence_and_measurement_error() {
        for values in [
            vec![3.*2_f32.powi(-24),1.+3.*2_f32.powi(-23),0.5,7.,2.,0.],
            vec![50.,1.,30.,50.,1.,30.],
            vec![0.,f32::from_bits(1),f32::MIN_POSITIVE,0.,1.,65535.],
        ] {
            let errors: Vec<_> = values.iter().map(|v| (*v as f64)/1024.).collect();
            let ms:Vec<_> = values.iter().zip(&errors).map(|(&v,&e)|measured(v,e)).collect();
            let proof = prove(&ms).unwrap();
            for partition in 0..(1_usize << (values.len()-1)) {
                let n=values.len();
                let mut forward=values.clone(); let mut backward=values.clone();
                let mut ideals:Vec<_>=values.iter().zip(&errors).map(|(&v,&e)|v as f64-e).collect();
                let raw_ideals=ideals.clone(); let mut reverse_ideals=ideals.clone();
                for i in 1..n {
                    let reset=partition & (1 << (i-1)) != 0;
                    let gate=if reset { D as f32 } else { 0. };
                    forward[i]=actual_max((forward[i-1]-gate).max(0.),values[i]);
                    ideals[i]=raw_ideals[i].max(if reset { 0. } else { ideals[i-1] });
                }
                for i in (0..n-1).rev() {
                    let reset=partition & (1 << i) != 0;
                    let gate=if reset { D as f32 } else { 0. };
                    backward[i]=actual_max((backward[i+1]-gate).max(0.),values[i]);
                    reverse_ideals[i]=raw_ideals[i].max(if reset { 0. } else { reverse_ideals[i+1] });
                }
                for i in 0..n {
                    check(proof.forward[i],forward[i],ideals[i]);
                    check(proof.backward[i],backward[i],reverse_ideals[i]);
                    check(proof.line_maxima[i],actual_max(forward[i],backward[i]),ideals[i].max(reverse_ideals[i]));
                }
            }
        }
    }

    #[test]
    fn missing_or_invalid_measurement_premises_reject() {
        assert_eq!(prove(&[]).unwrap_err(),Unresolved::Empty);
        for error in [-1.,f64::NAN,f64::INFINITY] {
            assert_eq!(prove(&[measured(1.,error)]).unwrap_err(),Unresolved::InvalidError);
        }
        assert_eq!(maximum(Bound { upper:f32::MAX as f64,error:0. },Bound { upper:1.,error:0. }).unwrap_err(),Unresolved::ArithmeticOverflow);
    }
}
