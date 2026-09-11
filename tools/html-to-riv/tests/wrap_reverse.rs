use nuxie_html_to_riv::{compile, CompileInput, CompileOutput, RuntimeCapability};
fn css(value: &str) -> CompileOutput {
    compile(&CompileInput {html:"<div id=root><div id=a></div></div>".into(),css:value.into(),width:390.,height:320.,..Default::default()}).unwrap()
}
fn equivalent(a: &str,b: &str) {
    let a=css(a);let b=css(b);
    assert_eq!(a.riv,b.riv);assert_eq!(serde_json::to_value(a.source_map).unwrap(),serde_json::to_value(b.source_map).unwrap());
    assert_eq!(a.runtime_requirements,b.runtime_requirements);
}
#[test]
fn wrapping_modes_preserve_cascade_and_line_policy() {
    for mode in ["nowrap","wrap","wrap-reverse"] {
        equivalent(&format!("#root{{flex-wrap:{mode}}}"),&format!("#root{{--w:{mode};flex-wrap:var(--w)}}"));
        equivalent(&format!("#root{{flex-wrap:{mode}}}#a{{flex-wrap:inherit}}"),&format!("#root,#a{{flex-wrap:{mode}}}"));
    }
    let reverse=css("#root{flex-wrap:wrap-reverse}");
    assert_eq!(reverse.runtime_requirements.version,7);
    assert_eq!(reverse.runtime_requirements.layout_align_content.len(),1);
    assert!(reverse.runtime_requirements.capabilities.contains(&RuntimeCapability::LayoutCssAlignContentV1));
    assert_ne!(reverse.riv,css("#root{flex-wrap:wrap}").riv);
    for reset in ["initial","unset","var(--missing)","var(--bad)"] {
        equivalent(&format!("#root{{--bad:7px;flex-wrap:wrap-reverse;flex-wrap:{reset}}}"),"#root{flex-wrap:nowrap}");
    }
    equivalent("#root{flex-wrap:wrap-reverse}#a{flex-wrap:initial}","#root{flex-wrap:wrap-reverse}");
    equivalent("#root{flex-wrap:wrap-reverse!important;flex-wrap:wrap}","#root{flex-wrap:wrap-reverse}");
}
#[test]
fn wrapping_rejects_invalid_syntax_and_composes_with_distribution() {
    for value in ["reverse","wrap reverse","wrap-reverse nowrap","safe wrap-reverse"] {
        let result=compile(&CompileInput{html:"<div></div>".into(),css:format!("div{{flex-wrap:{value}}}"),width:390.,height:320.,..Default::default()});
        assert!(result.is_err(),"{value}");
    }
    let output=css("#root{flex-wrap:wrap-reverse;justify-content:space-evenly;align-content:space-around}#a{align-self:flex-end;order:-1}");
    assert_eq!(output.runtime_requirements.version,8);
    let supported=output.runtime_requirements.capabilities.iter().copied().collect::<Vec<_>>();
    output.runtime_requirements.ensure_supported(&supported).unwrap();
}
