//! Conditional numerical proof for ordinary, single-pass flex participants.
//!
//! This private analyzer neither discovers structural premises nor admits CSS.
//! Callers must supply the actual emitted topology's premises, scalar provenance,
//! and independently proved parent/world envelopes. Missing proof is unresolved.
//! Bounds describe exact CSS arithmetic versus immutable native f32 operations,
//! not browser pixels. No freeze-dependent profile is approximated as one pass.
use super::scalar_provenance::ScalarProvenance;

const U: f64 = 1.0 / 16_777_216.0;
const TINY: f64 = f32::from_bits(1) as f64;
const NORMAL: f64 = f32::MIN_POSITIVE as f64;
const MAX: f64 = f32::MAX as f64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Premise {
    ZeroInsetsAndGaps,
    NoWrap,
    NoAutoMargins,
    NoIntrinsicSizing,
    NoAspectRatio,
    NoConstraintsOrAnimation,
    PureTranslation,
    KnownTargetPreserved,
    CrossStartDefinite,
    LeftToRight,
    IdentityOwnTransform,
    ZeroOrigins,
    PhysicalFlexStart,
    NativeReverseFlow,
    ActualParticipantOrder,
    DescendantGeometryAccounted,
}
const REQUIRED: [Premise; 16] = [
    Premise::ZeroInsetsAndGaps, Premise::NoWrap, Premise::NoAutoMargins,
    Premise::NoIntrinsicSizing, Premise::NoAspectRatio,
    Premise::NoConstraintsOrAnimation, Premise::PureTranslation,
    Premise::KnownTargetPreserved, Premise::CrossStartDefinite,
    Premise::LeftToRight, Premise::IdentityOwnTransform, Premise::ZeroOrigins,
    Premise::PhysicalFlexStart, Premise::NativeReverseFlow,
    Premise::ActualParticipantOrder, Premise::DescendantGeometryAccounted,
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Unresolved {
    MissingPremise(Premise),
    InvalidEnvelope,
    PossibleNativeOverflow,
    DivisorNotPositive,
    NegativeParent,
    NegativePercentage,
    InvalidBudget,
    EmptyGroup,
    CountNotRepresentable,
    AggregateUncertainty,
    NegativeScalar { item: usize },
    ZeroClassification { item: usize },
    UnsupportedFactorMapping { item: usize },
    MixedZeroBasisShrink,
    InitialFreeze { item: usize },
    PossibleFreeze { item: usize },
    GeometryBudget,
}
type Proof<T> = Result<T, Unresolved>;

/// Encloses ideal values and bounds absolute native-minus-ideal error.
/// Construction requires explicit error; zero is never supplied implicitly.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ErrorEnvelope {
    lo: f64,
    hi: f64,
    error: f64,
}
impl ErrorEnvelope {
    pub(crate) fn new(lo: f64, hi: f64, error: f64) -> Proof<Self> {
        let result = Self { lo, hi, error };
        result.validate()?;
        Ok(result)
    }
    pub(crate) fn lower(self) -> f64 { self.lo }
    pub(crate) fn upper(self) -> f64 { self.hi }
    pub(crate) fn error_upper(self) -> f64 { self.error }
    /// Resolve a preferred/min/max percentage over an entire owner domain.
    /// `percent` retains the authored coefficient (50 means 50%, not 0.5).
    /// The immutable expression is fl(fl(percent * owner) * 0.01f32),
    /// whereas ideal CSS arithmetic divides by exactly100. The literal0.01
    /// therefore carries its own representation error. This is not the
    /// padding/font percentage path, which divides before multiplication.
    pub(crate) fn preferred_percent(self, percent: &ScalarProvenance) -> Proof<Self> {
        self.validate()?;
        if self.lo < 0. { return Err(Unresolved::NegativeParent); }
        if !percent.is_nonnegative() || percent.native() < 0. {
            return Err(Unresolved::NegativePercentage);
        }
        let coefficient = Self::scalar(percent)?;
        let scale = ScalarProvenance::from_decimal("0.01", 0.01_f32)
            .map_err(|_| Unresolved::InvalidEnvelope)?;
        // Validate the first product before applying the small multiplier:
        // a finite final ideal value cannot hide native intermediate overflow.
        coefficient.mul(self)?.mul(Self::scalar(&scale)?)
    }
    fn zero() -> Self { Self { lo: 0., hi: 0., error: 0. } }
    fn is_zero(self) -> bool { self.lo == 0. && self.hi == 0. && self.error == 0. }
    fn magnitude(self) -> f64 { self.lo.abs().max(self.hi.abs()) }
    fn validate(self) -> Proof<()> {
        if !self.lo.is_finite() || !self.hi.is_finite() || !self.error.is_finite()
            || self.lo > self.hi || self.error < 0. {
            return Err(Unresolved::InvalidEnvelope);
        }
        if add_up(self.magnitude(), self.error) > MAX {
            return Err(Unresolved::PossibleNativeOverflow);
        }
        Ok(())
    }
    fn scalar(value: &ScalarProvenance) -> Proof<Self> {
        let ideal = value.ideal_bounds();
        Self::new(ideal.lower(), ideal.upper(), value.absolute_error_upper())
    }
    fn rounded(lo: f64, hi: f64, propagated: f64) -> Proof<Self> {
        let magnitude = lo.abs().max(hi.abs());
        let rounding = add_up(mul_up(U, add_up(magnitude, propagated)), TINY);
        Self::new(lo, hi, add_up(propagated, rounding))
    }
    pub(crate) fn add(self, rhs: Self) -> Proof<Self> {
        // Native finite addition to exact zero does not round.
        if self.is_zero() { return Ok(rhs); }
        if rhs.is_zero() { return Ok(self); }
        Self::rounded(add_down(self.lo, rhs.lo), add_up(self.hi, rhs.hi),
            add_up(self.error, rhs.error))
    }
    fn neg(self) -> Self { Self { lo: -self.hi, hi: -self.lo, error: self.error } }
    pub(crate) fn sub(self, rhs: Self) -> Proof<Self> { self.add(rhs.neg()) }
    fn mul(self, rhs: Self) -> Proof<Self> {
        if self.is_zero() || rhs.is_zero() { return Ok(Self::zero()); }
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for a in [self.lo, self.hi] {
            for b in [rhs.lo, rhs.hi] {
                lo = lo.min(mul_down(a, b));
                hi = hi.max(mul_up(a, b));
            }
        }
        let error = add_up(add_up(mul_up(self.magnitude(), rhs.error),
            mul_up(rhs.magnitude(), self.error)), mul_up(self.error, rhs.error));
        Self::rounded(lo, hi, error)
    }
    fn div(self, rhs: Self) -> Proof<Self> {
        let denominator = sub_down(rhs.lo, rhs.error);
        if denominator <= 0. { return Err(Unresolved::DivisorNotPositive); }
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for a in [self.lo, self.hi] {
            for b in [rhs.lo, rhs.hi] {
                lo = lo.min(div_down(a, b));
                hi = hi.max(div_up(a, b));
            }
        }
        let error = div_up(add_up(self.error,
            div_up(mul_up(self.magnitude(), rhs.error), rhs.lo)), denominator);
        Self::rounded(lo, hi, error)
    }
    fn clamp(self, lower: Option<f64>, upper: Option<f64>) -> Self {
        let clamp = |mut x: f64| {
            if let Some(hi) = upper { x = x.min(hi); }
            if let Some(lo) = lower { x = x.max(lo); }
            x
        };
        Self { lo: clamp(self.lo), hi: clamp(self.hi), error: self.error }
    }
    fn extra(self, error: f64) -> Proof<Self> {
        Self::new(self.lo, self.hi, add_up(self.error, error))
    }
}

// Each nontrivial binary64 operation is directed outward individually. Do not
// combine these expressions: the bookkeeping itself must never round inward.
fn add_up(a: f64, b: f64) -> f64 {
    if a == 0. { b } else if b == 0. { a } else { (a + b).next_up() }
}
fn add_down(a: f64, b: f64) -> f64 {
    if a == 0. { b } else if b == 0. { a } else { (a + b).next_down() }
}
fn sub_down(a: f64, b: f64) -> f64 { add_down(a, -b) }
fn mul_up(a: f64, b: f64) -> f64 {
    if a == 0. || b == 0. { 0. } else { (a * b).next_up() }
}
fn mul_down(a: f64, b: f64) -> f64 {
    if a == 0. || b == 0. { 0. } else { (a * b).next_down() }
}
fn div_up(a: f64, b: f64) -> f64 {
    if a == 0. { 0. } else { (a / b).next_up() }
}
fn div_down(a: f64, b: f64) -> f64 {
    if a == 0. { 0. } else { (a / b).next_down() }
}
fn sum(values: impl IntoIterator<Item = ErrorEnvelope>) -> Proof<ErrorEnvelope> {
    values.into_iter().try_fold(ErrorEnvelope::zero(), |total, x| total.add(x))
}

/// Logical CSS order after stable order sorting. Native coefficients must be
/// actual emitted values. For zero-basis F2 items the authored shrink is allowed
/// to differ; only that irrelevant shrink operation is eliminated below.
#[derive(Clone, Debug)]
pub(crate) struct Item {
    pub(crate) basis: ScalarProvenance,
    pub(crate) grow: ScalarProvenance,
    pub(crate) shrink: ScalarProvenance,
    pub(crate) min: ScalarProvenance,
    pub(crate) max: Option<ScalarProvenance>,
}
pub(crate) struct GroupDescriptor<'a> {
    pub(crate) premises: &'a [Premise],
    pub(crate) parent: ErrorEnvelope,
    pub(crate) parent_world: ErrorEnvelope,
    pub(crate) reverse: bool,
    pub(crate) items: &'a [Item],
    pub(crate) geometry_budget: f64,
    pub(crate) additional_geometry_error: f64,
}
#[derive(Debug)]
pub(crate) struct Analysis {
    pub(crate) sizes: Vec<ErrorEnvelope>,
    pub(crate) positions: Vec<ErrorEnvelope>,
    pub(crate) file_order: Vec<usize>,
    pub(crate) traversal: Vec<usize>,
    pub(crate) join_error_upper: f64,
    pub(crate) max_error_upper: f64,
}

fn ideal_le(a: &ScalarProvenance, b: &ScalarProvenance) -> bool {
    a.proves_equal(b) || a.ideal_bounds().upper() <= b.ideal_bounds().lower()
}

pub(crate) fn analyze(group: &GroupDescriptor<'_>) -> Proof<Analysis> {
    for premise in REQUIRED {
        if !group.premises.contains(&premise) { return Err(Unresolved::MissingPremise(premise)); }
    }
    group.parent.validate()?;
    group.parent_world.validate()?;
    if group.parent.lo < 0. { return Err(Unresolved::NegativeParent); }
    if !group.geometry_budget.is_finite() || group.geometry_budget < 0.
        || !group.additional_geometry_error.is_finite() || group.additional_geometry_error < 0. {
        return Err(Unresolved::InvalidBudget);
    }
    let n = group.items.len();
    if n == 0 { return Err(Unresolved::EmptyGroup); }
    // This is an arithmetic uncertainty check, not an arbitrary item cap.
    // Conversion of count is exact below 2^24; gamma is unavailable at/above it.
    if n >= 16_777_216 { return Err(Unresolved::CountNotRepresentable); }
    let nu = mul_up(n as f64, U);
    let gamma = div_up(nu, sub_down(1., nu));
    if mul_up(gamma, group.parent.magnitude()) > group.geometry_budget {
        return Err(Unresolved::AggregateUncertainty);
    }
    let mut bases = Vec::with_capacity(n);
    let mut grows = Vec::with_capacity(n);
    let mut shrinks = Vec::with_capacity(n);
    let mut active = Vec::with_capacity(n);
    let mut unequal_zero = false;
    let mut positive_shrink = false;
    for (index, item) in group.items.iter().enumerate() {
        for value in [&item.basis, &item.grow, &item.shrink, &item.min]
            .into_iter().chain(item.max.iter()) {
            if !value.is_nonnegative() || value.native() < 0. {
                return Err(Unresolved::NegativeScalar { item: index });
            }
            ErrorEnvelope::scalar(value)?;
        }
        // A zero-basis item's shrink disappears from the scaled denominator.
        // Its authored zero/nonzero classification is immaterial only under
        // the whole-group check below. Basis and grow must always agree.
        for value in [&item.basis, &item.grow].into_iter().chain(
            (!item.basis.is_exact_zero()).then_some(&item.shrink)) {
            if value.is_exact_zero() != (value.native() == 0.) {
                return Err(Unresolved::ZeroClassification { item: index });
            }
        }
        if item.basis.native() < item.min.native()
            || item.max.as_ref().is_some_and(|max| item.basis.native() > max.native())
            || !ideal_le(&item.min, &item.basis)
            || item.max.as_ref().is_some_and(|max| !ideal_le(&item.basis, max)) {
            return Err(Unresolved::InitialFreeze { item: index });
        }
        let equal_factors = item.grow.proves_equal(&item.shrink)
            && item.grow.native() == item.shrink.native();
        if item.basis.is_exact_zero() { unequal_zero |= !equal_factors; }
        else {
            if !equal_factors { return Err(Unresolved::UnsupportedFactorMapping { item: index }); }
            positive_shrink |= !item.shrink.is_exact_zero();
        }
        bases.push(ErrorEnvelope::scalar(&item.basis)?);
        grows.push(ErrorEnvelope::scalar(&item.grow)?);
        shrinks.push(ErrorEnvelope::scalar(if item.basis.is_exact_zero() { &item.grow } else { &item.shrink })?);
        active.push(!item.grow.is_exact_zero() || !item.shrink.is_exact_zero());
    }
    if unequal_zero && positive_shrink { return Err(Unresolved::MixedZeroBasisShrink); }
    let file_order: Vec<_> = if group.reverse { (0..n).collect() } else { (0..n).rev().collect() };
    let traversal: Vec<_> = file_order.iter().copied().rev().collect();
    let used = sum(file_order.iter().map(|&i| bases[i]))?;
    let free = group.parent.sub(used)?;
    let unfrozen: Vec<_> = file_order.iter().copied().filter(|&i| active[i]).collect();
    let mut branches = Vec::with_capacity(2);
    for growing in [false, true] {
        let factors = if growing { &grows } else { &shrinks };
        let factor_sum = sum(unfrozen.iter().map(|&i| factors[i]))?;
        let mut remaining = if growing { free.clamp(Some(0.), None) } else { free.clamp(None, Some(0.)) };
        let mut targets = bases.clone();
        let mut nonnegative = vec![true; n];
        if factor_sum.hi != 0. {
            remaining = remaining.mul(factor_sum.clamp(None, Some(1.)))?;
            let scaled: Vec<_> = (0..n).map(|i| bases[i].mul(shrinks[i])).collect::<Proof<_>>()?;
            let denominator = if growing { factor_sum } else { sum(unfrozen.iter().map(|&i| scaled[i]))? };
            if denominator.hi != 0. {
                for &i in &unfrozen {
                    // Preserve the native difference in grow versus shrink
                    // parentheses; multiplication/division are not reassociated.
                    let term = if growing { remaining.div(denominator)?.mul(grows[i])? }
                        else { remaining.mul(scaled[i].div(denominator)?)? };
                    targets[i] = bases[i].add(term)?.extra(2. * NORMAL)?;
                    nonnegative[i] = growing || term.is_zero();
                }
            }
        }
        for (i, target) in targets.iter().enumerate() {
            let item = &group.items[i];
            let min = item.min.ideal_bounds();
            let lower_ok = (item.min.is_exact_zero() && item.min.native() == 0. && nonnegative[i])
                || (target.lo >= min.upper() && sub_down(target.lo, target.error) >= f64::from(item.min.native()));
            let upper_ok = item.max.as_ref().is_none_or(|max| {
                target.hi <= max.ideal_bounds().lower()
                    && add_up(target.hi, target.error) <= f64::from(max.native())
            });
            if !lower_ok || !upper_ok { return Err(Unresolved::PossibleFreeze { item: i }); }
        }
        branches.push(targets);
    }
    let join_error_upper = mul_up(2., add_up(group.parent.error, used.error));
    let sizes: Vec<_> = (0..n).map(|i| ErrorEnvelope::new(
        branches[0][i].lo.min(branches[1][i].lo), branches[0][i].hi.max(branches[1][i].hi),
        add_up(branches[0][i].error.max(branches[1][i].error), join_error_upper))).collect::<Proof<_>>()?;
    let occupied = sum(file_order.iter().map(|&i| sizes[i]))?;
    let remaining = group.parent.sub(occupied)?;
    let first = if group.reverse { remaining } else { ErrorEnvelope::zero() };
    let mut total = ErrorEnvelope::zero();
    let mut positions = vec![ErrorEnvelope::zero(); n];
    for (index, &i) in traversal.iter().enumerate() {
        let offset = if index == 0 { first } else { ErrorEnvelope::zero() };
        let local = total.add(offset)?;
        positions[i] = group.parent_world.add(local)?;
        total = total.add(offset.add(sizes[i])?)?;
    }
    let max_error_upper = sizes.iter().chain(&positions).fold(group.additional_geometry_error,
        |error, x| error.max(x.error));
    if max_error_upper > group.geometry_budget { return Err(Unresolved::GeometryBudget); }
    Ok(Analysis { sizes, positions, file_order, traversal, join_error_upper, max_error_upper })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scalar(value: f32) -> ScalarProvenance { ScalarProvenance::exact_constant(value).unwrap() }
    fn item(basis: f32, factor: f32) -> Item {
        Item { basis: scalar(basis), grow: scalar(factor), shrink: scalar(factor),
            min: scalar(0.), max: None }
    }
    fn group(items: &[Item]) -> GroupDescriptor<'_> {
        GroupDescriptor { premises: &REQUIRED, parent: ErrorEnvelope::new(0., 8192., 0.002).unwrap(),
            parent_world: ErrorEnvelope::new(0., 0., 0.).unwrap(), reverse: false,
            items, geometry_budget: 0.1, additional_geometry_error: 0. }
    }

    #[test]
    fn explicit_premises_and_ancestor_error_are_required() {
        let items = [item(60., 0.125), item(40., 0.125)];
        let mut descriptor = group(&items);
        assert!(analyze(&descriptor).is_ok());
        for missing in REQUIRED {
            let remaining: Vec<_> = REQUIRED.into_iter().filter(|p| *p != missing).collect();
            let missing_group = GroupDescriptor { premises: &remaining, ..group(&items) };
            assert_eq!(analyze(&missing_group).unwrap_err(), Unresolved::MissingPremise(missing));
        }
        descriptor.additional_geometry_error = 0.100001;
        assert_eq!(analyze(&descriptor).unwrap_err(), Unresolved::GeometryBudget);
        descriptor.additional_geometry_error = f64::NAN;
        assert_eq!(analyze(&descriptor).unwrap_err(), Unresolved::InvalidBudget);
    }

    #[test]
    fn ideal_equality_is_not_inferred_from_equal_native_floats() {
        let mut items = [item(60., 0.125)];
        items[0].shrink = ScalarProvenance::from_decimal("0.1250000001", 0.125).unwrap();
        assert_eq!(analyze(&group(&items)).unwrap_err(), Unresolved::UnsupportedFactorMapping { item: 0 });
    }

    #[test]
    fn tiny_nonzero_basis_cannot_be_classified_as_zero() {
        let mut items = [item(0., 1.)];
        items[0].basis = ScalarProvenance::from_decimal("1e-9999", 0.).unwrap();
        assert_eq!(analyze(&group(&items)).unwrap_err(), Unresolved::ZeroClassification { item: 0 });
    }

    #[test]
    fn zero_basis_shrink_elimination_requires_the_whole_group() {
        let mut zero = item(0., 1.);
        // The immutable fill fraction supplied the actual native shrink1,
        // while authored CSS shrink0 is immaterial to a zero scaled basis.
        zero.shrink = ScalarProvenance::from_decimal("0", 1.).unwrap();
        assert!(analyze(&group(&[zero.clone()])).is_ok());
        assert_eq!(analyze(&group(&[zero, item(60., 0.125)])).unwrap_err(), Unresolved::MixedZeroBasisShrink);
    }

    #[test]
    fn freeze_uncertainty_stays_unresolved() {
        let items = [item(60., 1.), item(40., 1.)];
        assert!(matches!(analyze(&group(&items)), Err(Unresolved::PossibleFreeze { .. })));
        let mut bounded = [item(60., 0.125)];
        bounded[0].min = scalar(61.);
        assert_eq!(analyze(&group(&bounded)).unwrap_err(), Unresolved::InitialFreeze { item: 0 });
    }

    #[test]
    fn actual_file_order_and_native_traversal_are_reported() {
        let items = [item(60., 0.125), item(40., 0.125), item(30., 0.)];
        let mut descriptor = group(&items);
        let ordinary = analyze(&descriptor).unwrap();
        assert_eq!(ordinary.file_order, [2, 1, 0]);
        assert_eq!(ordinary.traversal, [0, 1, 2]);
        descriptor.reverse = true;
        let reverse = analyze(&descriptor).unwrap();
        assert_eq!(reverse.file_order, [0, 1, 2]);
        assert_eq!(reverse.traversal, [2, 1, 0]);
        assert!(reverse.max_error_upper >= ordinary.max_error_upper);
    }

    #[test]
    fn nonfinite_or_overflowing_envelopes_fail_closed() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(ErrorEnvelope::new(0., value, 0.).is_err());
            assert!(ErrorEnvelope::new(0., 1., value).is_err());
        }
        assert!(ErrorEnvelope::new(1., 0., 0.).is_err());
        assert!(ErrorEnvelope::new(0., 1., -1.).is_err());
        let max = ErrorEnvelope::new(MAX, MAX, 0.).unwrap();
        assert_eq!(max.add(max).unwrap_err(), Unresolved::PossibleNativeOverflow);
        let bad_divisor = ErrorEnvelope::new(1., 2., 1.).unwrap();
        assert_eq!(max.div(bad_divisor).unwrap_err(), Unresolved::DivisorNotPositive);
    }

    #[test]
    fn zero_shortcuts_and_world_error_preserve_their_meaning() {
        let zero = ErrorEnvelope::zero();
        let value = ErrorEnvelope::new(1., 2., 0.01).unwrap();
        assert_eq!(zero.add(value).unwrap().error, value.error);
        assert!(zero.mul(value).unwrap().is_zero());
        let items = [item(60., 0.125), item(40., 0.125)];
        let mut descriptor = group(&items);
        descriptor.parent_world = ErrorEnvelope::new(-5., 10., 0.02).unwrap();
        let result = analyze(&descriptor).unwrap();
        assert!(result.positions.iter().all(|p| p.error >= 0.02));
    }

    #[test]
    fn preferred_percentage_encloses_the_whole_owner_domain() {
        let owner = ErrorEnvelope::new(0., 16384., 0.001).unwrap();
        let percent = ScalarProvenance::from_decimal("33.333333", 33.333332_f32).unwrap();
        let result = owner.preferred_percent(&percent).unwrap();
        assert!(result.lower() <= 0.);
        assert!(result.upper() >= 16384. * 33.333333 / 100.);
        assert!(result.error_upper() > owner.error_upper() * 0.33333333);
        for native_owner in [0_f32, 1., 100., 8192., 16384.] {
            let native = (percent.native() * native_owner) * 0.01_f32;
            let ideal = f64::from(native_owner) * 33.333333 / 100.;
            assert!((f64::from(native) - ideal).abs() <= result.error_upper());
        }
    }

    #[test]
    fn preferred_percentage_preserves_tiny_ideal_and_scale_error() {
        let owner = ErrorEnvelope::new(1., 16384., 0.).unwrap();
        let tiny = ScalarProvenance::from_decimal("1e-9999", 0.).unwrap();
        let result = owner.preferred_percent(&tiny).unwrap();
        assert!(result.upper() > 0.);
        assert!(result.error_upper() > 0.);
        let half = owner.preferred_percent(&scalar(50.)).unwrap();
        // Fifty and owner are exact; the result still accounts for both native
        // multiplications and the inexact native literal0.01.
        assert!(half.error_upper() > 16384. * 50. * (f64::from(0.01_f32) - 0.01).abs());
        let zero = owner.preferred_percent(&scalar(0.)).unwrap();
        assert!(zero.is_zero());
    }

    #[test]
    fn preferred_percentage_rejects_intermediate_overflow_and_negative_inputs() {
        let owner = ErrorEnvelope::new(MAX / 2., MAX / 2., 0.).unwrap();
        assert_eq!(owner.preferred_percent(&scalar(4.)).unwrap_err(), Unresolved::PossibleNativeOverflow);
        let ordinary = ErrorEnvelope::new(0., 16384., 0.).unwrap();
        assert_eq!(ordinary.preferred_percent(&scalar(-1.)).unwrap_err(), Unresolved::NegativePercentage);
        assert_eq!(ErrorEnvelope::new(-1., 0., 0.).unwrap().preferred_percent(&scalar(1.)).unwrap_err(), Unresolved::NegativeParent);
    }
}
