use nuxie_html_to_riv::{compile, CompileInput, LayoutStackingRequirement, RuntimeCapability};

fn source(extra: &str) -> CompileInput {
    CompileInput {
        html: "<div id=host><div id=a></div><div id=b></div></div>".into(),
        css: format!("#host{{width:100%;height:200px;z-index:3}}#a,#b{{width:40px;height:40px}}{extra}"),
        width: 390., height: 320., ..Default::default()
    }
}
#[test]
fn stacking_syntax_cascade_and_contract() {
    let levels = |css: &str| {
        let output = compile(&source(css)).unwrap();
        assert_eq!(output.runtime_requirements.version, 19);
        let caps = output.runtime_requirements.capabilities.iter().copied().collect::<Vec<_>>();
        output.runtime_requirements.ensure_supported(&caps).unwrap();
        output.runtime_requirements.layout_stacking.iter().map(|entry| {
            (output.source_map.iter().find(|n| n.object_id == entry.object_id).unwrap().id.clone(), entry.level)
        }).collect::<Vec<_>>()
    };
    assert_eq!(levels(""), vec![("host".into(), 3)]);
    assert_eq!(levels("#a{z-index:inherit}"), levels("#a{z-index:3}"));
    assert_eq!(levels("#a{--z:-2;z-index:var(--z)}"), levels("#a{z-index:-2}"));
    assert_eq!(levels("#a{z-index:+0002}"), levels("#a{z-index:2}"));
    for reset in ["auto", "initial", "unset", "var(--missing)"] {
        assert_eq!(levels(&format!("#a{{z-index:{reset}}}")), levels(""));
    }
    assert_ne!(levels("#a{z-index:0}"), levels("#a{z-index:auto}"));
    assert_eq!(levels("#a{z-index:-2147483648}#b{z-index:2147483647}").len(), 3);
    for invalid in ["1.5", "1px", "1%", "1 2", "calc(1 + 1)"] {
        assert!(compile(&source(&format!("#a{{z-index:{invalid}}}"))).is_err(), "{invalid}");
    }
    for position in ["static", "relative", "absolute"] {
        let output = compile(&source(&format!("#a{{position:{position};z-index:-1}}"))).unwrap();
        let requirements = output.runtime_requirements;
        requirements.ensure_supported(&requirements.capabilities.iter().copied().collect::<Vec<_>>()).unwrap();
    }
}
#[test]
fn stacking_contract_rejects_missing_capability_and_invalid_metadata() {
    let output = compile(&source("#a{z-index:0}")).unwrap();
    let valid = output.runtime_requirements;
    let caps = valid.capabilities.iter().copied().collect::<Vec<_>>();
    assert!(valid.ensure_supported(&caps.iter().copied().filter(|c| *c != RuntimeCapability::LayoutCssStackingV1).collect::<Vec<_>>()).is_err());
    for mode in 0..5 {
        let mut bad = valid.clone();
        match mode {
            0 => bad.version = 18,
            1 => bad.layout_stacking.clear(),
            2 => { bad.capabilities.remove(&RuntimeCapability::LayoutCssStackingV1); },
            3 => bad.layout_stacking.push(bad.layout_stacking[0].clone()),
            _ => bad.layout_stacking.push(LayoutStackingRequirement { object_id: 0, level: 1 }),
        }
        assert!(bad.ensure_supported(&caps).is_err(), "mode {mode}");
    }
    valid.ensure_supported(&caps).unwrap();
}
