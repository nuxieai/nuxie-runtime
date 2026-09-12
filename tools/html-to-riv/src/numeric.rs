//! Focused exponent guard for resolved definite size chains. Unknown intrinsic
//! measurements and aggregate layout/position arithmetic are not proven here.
use super::{Diagnostic, Size, Style, unsupported};

#[derive(Clone, Copy)]
pub(super) struct Bounds {
    upper: [Option<f32>; 2],
    // Lower witness at the maximum supported viewport. Automatic minima may
    // raise a preferred size, so losing its upper bound must not lose this
    // independent evidence of overflow deeper in a percentage chain.
    witness: [Option<f32>; 2],
}
impl Bounds {
    // This is the documented complete viewport domain, not the input viewport.
    pub const VIEWPORT: Self = Self { upper: [Some(16384.); 2], witness: [Some(16384.); 2] };
    pub fn child(self, style: &Style, parent: &Style, source: &str) -> Result<Self, Diagnostic> {
        let sizes = [style.width, style.height];
        let minima = [style.min_width, style.min_height];
        let maxima = [style.max_width, style.max_height];
        let mut bounds = [None; 2];
        let mut witnesses = [None; 2];
        let parent_main = if parent.direction.is_row() { 0 } else { 1 };
        for axis in 0..2 {
            // Preserve overflow as an upper-bound infinity until native min/max
            // clamping. Native boundary probes establish a finite maximum can
            // cap an overflowing preferred expression; a larger minimum wins.
            let preferred = resolve(sizes[axis], self.upper[axis]);
            let minimum = resolve(minima[axis], self.upper[axis]);
            let maximum = resolve(maxima[axis], self.upper[axis]);
            let used = if axis == parent_main && !style.flex.legacy() {
                // Neither the authored main dimension nor basis is the used
                // flex size. A candidate flex plan needs a separate bound.
                None
            } else if matches!(sizes[axis], Size::Auto) && axis != parent_main
                && style.self_alignment.stretches() && !style.margins.cross(parent.direction) {
                // Ordinary cross-axis stretch uses the parent's definite size.
                self.upper[axis]
            } else { preferred };
            // A local maximum is not a proof for intrinsic measurements. Keep
            // unknown preferred/minimum expressions unknown across descendants.
            let capped_used = used.map(|used| maximum.map_or(used, |maximum| used.min(maximum)));
            bounds[axis] = match (capped_used, minimum) {
                (Some(used), Some(minimum)) => Some(used.max(minimum)),
                _ => None,
            };
            let preferred_witness = resolve_witness(sizes[axis], self.witness[axis]);
            let minimum_witness = resolve_witness(minima[axis], self.witness[axis]);
            let maximum_witness = resolve_witness(maxima[axis], self.witness[axis]);
            let used_witness = if axis == parent_main && !style.flex.legacy() { None }
                else if matches!(sizes[axis], Size::Auto) && axis != parent_main
                    && style.self_alignment.stretches() && !style.margins.cross(parent.direction) { self.witness[axis] }
                else { preferred_witness };
            let capped_witness = match (used_witness, maximum_witness) {
                (Some(used), Some(maximum)) => Some(used.min(maximum)),
                (Some(used), None) if matches!(maxima[axis], Size::Auto) => Some(used),
                _ => None,
            };
            // An automatic minimum cannot reduce a known preferred size. This
            // is only a lower witness: upper remains Unknown above.
            witnesses[axis] = capped_witness.map(|used| used.max(minimum_witness.unwrap_or(0.)));
            if capped_used.is_some_and(|bound| !bound.is_finite()) || minimum.is_some_and(|bound| !bound.is_finite())
                || witnesses[axis].is_some_and(|bound| !bound.is_finite()) || minimum_witness.is_some_and(|bound| !bound.is_finite()) {
                return Err(unsupported(source, "Resolved percentage size may exceed finite binary32 geometry within the supported viewport domain (0,16384], after native min/max clamping"));
            }
        }
        Ok(Self { upper: bounds, witness: witnesses })
    }
}
// For nonnegative arithmetic a correctly rounded operation is monotonic.
// Round each real product upward so the propagated binary32 value never
// understates any native value attainable inside the viewport envelope.
fn multiply_upper(a: f32, b: f32) -> f32 {
    let exact = f64::from(a) * f64::from(b);
    let rounded = exact as f32;
    if f64::from(rounded) < exact { rounded.next_up() } else { rounded }
}
fn resolve(size: Size, parent: Option<f32>) -> Option<f32> {
    match size {
        Size::Auto => None,
        Size::Pixels(value) => Some(value),
        // Immutable Taffy RIVE_YOGA_PERCENT_TAG preserves Yoga operation order:
        // percentage * parent * 0.01, not (percentage / 100) * parent.
        Size::Percent(percent) => parent.map(|parent| multiply_upper(multiply_upper(percent, parent), 0.01)),
    }
}

fn resolve_witness(size: Size, parent: Option<f32>) -> Option<f32> {
    match size {
        Size::Auto => None,
        Size::Pixels(value) => Some(value),
        // Native binary32 operations are monotone in the nonnegative parent;
        // evaluating a lower witness therefore preserves a lower witness.
        Size::Percent(percent) => parent.map(|parent| percent * parent * 0.01),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn outward_rounding_never_understates_a_definite_product() {
        for (parent, percent) in [(16384., 1000000.), (12345.67, 133.3333), (1., 0.), (1e36, 0.01)] {
            let upper=resolve(Size::Percent(percent),Some(parent)).unwrap();
            assert!(f64::from(upper)>=f64::from(percent * parent * 0.01));
        }
    }
    #[test]
    fn intrinsic_and_flexible_used_sizes_are_not_invented() {
        let parent=Style::default();
        let mut child=Style { height:Size::Auto,max_height:Size::Pixels(30.),..Style::default() };
        assert!(Bounds::VIEWPORT.child(&child,&parent,"test").unwrap().upper[1].is_none());
        child.height=Size::Pixels(20.);child.flex=super::super::flex::Flex {grow:1.,shrink:1.,basis:Size::Pixels(0.)};
        assert!(Bounds::VIEWPORT.child(&child,&parent,"test").unwrap().upper[1].is_none());
        assert!(resolve(Size::Percent(1000000.),None).is_none());
    }
}
