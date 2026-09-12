//! Ordinary ratio-owner bounds for one authored percentage axis. The opposite
//! axis remains automatic at runtime; no parent measurements enter this plan.
use super::{Diagnostic, Size, Style, unsupported, box_sizing::Lowered,
    computed_provenance::NumericSize, scalar_provenance::ScalarProvenance};

pub(super) struct Resolved {
    pub automatic: usize,
    pub minimum: Size,
    pub maximum: Size,
    guard: bool,
}
fn zero(size: Size, numeric: &NumericSize) -> bool {
    matches!((size,numeric),(Size::Pixels(0.),NumericSize::Pixels(Ok(v))) if v.is_exact_zero() && v.native()==0.)
}
fn absent(size: Size, numeric: &NumericSize) -> bool { matches!((size,numeric),(Size::Auto,NumericSize::Auto)) }
fn scalar(size: Size, numeric: &NumericSize, percent: bool) -> bool {
    match (size,numeric) {
        (Size::Pixels(n),NumericSize::Pixels(Ok(v))) => v.native()==n && v.is_nonnegative(),
        (Size::Percent(n),NumericSize::Percent(Ok(v))) if percent => v.native()==n && v.is_nonnegative(),
        _ => false,
    }
}
pub(super) fn resolve(style: &Style, parent: &Style, source: &str) -> Result<Option<Resolved>, Diagnostic> {
    if !super::image_constraints::has_constraints(style) { return Ok(None); }
    let sizes=[style.width,style.height];
    let numeric=[&style.numeric.width,&style.numeric.height];
    let automatic=match sizes {
        [Size::Percent(_),Size::Auto] => 1,
        [Size::Auto,Size::Percent(_)] => 0,
        _ => return Ok(None),
    };
    let definite=1-automatic;
    let reject=||unsupported(source,"Responsive image constraints require one percentage preferred axis, opposite point/percentage min/max bounds, zero image padding, and ordinary sizing without automatic cross stretch; preferred-axis bounds need separate qualification");
    if !style.flex.legacy() || !style.padding.is_zero()
        || !style.numeric.padding.iter().all(|v|v.as_ref().is_ok_and(|v|v.is_exact_zero() && v.native()==0.))
        || !scalar(sizes[definite],numeric[definite],true)
        || !absent(sizes[automatic],numeric[automatic])
        || (automatic==usize::from(parent.direction.is_row()) && style.self_alignment.stretches()) {
        return Err(reject());
    }
    let minima=[style.min_width,style.min_height];
    let maxima=[style.max_width,style.max_height];
    let min_numeric=[&style.numeric.min_width,&style.numeric.min_height];
    let max_numeric=[&style.numeric.max_width,&style.numeric.max_height];
    if !zero(minima[definite],min_numeric[definite]) || !absent(maxima[definite],max_numeric[definite]) { return Err(reject()); }
    let maximum_absent=absent(maxima[automatic],max_numeric[automatic]);
    if !scalar(minima[automatic],min_numeric[automatic],true)
        || !(maximum_absent || scalar(maxima[automatic],max_numeric[automatic],true)) { return Err(reject()); }
    // Keep authored bounds separate: point/percentage ordering can change on
    // resize, and native clamping gives the minimum priority when they cross.
    Ok(Some(Resolved{automatic,minimum:minima[automatic],maximum:maxima[automatic],guard:!maximum_absent}))
}
impl Resolved {
    pub fn lower_outer(&self, style: &Style) -> Lowered {
        let mut numeric=style.numeric.clone();
        let mut bounds=[style.min_width,style.min_height,style.max_width,style.max_height];
        // Both minima stay explicit. Native ratio completion must not transfer
        // the authored automatic-axis bound to the definite axis.
        if self.guard {
            let definite=1-self.automatic;
            bounds[definite+2]=[style.width,style.height][definite];
            if definite==0 { numeric.max_width=numeric.width.clone(); }
            else { numeric.max_height=numeric.height.clone(); }
        }
        // Retain the reset's mathematical zero instead of deriving it from a
        // rounded authored bound. This also documents ordinary wire semantics.
        let reset=NumericSize::Pixels(ScalarProvenance::exact_constant(0.).map_err(Into::into));
        if self.automatic==0 { numeric.min_height=reset; } else { numeric.min_width=reset; }
        Lowered{sizes:[style.width,style.height],bounds,numeric}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn style(css:&str)->Style {
        let document=scraper::Html::parse_document("<img id=image>");
        let selector=scraper::Selector::parse("#image").unwrap();
        super::super::computed(document.select(&selector).next().unwrap(),
            &crate::css::stylesheet(&format!("#image{{align-self:flex-start;{css}}}")).unwrap(),
            &Style::default(),false,true).unwrap()
    }
    #[test]
    fn synthetic_maximum_keeps_original_percentage_provenance_and_authored_style() {
        let authored=style("width:47.990000001%;max-height:83px");
        let plan=resolve(&authored,&Style::default(),"image").unwrap().unwrap();
        let lowered=plan.lower_outer(&authored);
        assert!(matches!(authored.max_width,Size::Auto));
        assert!(matches!(authored.height,Size::Auto));
        let (NumericSize::Percent(Ok(original)),NumericSize::Percent(Ok(guard)))=(&authored.numeric.width,&lowered.numeric.max_width) else {panic!("percentage provenance missing")};
        assert!(original.proves_equal(guard));
        assert_eq!(original.native(),guard.native());
        assert!(guard.absolute_error_upper()>0.);
        assert!(matches!(lowered.bounds[0],Size::Pixels(0.)));
        assert!(matches!(lowered.bounds[1],Size::Pixels(0.)));
    }
    #[test]
    fn combined_bounds_preserve_mixed_units_conflicts_and_original_provenance() {
        for css in ["width:50%;min-height:100px;max-height:35%",
            "height:50%;min-width:80%;max-width:30%", "width:50%;min-height:45%"] {
            let authored=style(css);
            let plan=resolve(&authored,&Style::default(),"image").unwrap().unwrap();
            let lowered=plan.lower_outer(&authored);
            let axis=plan.automatic;
            let unchanged=|left:Size,right:Size|match (left,right) {
                (Size::Pixels(a),Size::Pixels(b)) | (Size::Percent(a),Size::Percent(b)) => a==b,
                (Size::Auto,Size::Auto)=>true, _=>false,
            };
            assert!(unchanged(lowered.bounds[axis],[authored.min_width,authored.min_height][axis]));
            assert!(unchanged(lowered.bounds[axis+2],[authored.max_width,authored.max_height][axis]));
            let original=if axis==0 {&authored.numeric.min_width} else {&authored.numeric.min_height};
            let emitted=if axis==0 {&lowered.numeric.min_width} else {&lowered.numeric.min_height};
            match (original,emitted) {
                (NumericSize::Pixels(Ok(a)),NumericSize::Pixels(Ok(b))) |
                (NumericSize::Percent(Ok(a)),NumericSize::Percent(Ok(b))) => assert!(a.proves_equal(b)),
                _=>panic!("minimum provenance changed"),
            }
            if plan.guard { assert!(matches!(lowered.bounds[3-axis],Size::Percent(50.))); }
            else { assert!(matches!(lowered.bounds[3-axis],Size::Auto)); }
        }
    }
    #[test]
    fn unsupported_bound_topologies_and_tiny_percentage_padding_stay_closed() {
        for css in ["width:40%;min-width:20%;min-height:20%;", "width:40%;min-height:auto;max-height:10px",
            "width:40%;max-width:50px;max-height:10px", "width:40%;min-height:auto",
            "width:40%;max-height:10px;padding:1e-50%", "width:40%;max-height:10px;padding:1e-50px"] {
            assert!(resolve(&style(css),&Style::default(),"image").is_err(),"{css}");
        }
        let mut missing=style("height:40%;max-width:200px");
        missing.numeric.height=NumericSize::Auto;
        assert!(resolve(&missing,&Style::default(),"image").is_err());
    }
}
