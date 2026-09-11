use nuxie_html_to_riv::{compile, CompileInput, RuntimeCapability, RuntimeRequirements};

fn candidate() -> RuntimeRequirements {
    let requirements = compile(&CompileInput {
        html: "<div id=root><div id=card></div></div>".into(),
        css: "#card{position:absolute}".into(),
        width: 390.0, height: 320.0, ..Default::default()
    }).unwrap().runtime_requirements;
    assert_eq!(requirements.version, 18);
    assert_eq!(requirements.layout_absolute, requirements.layout_positioned);
    requirements
}

#[test]
fn absolute_contract_roundtrips_and_requires_host_capability() {
    let requirements = candidate();
    let supported = requirements.capabilities.iter().copied().collect::<Vec<_>>();
    requirements.ensure_supported(&supported).unwrap();
    let encoded = serde_json::to_vec(&requirements).unwrap();
    let decoded: RuntimeRequirements = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(requirements, decoded);
    let old_host = requirements.capabilities.iter().copied()
        .filter(|cap| *cap != RuntimeCapability::LayoutCssAbsolutePositionV1).collect::<Vec<_>>();
    assert_eq!(requirements.ensure_supported(&old_host).unwrap_err().code, "missing-runtime-capability");
}

#[test]
fn absolute_contract_rejects_inconsistent_or_unpositioned_targets() {
    let requirements = candidate();
    let supported = requirements.capabilities.iter().copied().collect::<Vec<_>>();
    for mode in 0..6 {
        let mut malformed = requirements.clone();
        match mode {
            0 => malformed.version = 17,
            1 => malformed.layout_absolute.clear(),
            2 => malformed.layout_absolute.push(malformed.layout_absolute[0]),
            3 => malformed.layout_absolute[0] = 0,
            4 => malformed.layout_absolute[0] = u32::MAX,
            _ => { malformed.capabilities.remove(&RuntimeCapability::LayoutCssAbsolutePositionV1); }
        }
        assert_eq!(malformed.ensure_supported(&supported).unwrap_err().code,
            "invalid-layout-absolute-position", "mode {mode}");
    }
    assert!(requirements.ensure_layout_targets(|_| false).is_err());
}

#[test]
fn absolute_cascade_offsets_and_static_reset_emit_consistent_requirements() {
    let make = |css: &str| compile(&CompileInput {
        html: "<div id=root><div id=card></div></div>".into(), css: css.into(),
        width: 390.0, height: 320.0, ..Default::default()
    }).unwrap();
    for (left, right) in [
        ("#card{position:absolute;inset:5px 10%}", "#card{position:absolute;top:5px;right:10%;bottom:5px;left:10%}"),
        ("#card{--p:absolute;position:var(--p)}", "#card{position:absolute}"),
        ("#root{position:absolute}#card{position:inherit}", "#root,#card{position:absolute}"),
        ("#card{position:absolute;left:10px;position:initial}", ""),
    ] {
        let a=make(left); let b=make(right);
        assert_eq!(a.riv,b.riv);
        assert_eq!(a.runtime_requirements,b.runtime_requirements);
    }
    let inherited=make("#root{position:absolute}");
    assert_eq!(inherited.runtime_requirements.layout_absolute.len(),1);
}
