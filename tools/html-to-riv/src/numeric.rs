//! Focused exponent guard for resolved definite size chains. Unknown intrinsic
//! measurements and aggregate layout/position arithmetic are not proven here.
use super::{Diagnostic, Direction, Size, Style, unsupported};

/// Only the ordinary sizing inputs consumed by this exponent guard. Synthetic
/// owners do not clone inherited variables, paint or numerical provenance.
struct Layout {
    sizes: [Size; 2],
    bounds: [Size; 4],
    padding: super::padding::Padding,
    parent_direction: Direction,
    flexible: bool,
    cross_stretch: bool,
}

#[derive(Clone, Copy)]
pub(super) struct Bounds {
    upper: [Option<f32>; 2],
    lower: [Option<f32>; 2],
    // Lower witness at the maximum supported viewport. Automatic minima may
    // raise a preferred size, so losing its upper bound must not lose this
    // independent evidence of overflow deeper in a percentage chain.
    witness: [Option<f32>; 2],
}
impl Bounds {
    // This is the documented complete viewport domain, not the input viewport.
    pub const VIEWPORT: Self = Self { upper: [Some(16384.); 2], lower: [Some(0.); 2], witness: [Some(16384.); 2] };
    pub fn image(mut self, width:u32, height:u32, aspect_axis:Option<usize>, source:&str)->Result<Self,Diagnostic> {
        let ratio = width as f32 / height as f32;
        if let Some(axis) = aspect_axis {
            let scale = if axis == 0 { ratio } else { 1. / ratio };
            self.upper[axis] = self.upper[1-axis].map(|v| multiply_upper(v, scale));
            self.lower[axis] = Some(0.);
            self.witness[axis] = None;
        }
        let [Some(w),Some(h)] = self.upper else {
            return Err(unsupported(source,"Image sizing requires bounded containing dimensions; intrinsic flex-container cycles need separate qualification"));
        };
        if !w.is_finite() || !h.is_finite() || !multiply_upper(w, 1. / ratio).is_finite() || !multiply_upper(h, ratio).is_finite() {
            return Err(unsupported(source,"Image aspect ratio or fit can exceed finite geometry within the supported viewport range"));
        }
        Ok(self)
    }
    /// Ratio owners resolve opposite-axis constraints against the original
    /// containing content box. Check the native ratio intermediate BEFORE its
    /// clamp: a finite maximum must not disguise overflowing ratio arithmetic.
    pub fn image_constrained(mut self, width:u32, height:u32, axis:usize, containing:Self,
        minimum:Size, maximum:Size, source:&str)->Result<Self,Diagnostic> {
        let ratio=width as f32 / height as f32;
        let definite=self.upper[1-axis].ok_or_else(||unsupported(source,"Responsive image ratio requires a bounded preferred axis"))?;
        let derive_upper=|value| if axis==0 { multiply_upper(value,ratio) } else { divide_upper(value,ratio) };
        let derive_lower=|value| if axis==0 { value*ratio } else { value/ratio };
        let intermediate=derive_upper(definite);
        if !intermediate.is_finite() {
            return Err(unsupported(source,"Responsive image ratio intermediate may exceed finite binary32 geometry before automatic-axis constraints"));
        }
        let min=resolve(minimum,containing.upper[axis]).ok_or_else(||unsupported(source,"Responsive image minimum requires a bounded original containing axis"))?;
        let max=resolve(maximum,containing.upper[axis]);
        if max.is_none() && !matches!(maximum,Size::Auto) {
            return Err(unsupported(source,"Responsive image percentage maximum requires a bounded original containing axis"));
        }
        if !min.is_finite() { return Err(unsupported(source,"Responsive image minimum may exceed finite binary32 geometry")); }
        self.upper[axis]=Some(max.map_or(intermediate,|max|intermediate.min(max)).max(min));
        self.lower[axis]=cap_lower(self.lower[1-axis].map(derive_lower),resolve_witness(minimum,containing.lower[axis]),
            resolve_witness(maximum,containing.lower[axis]),maximum);
        self.witness[axis]=cap_lower(self.witness[1-axis].map(derive_lower),resolve_witness(minimum,containing.witness[axis]),
            resolve_witness(maximum,containing.witness[axis]),maximum);
        // Fit can overflow even after the ratio-derived dimension was capped,
        // for example when a large minimum produces a distorted content box.
        self.image(width,height,None,source)
    }
    /// After image ratio/fit validation, bound the final automatic outer box.
    /// `self` is the original containing block: vertical percentage padding
    /// also uses its width, never the inner image's content width or height.
    /// The inputs are the actual emission inputs, not authored dimensions that
    /// an image composition may have replaced. Fixed and cross-axis Fill boxes
    /// were already checked through their clamp/floor/subtraction path.
    pub fn image_outer(self, content: Self, sizes: [Size; 2], padding: super::padding::Padding,
        parent_direction: Direction, stretch: bool, source: &str) -> Result<(), Diagnostic> {
        let main = usize::from(!parent_direction.is_row());
        let sides = padding.sides();
        for axis in 0..2 {
            if !matches!(sizes[axis], Size::Auto) || (axis != main && stretch) { continue; }
            let inset = padding_sum(sides[axis], sides[axis + 2], self.upper[0], true);
            let Some((content, inset)) = content.upper[axis].zip(inset) else {
                return Err(unsupported(source, "Content-derived image outer sizing requires bounded content and containing-width padding"));
            };
            if !round_upper(f64::from(content) + f64::from(inset)).is_finite() {
                return Err(unsupported(source, "Image content plus padding may exceed finite binary32 outer geometry within the supported viewport domain"));
            }
        }
        Ok(())
    }
    pub fn gap(self, gap:super::gap::Gap, children:usize, source:&str)->Result<(),Diagnostic> {
        if children<2 {return Ok(());}
        // Physical row gap depends on own content height, column on width.
        // Preserve native percent/100 then multiply operation order.
        for (value,axis) in gap.values().into_iter().zip([1,0]) {
            let upper=match value {
                super::gap::GapValue::Normal=>0.,
                super::gap::GapValue::Pixels(v)=>v,
                super::gap::GapValue::Percent(0.)=>0.,
                super::gap::GapValue::Percent(v)=> {
                    let owner=self.upper[axis].ok_or_else(||unsupported(source,"Percentage gaps require a bounded corresponding content dimension; intrinsic percentage gaps remain unqualified"))?;
                    multiply_upper(v/100.,owner)
                }
            };
            // Bound the repeated nonnegative gap additions with upward rounding.
            let mut total=0.;
            for _ in 1..children {total=round_upper(f64::from(total)+f64::from(upper));}
            if !total.is_finite() {return Err(unsupported(source,"Gap accumulation can overflow within the supported viewport range"));}
        }
        Ok(())
    }
    #[cfg(test)]
    pub fn child(self, style: &Style, parent: &Style, source: &str) -> Result<Self, Diagnostic> {
        self.child_with_sizing(style, &super::box_sizing::lower(style, source)?, parent, source)
    }
    pub fn child_with_sizing(self, style: &Style, sizing: &super::box_sizing::Lowered, parent: &Style, source: &str) -> Result<Self, Diagnostic> {
        self.child_with_stretch(style, sizing, parent,
            style.self_alignment.stretches() && !style.margins.cross(parent.direction), source)
    }
    pub fn child_with_stretch(self, style: &Style, sizing: &super::box_sizing::Lowered, parent: &Style,
        cross_stretch: bool, source: &str) -> Result<Self, Diagnostic> {
        self.child_layout(Layout { sizes: sizing.sizes, bounds: sizing.bounds, padding: style.padding,
            parent_direction: parent.direction, flexible: !style.flex.legacy(), cross_stretch }, source)
    }
    pub fn content_owner(self, owner: &super::box_sizing::ContentOwner, source: &str) -> Result<Self, Diagnostic> {
        self.child_layout(Layout { sizes: owner.sizes, bounds: owner.bounds,
            padding: super::padding::Padding::default(), parent_direction: owner.packing,
            flexible: false, cross_stretch: true }, source)
    }
    fn child_layout(self, layout: Layout, source: &str) -> Result<Self, Diagnostic> {
        if layout.padding.has_percentage() && self.upper[0].is_none() {
            return Err(unsupported(source, "Percentage padding requires a bounded containing content width; intrinsic percentage-padding bases need separate qualification"));
        }
        let sizes = layout.sizes;
        let minima = [layout.bounds[0], layout.bounds[1]];
        let maxima = [layout.bounds[2], layout.bounds[3]];
        let mut bounds = [None; 2];
        let mut witnesses = [None; 2];
        let mut lowers = [None; 2];
        let parent_main = if layout.parent_direction.is_row() { 0 } else { 1 };
        for axis in 0..2 {
            // Preserve overflow as an upper-bound infinity until native min/max
            // clamping. Native boundary probes establish a finite maximum can
            // cap an overflowing preferred expression; a larger minimum wins.
            let preferred = resolve(sizes[axis], self.upper[axis]);
            let minimum = resolve(minima[axis], self.upper[axis]);
            let maximum = resolve(maxima[axis], self.upper[axis]);
            let used = if axis == parent_main && layout.flexible {
                // Neither the authored main dimension nor basis is the used
                // flex size. A candidate flex plan needs a separate bound.
                None
            } else if matches!(sizes[axis], Size::Auto) && axis != parent_main
                && layout.cross_stretch {
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
            let preferred_lower = resolve_witness(sizes[axis], self.lower[axis]);
            let minimum_lower = resolve_witness(minima[axis], self.lower[axis]);
            let maximum_lower = resolve_witness(maxima[axis], self.lower[axis]);
            let used_lower = if axis == parent_main && layout.flexible { None }
                else if matches!(sizes[axis], Size::Auto) && axis != parent_main
                    && layout.cross_stretch { self.lower[axis] }
                else { preferred_lower };
            lowers[axis] = cap_lower(used_lower, minimum_lower, maximum_lower, maxima[axis]);
            let preferred_witness = resolve_witness(sizes[axis], self.witness[axis]);
            let minimum_witness = resolve_witness(minima[axis], self.witness[axis]);
            let maximum_witness = resolve_witness(maxima[axis], self.witness[axis]);
            let used_witness = if axis == parent_main && layout.flexible { None }
                else if matches!(sizes[axis], Size::Auto) && axis != parent_main
                    && layout.cross_stretch { self.witness[axis] }
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
        if !layout.padding.is_zero() {
            let sides = layout.padding.sides();
            for axis in 0..2 {
                // All four percentages use containing CONTENT width, including
                // top/bottom. Padding uses ordinary percent/100 conversion,
                // unlike dimension's Yoga-tagged multiply-then-scale path.
                let upper = padding_sum(sides[axis], sides[axis + 2], self.upper[0], true);
                let lower = padding_sum(sides[axis], sides[axis + 2], self.lower[0], false).unwrap_or(0.);
                let witness = padding_sum(sides[axis], sides[axis + 2], self.witness[0], false);
                if upper.is_some_and(|value| !value.is_finite()) || witness.is_some_and(|value| !value.is_finite()) {
                    return Err(unsupported(source, "Resolved padding or padding sum may exceed finite binary32 geometry within the supported viewport domain"));
                }
                // Padding floors the outer box AFTER maximum clamping. For an
                // upper content bound subtract the lower inset, not its upper.
                // max(outer, padding) - padding == max(outer - padding, 0).
                // This preserves the shared inset rather than subtracting an
                // unrelated interval endpoint from an inflated floor bound.
                bounds[axis] = bounds[axis].map(|outer| subtract_upper(outer, lower));
                // These are lower bounds, so subtract an upper inset. If that
                // inset is unknown, neither subtraction provides a witness.
                lowers[axis] = lowers[axis].zip(upper).map(|(outer, inset)| (outer - inset).max(0.));
                witnesses[axis] = witnesses[axis].zip(upper).map(|(outer, inset)| (outer - inset).max(0.));
            }
        }
        Ok(Self { upper: bounds, lower: lowers, witness: witnesses })
    }
}
fn cap_lower(used: Option<f32>, minimum: Option<f32>, maximum: Option<f32>, maximum_size: Size) -> Option<f32> {
    let capped = match (used, maximum) {
        (Some(used), Some(maximum)) => Some(used.min(maximum)),
        (Some(used), None) if matches!(maximum_size, Size::Auto) => Some(used),
        _ => None,
    };
    capped.map(|used| used.max(minimum.unwrap_or(0.)))
}
fn padding_sum(a: super::padding::Inset, b: super::padding::Inset, width: Option<f32>, upper: bool) -> Option<f32> {
    use super::padding::Inset;
    let resolve = |side| match side {
        Inset::Pixels(value) => Some(value),
        Inset::Percent(0.) => Some(0.),
        Inset::Percent(percent) => width.map(|width| if upper { multiply_upper(percent / 100., width) } else { percent / 100. * width }),
    };
    resolve(a).zip(resolve(b)).map(|(a,b)| if upper { round_upper(f64::from(a) + f64::from(b)) } else { a + b })
}
fn subtract_upper(outer: f32, inset: f32) -> f32 { round_upper((f64::from(outer) - f64::from(inset)).max(0.)) }
fn round_upper(exact: f64) -> f32 {
    let rounded = exact as f32;
    if f64::from(rounded) < exact { rounded.next_up() } else { rounded }
}
// For nonnegative arithmetic a correctly rounded operation is monotonic.
// Round each real product upward so the propagated binary32 value never
// understates any native value attainable inside the viewport envelope.
fn multiply_upper(a: f32, b: f32) -> f32 {
    let exact = f64::from(a) * f64::from(b);
    round_upper(exact)
}
// Match Taffy's width / emitted aspect ratio directly. Multiplication by a
// rounded reciprocal can understate the division by one binary32 ULP.
fn divide_upper(value: f32, divisor: f32) -> f32 {
    round_upper(f64::from(value) / f64::from(divisor))
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
    fn responsive_ratio_clamps_use_original_opposite_parent_content_axis() {
        let parent=Bounds{upper:[Some(800.),Some(200.)],lower:[Some(800.),Some(200.)],witness:[Some(800.),Some(200.)]};
        let width=Bounds{upper:[Some(400.),None],lower:[Some(400.),None],witness:[Some(400.),None]};
        let result=width.image_constrained(96,64,1,parent,Size::Pixels(0.),Size::Percent(50.),"test").unwrap();
        assert_eq!(result.upper,[Some(400.),Some(100.)]);
        assert_eq!(result.lower,[Some(400.),Some(100.)]);
        assert_eq!(result.witness,[Some(400.),Some(100.)]);
        let height=Bounds{upper:[None,Some(100.)],lower:[None,Some(100.)],witness:[None,Some(100.)]};
        let result=height.image_constrained(96,64,0,parent,Size::Pixels(0.),Size::Percent(10.),"test").unwrap();
        assert_eq!(result.upper,[Some(80.),Some(100.)]);
        let result=height.image_constrained(96,64,0,parent,Size::Pixels(220.),Size::Auto,"test").unwrap();
        assert_eq!(result.upper,[Some(220.),Some(100.)]);
    }
    #[test]
    fn responsive_ratio_intermediate_and_final_fit_have_separate_overflow_guards() {
        let child=Bounds{upper:[Some(f32::MAX),None],lower:[Some(0.),None],witness:[None,None]};
        let error=child.image_constrained(1,8192,1,Bounds::VIEWPORT,Size::Pixels(0.),Size::Pixels(10.),"test").err().unwrap();
        assert!(error.message.contains("before automatic-axis constraints"));
        let child=Bounds{upper:[Some(10.),None],lower:[Some(0.),None],witness:[None,None]};
        let error=child.image_constrained(8192,1,1,Bounds::VIEWPORT,Size::Pixels(f32::MAX),Size::Auto,"test").err().unwrap();
        assert!(error.message.contains("fit"));
        let safe=child.image_constrained(1,8192,1,Bounds::VIEWPORT,Size::Pixels(0.),Size::Pixels(10.),"test").unwrap();
        assert_eq!(safe.upper,[Some(10.),Some(10.)]);
    }
    #[test]
    fn responsive_percentage_bound_never_borrows_the_image_size_for_unknown_parent() {
        let parent=Bounds{upper:[Some(800.),None],lower:[Some(0.),None],witness:[Some(800.),None]};
        let child=Bounds{upper:[Some(400.),None],lower:[Some(0.),None],witness:[Some(400.),None]};
        assert!(child.image_constrained(96,64,1,parent,Size::Pixels(0.),Size::Percent(20.),"test").is_err());
        assert!(child.image_constrained(96,64,1,parent,Size::Pixels(0.),Size::Pixels(20.),"test").is_ok());
        let unknown=Bounds{upper:[None,None],lower:[Some(0.),None],witness:[None,None]};
        assert!(unknown.image_constrained(96,64,1,parent,Size::Pixels(0.),Size::Pixels(20.),"test").is_err());
    }
    #[test]
    fn outward_ratio_division_bounds_native_division_without_reciprocal_substitution() {
        let mut discriminating=false;
        for w in [1.,47.99,1000.25,f32::MAX/8192.] {
            for (iw,ih) in [(7.,3.),(96.,64.),(8191.,8192.),(1.,8192.)] {
                let ratio=iw/ih;
                let native=w/ratio;
                assert!(divide_upper(w,ratio)>=native);
                discriminating |= native>w*(1./ratio);
            }
        }
        assert!(discriminating,"test must distinguish direct division from reciprocal multiplication");
    }
    #[test]
    fn outward_rounding_never_understates_a_definite_product() {
        for (parent, percent) in [(16384., 1000000.), (12345.67, 133.3333), (1., 0.), (1e36, 0.01)] {
            let upper=resolve(Size::Percent(percent),Some(parent)).unwrap();
            assert!(f64::from(upper)>=f64::from(percent * parent * 0.01));
        }
    }
    #[test]
    fn padding_does_not_pass_border_box_bounds_as_content_box_bounds() {
        let mut child=Style {width:Size::Pixels(100.),height:Size::Pixels(100.),..Style::default()};
        child.padding.apply("padding","10px",super::super::padding::Padding::default(),16.,"test").unwrap();
        let bounds=Bounds::VIEWPORT.child(&child,&Style::default(),"test").unwrap();
        assert_eq!(bounds.upper,[Some(80.);2]);assert_eq!(bounds.witness,[Some(80.);2]);
    }
    #[test]
    fn content_box_bounds_use_lowered_outer_dimensions_and_native_cancellation() {
        let mut child=Style {box_sizing:super::super::box_sizing::BoxSizing::ContentBox,width:Size::Pixels(1.),height:Size::Pixels(1.),..Style::default()};
        child.padding.apply("padding","1px",super::super::padding::Padding::default(),16.,"test").unwrap();
        let content=Bounds::VIEWPORT.child(&child,&Style::default(),"test").unwrap();
        assert_eq!(content.upper,[Some(1.);2]);
        child.width=Size::Pixels(999999.9375);child.padding.apply("padding","1000000px",super::super::padding::Padding::default(),16.,"test").unwrap();
        let content=Bounds::VIEWPORT.child(&child,&Style::default(),"test").unwrap();
        assert!(content.upper[0].unwrap()>=1000000.);
    }
    #[test]
    fn percentage_content_upper_subtracts_lower_width_based_insets() {
        let parent=Bounds {upper:[Some(200.),Some(20.)],lower:[Some(100.),Some(0.)],witness:[Some(200.),Some(20.)]};
        let mut child=Style {width:Size::Pixels(100.),height:Size::Pixels(100.),..Style::default()};
        child.padding.apply("padding","10%",super::super::padding::Padding::default(),16.,"test").unwrap();
        let content=parent.child(&child,&Style::default(),"test").unwrap();
        assert_eq!(content.upper,[Some(80.);2]); // top/bottom also use width
        assert!(content.witness[0].unwrap()<=60.);
        child.max_width=Size::Pixels(1.);child.max_height=Size::Pixels(1.);
        assert_eq!(parent.child(&child,&Style::default(),"test").unwrap().upper,[Some(0.);2]);
    }
    #[test]
    fn two_finite_padding_sides_can_overflow_their_sum() {
        use super::super::padding::Inset;
        assert!(padding_sum(Inset::Percent(100.),Inset::Pixels(0.),Some(2e38),true).unwrap().is_finite());
        assert!(!padding_sum(Inset::Percent(100.),Inset::Percent(100.),Some(2e38),true).unwrap().is_finite());
    }
    fn tall_padded_image(final_percent: f32) -> (Bounds, Bounds, super::super::padding::Padding) {
        let parent_style = Style::default();
        let mut parent = Bounds::VIEWPORT;
        for percent in [1000000., 1000000., 1000000., 1000000., 1000000., 1000000., 1000000., final_percent] {
            parent = parent.child(&Style { width: Size::Percent(percent), height: Size::Pixels(1.),
                ..Style::default() }, &parent_style, "chain").unwrap();
        }
        let mut outer = Style { width: Size::Percent(100.), height: Size::Auto, ..Style::default() };
        outer.padding.apply("padding", "1000000% 0", super::super::padding::Padding::default(), 16., "image").unwrap();
        // The current public admission still rejects this percentage-padding
        // image. Exercise the proposed ordinary owners directly without opening
        // admission or allocating/rendering these enormous dimensions.
        let outer_content = parent.child(&outer, &parent_style, "outer").unwrap();
        let inner = outer_content.content_owner(&super::super::box_sizing::ContentOwner {
            packing: Direction::Column, sizes: [Size::Percent(100.), Size::Auto],
            bounds: [Size::Pixels(0.), Size::Pixels(0.), Size::Auto, Size::Auto],
        }, "inner").unwrap().image(1, 8192, Some(1), "image").unwrap();
        (parent, inner, outer.padding)
    }
    #[test]
    fn finite_image_and_padding_can_overflow_content_derived_outer() {
        let (parent, inner, padding) = tall_padded_image(10000.);
        let sides = padding.sides();
        let inset = padding_sum(sides[1], sides[3], parent.upper[0], true).unwrap();
        assert!(inset.is_finite());
        assert!(inner.upper.into_iter().all(|v| v.unwrap().is_finite()));
        assert!((inner.upper[1].unwrap() + inset).is_infinite());
        // Confirm an actual binary32 witness, not only an interval upper
        // bound: all prior native operations are finite, but the sum is not.
        let width = parent.witness[0].unwrap();
        let native_inset = (1000000_f32 / 100.) * width;
        let native_padding = native_inset + native_inset;
        let native_content = (100_f32 * width * 0.01) * 8192.;
        assert!(native_inset.is_finite() && native_padding.is_finite() && native_content.is_finite());
        assert!((native_content + native_padding).is_infinite());
        let error = parent.image_outer(inner, [Size::Percent(100.), Size::Auto], padding,
            Direction::Column, true, "image").unwrap_err();
        assert!(error.message.contains("content plus padding"));
    }
    #[test]
    fn finite_content_derived_outer_counterpart_remains_admitted() {
        let (parent, inner, padding) = tall_padded_image(1000.);
        parent.image_outer(inner, [Size::Percent(100.), Size::Auto], padding,
            Direction::Column, true, "image").unwrap();
    }
    #[test]
    fn outer_addition_uses_emitted_hug_axes_not_fixed_or_fill_axes() {
        let parent = Bounds { upper: [Some(1e38), Some(1.)], lower: [Some(0.); 2], witness: [None; 2] };
        let content = Bounds { upper: [Some(3e38); 2], lower: [Some(0.); 2], witness: [None; 2] };
        let mut padding = super::super::padding::Padding::default();
        padding.apply("padding", "50%", padding, 16., "test").unwrap();
        // Hypothetical content+inset overflows both axes. Fixed dimensions and
        // genuine cross Fill do not acquire that extra addition.
        parent.image_outer(content, [Size::Pixels(1.); 2], padding, Direction::Column, false, "fixed").unwrap();
        parent.image_outer(content, [Size::Auto, Size::Pixels(1.)], padding, Direction::Column, true, "fill width").unwrap();
        parent.image_outer(content, [Size::Pixels(1.), Size::Auto], padding, Direction::Row, true, "fill height").unwrap();
        assert!(parent.image_outer(content, [Size::Auto, Size::Pixels(1.)], padding, Direction::Column, false, "hug width").is_err());
        assert!(parent.image_outer(content, [Size::Pixels(1.), Size::Auto], padding, Direction::Row, false, "hug height").is_err());
        // Main Auto remains Hug even when cross-axis stretch is enabled.
        assert!(parent.image_outer(content, [Size::Pixels(1.), Size::Auto], padding, Direction::Column, true, "main height").is_err());
    }
    #[test]
    fn content_derived_outer_requires_original_width_and_content_bounds() {
        let unknown = Bounds { upper: [None; 2], lower: [None; 2], witness: [None; 2] };
        let mut padding = super::super::padding::Padding::default();
        padding.apply("padding", "1% 0", padding, 16., "test").unwrap();
        let sizes = [Size::Pixels(1.), Size::Auto];
        assert!(unknown.image_outer(Bounds::VIEWPORT, sizes, padding, Direction::Column, true, "unknown parent width").is_err());
        assert!(Bounds::VIEWPORT.image_outer(unknown, sizes, padding, Direction::Column, true, "unknown content").is_err());
        // Point padding has no containing-width dependency.
        padding.apply("padding", "1px", padding, 16., "test").unwrap();
        unknown.image_outer(Bounds::VIEWPORT, sizes, padding, Direction::Column, true, "points").unwrap();
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
