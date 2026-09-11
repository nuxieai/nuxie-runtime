use nuxie_html_to_riv::{compile, CompileInput, OverflowClipBox, RuntimeCapability};
fn input(css: &str) -> CompileInput {
    CompileInput { html:"<div id=a><div id=b></div></div>".into(), css:css.into(), width:390.,height:320.,..Default::default() }
}
#[test]
fn clip_margin_cascade_units_and_inheritance_emit_versioned_policy() {
    for (css, expected) in [
        ("#a{overflow:clip;overflow-clip-margin:-8px}",vec![(OverflowClipBox::PaddingBox,-8.)]),
        ("#a{overflow:clip;--m:-8px;overflow-clip-margin:var(--m)}",vec![(OverflowClipBox::PaddingBox,-8.)]),
        ("#a{overflow:clip;overflow-clip-margin:8px}",vec![(OverflowClipBox::PaddingBox,8.)]),
        ("#a{overflow:clip;overflow-clip-margin:content-box}",vec![(OverflowClipBox::ContentBox,0.)]),
        ("#a{overflow:clip;font-size:10px;overflow-clip-margin:2em border-box;font-size:20px}",vec![(OverflowClipBox::BorderBox,40.)]),
        ("#a{overflow:clip;overflow-clip-margin:content-box 1rem}#b{overflow:clip;overflow-clip-margin:inherit}",vec![(OverflowClipBox::ContentBox,16.);2]),
        ("#a{overflow:clip;--m:content-box 8px;overflow-clip-margin:var(--m)}",vec![(OverflowClipBox::ContentBox,8.)]),
        ("#a{overflow:clip;overflow-clip-margin:8px!important;overflow-clip-margin:12px}",vec![(OverflowClipBox::PaddingBox,8.)]),
    ] {
        let out=compile(&input(css)).unwrap();let req=out.runtime_requirements;
        assert_eq!(req.version,21,"{css}");
        assert_eq!(req.layout_overflow_clip_margins.iter().map(|v|(v.origin,v.pixels)).collect::<Vec<_>>(),expected,"{css}");
        assert!(req.capabilities.contains(&RuntimeCapability::LayoutCssOverflowClipMarginV1));
        req.ensure_supported(&req.capabilities.iter().copied().collect::<Vec<_>>()).unwrap();
    }
}
#[test]
fn clip_margin_defaults_ignored_modes_and_invalid_variable_values() {
    for css in [
        "#a{overflow:clip;overflow-clip-margin:0}",
        "#a{overflow:clip;overflow-clip-margin:8px;overflow-clip-margin:initial}",
        "#a{overflow:clip;overflow-clip-margin:8px;overflow-clip-margin:unset}",
        "#a{overflow:hidden;overflow-clip-margin:8px}",
        "#a{overflow:clip visible;overflow-clip-margin:8px}",
        "#a{overflow:visible clip;overflow-clip-margin:content-box 8px}",
        "#a{overflow:clip hidden;overflow-clip-margin:8px}",
        "#a{overflow:clip;--m:10%;overflow-clip-margin:var(--m)}",
        "#a{overflow:clip;overflow-clip-margin:var(--missing)}",
    ] {
        let out=compile(&input(css)).unwrap();assert!(out.runtime_requirements.layout_overflow_clip_margins.is_empty(),"{css}");
    }
    // The property is not inherited by default.
    let out=compile(&input("#a{overflow:clip;overflow-clip-margin:8px}#b{overflow:clip}")).unwrap();
    assert_eq!(out.runtime_requirements.layout_overflow_clip_margins.len(),1);
    for value in ["5%","2","content-box padding-box","1px 2px","1deg","calc(1px + 2px)"] {
        assert!(compile(&input(&format!("#a{{overflow:clip;overflow-clip-margin:{value}}}"))).is_err(),"{value}");
    }
}
