use nuxie_html_to_riv::{compile, CompileInput, CompileOutput};

fn compiled(extra: &str) -> CompileOutput {
    compile(&CompileInput {
        html: "<div id=root><div id=a><div id=inner></div></div><div id=b><div id=other></div></div></div>".into(),
        css: format!("#root{{width:100%;height:180px;flex-direction:row;gap:8px}}#inner{{width:40px;height:20px}}#other{{width:100px;height:30px}}{extra}"),
        width:390.,height:320.,..Default::default()
    }).unwrap()
}

fn equivalent(left: &str, right: &str) {
    let (left,right) = (compiled(left),compiled(right));
    assert_eq!(left.riv,right.riv);
    assert_eq!(left.runtime_requirements,right.runtime_requirements);
    assert_eq!(serde_json::to_value(left.source_map).unwrap(),serde_json::to_value(right.source_map).unwrap());
}

#[test]
fn content_auto_basis_preserves_cascade_and_independent_weights() {
    equivalent("#a{flex:auto}","#a{flex-grow:1;flex-shrink:1;flex-basis:auto}");
    equivalent("#a{flex:initial}","#a{flex:0 1 auto}");
    equivalent("#a{flex:2 .5 auto}","#a{--basis:auto;flex-grow:2;flex-shrink:.5;flex-basis:var(--basis)}");
    equivalent("#a{flex:auto!important;flex:none}","#a{flex:auto}");
    equivalent("#root{flex:2 1 auto}#a{flex:inherit}","#root,#a{flex:2 1 auto}");
    equivalent("#a{flex:1 1 30px;flex-basis:unset}","#a{flex:auto}");
    let output=compiled("#a{flex:auto}#b{flex:2 .5 auto}");
    for (id,grow,shrink) in [("a",1.,1.),("b",2.,0.5)] {
        let object_id=output.source_map.iter().find(|node|node.id==id).unwrap().object_id;
        let policy=output.runtime_requirements.layout_flex_factors.iter().find(|entry|entry.object_id==object_id).unwrap();
        assert_eq!((policy.grow,policy.shrink),(grow,shrink));
    }
    let capabilities=output.runtime_requirements.capabilities.iter().copied().collect::<Vec<_>>();
    output.runtime_requirements.ensure_supported(&capabilities).unwrap();
}

#[test]
fn intrinsic_sizing_requires_checked_host_and_valid_targets() {
    use nuxie_html_to_riv::RuntimeCapability;
    let good = compiled("#a{flex:auto}#b{flex:.25 .5 auto}").runtime_requirements;
    assert_eq!(good.version, 11);
    assert_eq!(good.layout_intrinsic_sizing.len(), 2);
    let capabilities = good.capabilities.iter().copied().collect::<Vec<_>>();
    let old_host = capabilities.iter().copied().filter(|c| *c != RuntimeCapability::LayoutCssIntrinsicSizingV1).collect::<Vec<_>>();
    assert!(good.ensure_supported(&old_host).is_err());
    for version in 1..=10 {
        let mut bad=good.clone();bad.version=version;
        assert_eq!(bad.ensure_supported(&capabilities).unwrap_err().code,"invalid-layout-intrinsic-sizing");
    }
    let mut bad=good.clone();bad.layout_intrinsic_sizing.clear();
    assert!(bad.ensure_supported(&capabilities).is_err());
    let mut bad=good.clone();bad.capabilities.remove(&RuntimeCapability::LayoutCssIntrinsicSizingV1);
    assert!(bad.ensure_supported(&capabilities).is_err());
    let mut bad=good.clone();bad.layout_intrinsic_sizing.push(bad.layout_intrinsic_sizing[0]);
    assert!(bad.ensure_supported(&capabilities).is_err());
    assert_eq!(good.ensure_layout_targets(|_| false).unwrap_err().code,"invalid-layout-intrinsic-sizing-target");
    let serialized=serde_json::to_vec(&good).unwrap();
    assert_eq!(serde_json::from_slice::<nuxie_html_to_riv::RuntimeRequirements>(&serialized).unwrap(),good);
}
