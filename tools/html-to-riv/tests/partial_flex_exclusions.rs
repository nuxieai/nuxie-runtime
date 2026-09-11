use nuxie_html_to_riv::{compile, CompileInput, RuntimeCapability};

fn input(css: &str) -> CompileInput {
    CompileInput {html:"<div id=root><div id=a></div></div>".into(),
        css:format!("#root{{width:100%;height:200px;flex-direction:row}}#a{{width:80px;height:40px;flex-basis:20px}}{css}"),
        width:390.,height:320.,..Default::default()}
}

#[test]
fn partial_factor_admission_keeps_numeric_bounds_and_basis_exclusions() {
    for value in ["0", "0.0001", ".125", ".9999", "1", "10000"] {
        let output=compile(&input(&format!("#a{{flex-grow:{value};flex-shrink:1}}"))).unwrap();
        let number=value.parse::<f32>().unwrap();
        assert_eq!(output.runtime_requirements.capabilities.contains(&RuntimeCapability::LayoutCssPartialFlexFactorsV1),number>0. && number<1.);
    }
    for property in ["flex-grow","flex-shrink"] {
        for value in ["-1", "NaN", "infinity", "10001", "1e50", "20%", "2px", ".2 .3"] {
            assert!(compile(&input(&format!("#a{{{property}:{value}}}"))).is_err(),"{property}:{value}");
        }
    }
    let indefinite = compile(&input(
        "#root{height:auto;flex-direction:column}#a{flex:.25 .5 30%}")).unwrap();
    assert_eq!(indefinite.runtime_requirements.version, 12);
    assert!(indefinite.runtime_requirements.capabilities.contains(
        &RuntimeCapability::LayoutCssIndefiniteBasisV1));
    assert!(compile(&input("#a{flex:.25 .5 min-content}")).is_err());
}
