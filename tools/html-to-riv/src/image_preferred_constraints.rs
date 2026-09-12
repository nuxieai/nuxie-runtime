//! Preferred-axis image bounds stay in the file's original percentage basis.
//! Point bounds propagate source-derived ratio limits; homogeneous percentage
//! bounds fold coefficients, never a viewport measurement or computed Style.
//! A point minimum can follow a folded percentage cap; folded preferred
//! percentages can also retain independent opposite-axis bounds on this owner.
use super::{Diagnostic, Size, Style, unsupported, box_sizing::Lowered,
    computed_provenance::NumericSize, scalar_provenance::{ScalarProvenance as Scalar, ScalarError}};
use crate::assets::ImageMetadata;

pub(super) struct Resolved {
    pub automatic: usize,
    lowered: Lowered,
    pub explicit_minima: bool,
}
fn zero(size:Size,numeric:&NumericSize)->bool {
    matches!((size,numeric),(Size::Pixels(0.),NumericSize::Pixels(Ok(v))) if v.is_exact_zero() && v.native()==0.)
}
fn absent(size:Size,numeric:&NumericSize)->bool {matches!((size,numeric),(Size::Auto,NumericSize::Auto))}
fn scalar(size:Size,numeric:&NumericSize)->Option<(bool,Scalar)> {
    match (size,numeric) {
        (Size::Pixels(n),NumericSize::Pixels(Ok(v))) if v.native()==n && v.is_nonnegative()=>Some((false,v.clone())),
        (Size::Percent(n),NumericSize::Percent(Ok(v))) if v.native()==n && v.is_nonnegative()=>Some((true,v.clone())),
        _=>None,
    }
}
fn transfer(value:&Scalar, automatic:usize, metadata:ImageMetadata)->Result<Scalar,ScalarError> {
    let intrinsic=[metadata.width as f32,metadata.height as f32];
    let ratio=intrinsic[0]/intrinsic[1];
    // CSS ideal ratio is the exact encoded dimensions. The native operation
    // uses the emitted binary32 ratio; retain that difference as real error.
    let ideal=value.multiply(&Scalar::exact_constant(intrinsic[automatic])?)?
        .divide_positive_constant(intrinsic[1-automatic])?;
    ideal.with_native(if automatic==1 {value.native()/ratio} else {value.native()*ratio})
}
pub(super) fn resolve(style:&Style,parent:&Style,metadata:ImageMetadata,source:&str)->Result<Option<Resolved>,Diagnostic> {
    let preferred=match [style.width,style.height] {
        [Size::Percent(_),Size::Auto]=>0,
        [Size::Auto,Size::Percent(_)]=>1,
        _=>return Ok(None),
    };
    let automatic=1-preferred;
    let sizes=[style.width,style.height];
    let sizes_numeric=[&style.numeric.width,&style.numeric.height];
    let minima=[style.min_width,style.min_height];
    let maxima=[style.max_width,style.max_height];
    let minima_numeric=[&style.numeric.min_width,&style.numeric.min_height];
    let maxima_numeric=[&style.numeric.max_width,&style.numeric.max_height];
    if zero(minima[preferred],minima_numeric[preferred]) && absent(maxima[preferred],maxima_numeric[preferred]) {return Ok(None);}
    let reject=||unsupported(source,"Preferred image bounds require one percentage preferred axis, point bounds, point minimum with percentage maximum, or preferred percentages with independent opposite bounds, zero image padding, and ordinary sizing without automatic cross stretch; reverse mixed bounds and preferred point bounds with opposite constraints need separate qualification");
    if !absent(sizes[automatic],sizes_numeric[automatic]) || !style.flex.legacy()
        || !style.padding.is_zero() || !style.numeric.padding.iter().all(|v|v.as_ref().is_ok_and(|v|v.is_exact_zero() && v.native()==0.))
        || (automatic==usize::from(parent.direction.is_row()) && style.self_alignment.stretches()) {return Err(reject());}
    let Some((true,coefficient))=scalar(sizes[preferred],sizes_numeric[preferred]) else {return Err(reject());};
    let Some((min_percent,minimum))=scalar(minima[preferred],minima_numeric[preferred]) else {return Err(reject());};
    let maximum=if absent(maxima[preferred],maxima_numeric[preferred]) {None}
        else {Some(scalar(maxima[preferred],maxima_numeric[preferred]).ok_or_else(reject)?)};
    let automatic_constraints=!zero(minima[automatic],minima_numeric[automatic]) || !absent(maxima[automatic],maxima_numeric[automatic]);
    if scalar(minima[automatic],minima_numeric[automatic]).is_none()
        || !(absent(maxima[automatic],maxima_numeric[automatic]) || scalar(maxima[automatic],maxima_numeric[automatic]).is_some()) {return Err(reject());}
    let mixed_cap=!min_percent && !minimum.is_exact_zero() && maximum.as_ref().is_some_and(|(percent,_)|*percent);
    let percent=if !minimum.is_exact_zero() {min_percent} else {maximum.as_ref().map_or(min_percent,|(p,_)|*p)};
    if !mixed_cap && ((!minimum.is_exact_zero() && min_percent!=percent) || maximum.as_ref().is_some_and(|(p,_)|*p!=percent)) {return Err(reject());}
    if !percent && automatic_constraints {return Err(reject());}
    let mut lowered=Lowered{sizes,bounds:[minima[0],minima[1],maxima[0],maxima[1]],numeric:style.numeric.clone()};
    let result=(||->Result<(),ScalarError>{
        if percent {
            let capped=if let Some((_,maximum))=&maximum {coefficient.minimum(maximum)?} else {coefficient};
            let selected=capped.maximum(&minimum)?;
            lowered.sizes[preferred]=Size::Percent(selected.native());
            let selected_numeric=NumericSize::Percent(Ok(selected.clone()));
            if preferred==0 {lowered.numeric.width=selected_numeric.clone();}
            else {lowered.numeric.height=selected_numeric.clone();}
            // Only preferred bounds were accounted into the coefficient.
            // Automatic percentages keep the original opposite parent basis.
            lowered.bounds[preferred]=Size::Pixels(0.);
            lowered.bounds[preferred+2]=Size::Auto;
            let reset=NumericSize::Pixels(Scalar::exact_constant(0.).map_err(Into::into));
            if preferred==0 {lowered.numeric.min_width=reset;lowered.numeric.max_width=NumericSize::Auto;}
            else {lowered.numeric.min_height=reset;lowered.numeric.max_height=NumericSize::Auto;}
            if !absent(maxima[automatic],maxima_numeric[automatic]) {
                // Suppress native maximum-vector ratio transfer using the
                // selected coefficient, never the original unclamped value.
                lowered.bounds[preferred+2]=Size::Percent(selected.native());
                if preferred==0 {lowered.numeric.max_width=selected_numeric;}
                else {lowered.numeric.max_height=selected_numeric;}
            }
        } else {
            if mixed_cap {
                let selected=coefficient.minimum(&maximum.as_ref().expect("mixed percentage maximum").1)?;
                lowered.sizes[preferred]=Size::Percent(selected.native());
                if preferred==0 {lowered.numeric.width=NumericSize::Percent(Ok(selected));lowered.numeric.max_width=NumericSize::Auto;}
                else {lowered.numeric.height=NumericSize::Percent(Ok(selected));lowered.numeric.max_height=NumericSize::Auto;}
                // The percentage cap precedes the point minimum; retaining it
                // as a bound would incorrectly cap the minimum on resize.
                lowered.bounds[preferred+2]=Size::Auto;
            }
            let auto_min=transfer(&minimum,automatic,metadata)?;
            lowered.bounds[automatic]=Size::Pixels(auto_min.native());
            let auto_min=NumericSize::Pixels(Ok(auto_min));
            if automatic==0 {lowered.numeric.min_width=auto_min;} else {lowered.numeric.min_height=auto_min;}
            if let Some((_,maximum))=maximum.as_ref().filter(|_|!mixed_cap) {
                let auto_max=transfer(&maximum.maximum(&minimum)?,automatic,metadata)?;
                lowered.bounds[automatic+2]=Size::Pixels(auto_max.native());
                let auto_max=NumericSize::Pixels(Ok(auto_max));
                if automatic==0 {lowered.numeric.max_width=auto_max;} else {lowered.numeric.max_height=auto_max;}
            }
        }
        Ok(())
    })();
    result.map_err(|_|unsupported(source,"Preferred image bound arithmetic cannot preserve finite source-derived geometry and provenance"))?;
    Ok(Some(Resolved{automatic,lowered,explicit_minima:!percent || automatic_constraints}))
}
impl Resolved {
    pub fn lower_outer(&self)->Lowered {Lowered{sizes:self.lowered.sizes,bounds:self.lowered.bounds,numeric:self.lowered.numeric.clone()}}
    pub fn minima(&self)->[Size;2] {[self.lowered.bounds[0],self.lowered.bounds[1]]}
    pub fn validate_bounds(&self,child:super::numeric::Bounds,containing:super::numeric::Bounds,metadata:ImageMetadata,source:&str)->Result<super::numeric::Bounds,Diagnostic> {
        child.image_preferred(metadata.width,metadata.height,self.automatic,containing,&self.lowered,source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn style(css:&str)->Style {
        let document=scraper::Html::parse_document("<img id=image>");
        let selector=scraper::Selector::parse("#image").unwrap();
        super::super::computed(document.select(&selector).next().unwrap(),
            &crate::css::stylesheet(&format!("#image{{align-self:flex-start;{css}}}")).unwrap(),&Style::default(),false,true).unwrap()
    }
    fn resolve_css(css:&str)->Resolved {resolve(&style(css),&Style::default(),ImageMetadata{width:96,height:64,asset_index:0},"image").unwrap().unwrap()}
    #[test]
    fn conflicting_point_bounds_propagate_normalized_limits_without_mutating_authored_preferred() {
        let authored=style("width:50%;min-width:200px;max-width:120px");
        let plan=resolve(&authored,&Style::default(),ImageMetadata{width:96,height:64,asset_index:0},"image").unwrap().unwrap();
        let lowered=plan.lower_outer();
        assert!(matches!(authored.max_width,Size::Pixels(120.)));
        assert!(matches!(authored.height,Size::Auto));
        assert!(matches!(lowered.bounds[0],Size::Pixels(200.)));
        assert!(matches!(lowered.bounds[2],Size::Pixels(120.)));
        let (Size::Pixels(min),Size::Pixels(max))=(lowered.bounds[1],lowered.bounds[3]) else {panic!("point bounds missing")};
        assert_eq!(min,200./1.5);assert_eq!(min,max);
        assert!(plan.explicit_minima);
    }
    #[test]
    fn percentage_constraints_fold_source_coefficients_and_keep_original_error() {
        for (css,coefficient) in [("width:50%;min-width:75%",75.),("width:50%;max-width:30%",30.),("width:50%;min-width:75%;max-width:30%",75.)] {
            let plan=resolve_css(css);let lowered=plan.lower_outer();
            assert!(matches!(lowered.sizes[0],Size::Percent(v) if v==coefficient));
            assert!(matches!(lowered.bounds,[Size::Pixels(0.),Size::Pixels(0.),Size::Auto,Size::Auto]));
            assert!(!plan.explicit_minima);
        }
        let authored=style("height:47.990000001%;max-height:90%");
        let plan=resolve(&authored,&Style::default(),ImageMetadata{width:96,height:64,asset_index:0},"image").unwrap().unwrap();
        let lowered=plan.lower_outer();
        let NumericSize::Percent(Ok(v))=&lowered.numeric.height else {panic!("missing selected provenance")};
        assert!(v.ideal_bounds().lower()<=47.990000001 && v.ideal_bounds().upper()>=47.990000001);
        assert!(v.absolute_error_upper()>0.);
    }
    #[test]
    fn propagated_ratio_keeps_exact_intrinsic_ideal_and_actual_native_operation() {
        let value=Scalar::from_decimal("100",100.).unwrap();
        let metadata=ImageMetadata{width:7,height:3,asset_index:0};
        let result=transfer(&value,1,metadata).unwrap();
        assert_eq!(result.native(),100./(7_f32/3.));
        let ideal=300_f64/7.;assert!(result.ideal_bounds().lower()<=ideal && result.ideal_bounds().upper()>=ideal);
        assert!(result.absolute_error_upper()>0.);
        let huge=Scalar::exact_constant(f32::MAX).unwrap();
        assert!(transfer(&huge,0,ImageMetadata{width:8192,height:1,asset_index:0}).is_err());
    }
    #[test]
    fn mixed_percentage_cap_precedes_point_minimum_and_removes_both_maxima() {
        for (cap,expected) in [(40.,40.),(75.,50.)] {
            let authored=style(&format!("width:50%;min-width:150px;max-width:{cap}%"));
            let plan=resolve(&authored,&Style::default(),ImageMetadata{width:96,height:64,asset_index:0},"image").unwrap().unwrap();
            let lowered=plan.lower_outer();
            assert!(matches!(lowered.sizes[0],Size::Percent(v) if v==expected));
            assert!(matches!(lowered.bounds,[Size::Pixels(150.),Size::Pixels(100.),Size::Auto,Size::Auto]));
            assert!(matches!(authored.max_width,Size::Percent(v) if v==cap));
            assert!(matches!(authored.width,Size::Percent(50.)));
            assert!(plan.explicit_minima);
        }
    }
    #[test]
    fn both_axis_guard_uses_selected_coefficient_and_keeps_opposite_provenance() {
        let authored=style("height:50%;min-height:75.000000001%;max-height:60%;min-width:80.000000001%;max-width:30%");
        let plan=resolve(&authored,&Style::default(),ImageMetadata{width:96,height:64,asset_index:0},"image").unwrap().unwrap();
        let lowered=plan.lower_outer();
        assert!(matches!(lowered.sizes[1],Size::Percent(75.)));
        assert!(matches!(lowered.bounds,[Size::Percent(80.),Size::Pixels(0.),Size::Percent(30.),Size::Percent(75.)]));
        let (NumericSize::Percent(Ok(selected)),NumericSize::Percent(Ok(guard)))=(&lowered.numeric.height,&lowered.numeric.max_height) else {panic!("selected guard provenance missing")};
        assert!(selected.proves_equal(guard));assert_eq!(selected.native(),guard.native());
        assert!(selected.ideal_bounds().lower()<=75.000000001 && selected.ideal_bounds().upper()>=75.000000001);
        let (NumericSize::Percent(Ok(original)),NumericSize::Percent(Ok(actual)))=(&authored.numeric.min_width,&lowered.numeric.min_width) else {panic!("opposite provenance missing")};
        assert!(original.proves_equal(actual));assert!(actual.absolute_error_upper()>0.);
        assert!(plan.explicit_minima);
    }
    #[test]
    fn mixed_cap_keeps_fractional_coefficient_error_and_opposite_automatic_minimum_stays_closed() {
        let authored=style("width:50%;min-width:150px;max-width:39.990000001%");
        let plan=resolve(&authored,&Style::default(),ImageMetadata{width:96,height:64,asset_index:0},"image").unwrap().unwrap();
        let lowered=plan.lower_outer();
        let NumericSize::Percent(Ok(selected))=&lowered.numeric.width else {panic!("missing coefficient")};
        assert!(selected.absolute_error_upper()>0.);
        assert!(selected.ideal_bounds().lower()<=39.990000001 && selected.ideal_bounds().upper()>=39.990000001);
        let bad=style("width:50%;min-width:75%;min-height:auto;max-height:100px");
        assert!(resolve(&bad,&Style::default(),ImageMetadata{width:96,height:64,asset_index:0},"image").is_err());
    }
    #[test]
    fn unqualified_topologies_and_missing_provenance_are_rejected() {
        for css in ["width:50%;min-width:100px;max-width:80%;min-height:10px", "width:50%;min-width:40%;max-width:300px",
            "width:50%;min-width:auto", "width:50%;min-width:10px;max-height:200px",
            "width:50%;min-width:10px;padding:1e-50px", "width:50%;min-width:10px;align-self:stretch"] {
            let parent=Style{direction:super::super::Direction::Row,..Style::default()};
            assert!(resolve(&style(css),&parent,ImageMetadata{width:96,height:64,asset_index:0},"image").is_err(),"{css}");
        }
        let mut authored=style("height:50%;min-height:180px");authored.numeric.min_height=NumericSize::Auto;
        assert!(resolve(&authored,&Style::default(),ImageMetadata{width:96,height:64,asset_index:0},"image").is_err());
    }
}
