use nuxie_html_to_riv::{compile, CompileInput, OverflowAxis, RuntimeCapability};
fn input(css:&str)->CompileInput {
    CompileInput {html:"<div id=a><div id=b></div></div>".into(),css:css.into(),width:390.,height:320.,..Default::default()}
}
#[test]
fn axis_shorthands_longhands_cascade_and_css_wide_values_emit_checked_contract() {
    for (css, axes) in [
        ("#a{overflow:clip visible}",vec![OverflowAxis::X]),
        ("#a{overflow:visible clip}",vec![OverflowAxis::Y]),
        ("#a{overflow-x:hidden;overflow-x:clip}",vec![OverflowAxis::X]),
        ("#a{overflow:clip;overflow-y:visible}",vec![OverflowAxis::X]),
        ("#a{overflow-x:clip;overflow:visible clip}",vec![OverflowAxis::Y]),
        ("#a{overflow-x:clip!important;overflow:visible}",vec![OverflowAxis::X]),
        ("#a{overflow:clip visible}#b{overflow:inherit}",vec![OverflowAxis::X,OverflowAxis::X]),
        ("#a{overflow:clip visible}#b{overflow-x:inherit;overflow-y:initial}",vec![OverflowAxis::X,OverflowAxis::X]),
        ("#a{overflow:clip;overflow-y:unset}",vec![OverflowAxis::X]),
        ("#a{--ov:clip visible;overflow:var(--ov)}",vec![OverflowAxis::X]),
    ] {
        let output=compile(&input(css)).unwrap();let req=output.runtime_requirements;
        assert_eq!(req.version,20,"{css}");assert_eq!(req.layout_axis_overflow.iter().map(|e|e.axis).collect::<Vec<_>>(),axes,"{css}");
        assert!(req.capabilities.contains(&RuntimeCapability::LayoutCssAxisOverflowV1));
        req.ensure_supported(&req.capabilities.iter().copied().collect::<Vec<_>>()).unwrap();
    }
}
#[test]
fn two_axis_and_visible_pairs_need_no_axis_policy_and_scrollable_pairs_reject() {
    for value in ["visible", "clip", "hidden", "visible visible", "clip clip", "hidden clip", "clip hidden", "hidden hidden"] {
        let out=compile(&input(&format!("#a{{overflow:{value}}}"))).unwrap();assert!(out.runtime_requirements.layout_axis_overflow.is_empty());assert_ne!(out.runtime_requirements.version,20);
    }
    for css in ["#a{overflow:visible hidden}","#a{overflow:hidden visible}","#a{overflow-x:hidden}"] {
        assert_eq!(compile(&input(css)).unwrap_err()[0].code,"unsupported-computed-overflow");
    }
    for css in ["#a{overflow:auto}","#a{overflow-x:scroll}","#a{overflow:clip clip clip}","#a{overflow-y:clip visible}"] {
        assert_eq!(compile(&input(css)).unwrap_err()[0].code,"unsupported-overflow");
    }
}
