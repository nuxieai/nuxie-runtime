use nuxie_html_to_riv::{CompileInput, CompileOutput, RuntimeCapability, compile};
fn css(value: &str) -> CompileOutput {
    compile(&CompileInput{html:"<div id=root><div id=a><div id=b></div></div></div>".into(),css:format!("#root{{width:100%;height:240px;flex-direction:row}}#a,#b{{width:80px;height:40px}}{value}"),width:390.,height:320.,..Default::default()}).unwrap()
}
fn equivalent(a: &str, b: &str) {
    let a = css(a);
    let b = css(b);
    assert_eq!(a.riv, b.riv);
    assert_eq!(a.runtime_requirements, b.runtime_requirements);
    assert_eq!(
        serde_json::to_value(a.source_map).unwrap(),
        serde_json::to_value(b.source_map).unwrap()
    );
}
#[test]
fn independent_factors_preserve_shorthand_variables_and_cascade() {
    for (grow, shrink) in [(0, 1), (1, 0), (2, 1), (1, 2)] {
        equivalent(
            &format!("#a{{flex:{grow} {shrink} 70px}}"),
            &format!("#a{{flex-grow:{grow};flex-shrink:{shrink};flex-basis:70px}}"),
        );
        equivalent(
            &format!("#a{{--f:{grow} {shrink} 70px;flex:var(--f)}}"),
            &format!("#a{{flex:{grow} {shrink} 70px}}"),
        );
    }
    equivalent("#a{flex:2 1 70px}#b{flex:inherit}", "#a,#b{flex:2 1 70px}");
    equivalent("#a{flex:initial}", "#a{flex:0 1 auto}");
    equivalent("#a{flex:unset}", "#a{flex:0 1 auto}");
    equivalent("#a{flex:2 1 70px!important;flex:none}", "#a{flex:2 1 70px}");
}
#[test]
fn nested_older_policies_do_not_downgrade_independent_factor_contract() {
    let out =
        css("#a{flex:2 1 70px}#b{justify-content:space-evenly;flex-wrap:wrap;align-self:center}");
    assert_eq!(out.runtime_requirements.version, 9);
    assert_eq!(out.runtime_requirements.layout_flex_factors.len(), 1);
    assert!(
        out.runtime_requirements
            .capabilities
            .contains(&RuntimeCapability::LayoutCssFlexFactorsV1)
    );
    out.runtime_requirements
        .ensure_supported(
            &out.runtime_requirements
                .capabilities
                .iter()
                .copied()
                .collect::<Vec<_>>(),
        )
        .unwrap();
    assert!(!out.runtime_requirements.layout_justify_content.is_empty());
    assert!(!out.runtime_requirements.layout_align_content.is_empty());
}
