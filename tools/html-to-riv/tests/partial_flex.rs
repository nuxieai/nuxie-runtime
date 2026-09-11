use nuxie_html_to_riv::{compile, CompileInput, CompileOutput, RuntimeCapability};

fn css(value: &str) -> CompileOutput {
    compile(&CompileInput {html:"<div id=root><div id=a><div id=b></div></div></div>".into(),
        css:format!("#root{{width:100%;height:240px;flex-direction:row}}#a,#b{{width:80px;height:40px}}{value}"),
        width:390.,height:320.,..Default::default()}).unwrap()
}
fn equivalent(a: &str, b: &str) {
    let (a,b) = (css(a),css(b));
    assert_eq!(a.riv,b.riv);
    assert_eq!(a.runtime_requirements,b.runtime_requirements);
    assert_eq!(serde_json::to_value(a.source_map).unwrap(),serde_json::to_value(b.source_map).unwrap());
}

#[test]
fn partial_factors_preserve_cascade_and_require_corrected_host() {
    for (grow,shrink) in [(0.1,0.1),(0.25,0.25),(0.25,0.5),(0.5,0.),(1.,0.25)] {
        equivalent(&format!("#a{{flex:{grow} {shrink} 20px}}"),&format!("#a{{flex-grow:{grow};flex-shrink:{shrink};flex-basis:20px}}"));
        equivalent(&format!("#a{{--f:{grow} {shrink} 20px;flex:var(--f)}}"),&format!("#a{{flex:{grow} {shrink} 20px}}"));
        let out=css(&format!("#a{{flex:{grow} {shrink} 20px}}#b{{flex:2 1 20px}}"));
        assert_eq!(out.runtime_requirements.version,10);
        assert!(out.runtime_requirements.capabilities.contains(&RuntimeCapability::LayoutCssPartialFlexFactorsV1));
        out.runtime_requirements.ensure_supported(&out.runtime_requirements.capabilities.iter().copied().collect::<Vec<_>>()).unwrap();
    }
    equivalent("#a{flex:.25 .5 20px}#b{flex:inherit}","#a,#b{flex:.25 .5 20px}");
    equivalent("#a{flex:.25 .5 20px;flex-grow:initial}","#a{flex:0 .5 20px}");
    equivalent("#a{flex:.25 .5 20px;flex-shrink:unset}","#a{flex:.25 1 20px}");
    equivalent("#a{flex:.25 .5 20px!important;flex:none}","#a{flex:.25 .5 20px}");
    equivalent("#a{--f:.5 .5 20px;flex:var(--f);flex-shrink:0}","#a{flex:.5 0 20px}");
}

#[test]
fn partial_contract_rejects_older_or_inconsistent_hosts() {
    let good=css("#a{flex:.25 .25 20px}").runtime_requirements;
    let capabilities=good.capabilities.iter().copied().collect::<Vec<_>>();
    let old_host=capabilities.iter().copied().filter(|c| *c != RuntimeCapability::LayoutCssPartialFlexFactorsV1).collect::<Vec<_>>();
    assert!(good.ensure_supported(&old_host).is_err());
    for version in 1..=9 {
        let mut bad=good.clone();bad.version=version;
        assert!(bad.ensure_supported(&capabilities).is_err());
    }
    let mut bad=good.clone();bad.capabilities.remove(&RuntimeCapability::LayoutCssPartialFlexFactorsV1);
    assert!(bad.ensure_supported(&capabilities).is_err());
    let mut bad=good.clone();bad.layout_flex_factors[0].grow=1.;bad.layout_flex_factors[0].shrink=1.;
    assert!(bad.ensure_supported(&capabilities).is_err());
    let encoded=serde_json::to_vec(&good).unwrap();
    assert_eq!(serde_json::from_slice::<nuxie_html_to_riv::RuntimeRequirements>(&encoded).unwrap(),good);
}
