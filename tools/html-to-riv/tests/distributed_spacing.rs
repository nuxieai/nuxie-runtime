use nuxie_html_to_riv::{compile, CompileInput, RuntimeCapability, RuntimeRequirements};

fn css(source: &str) -> nuxie_html_to_riv::CompileOutput {
    compile(&CompileInput { html: "<div id=root><div id=a></div></div>".into(), css: source.into(), width:390., height:320., ..Default::default() }).unwrap()
}

#[test]
fn distributed_values_follow_cascade_and_publish_version_eight() {
    for (property, value) in [("justify-content","space-around"),("justify-content","space-evenly"),("align-content","space-evenly")] {
        let direct=css(&format!("#root{{{property}:{value}}}"));
        let variable=css(&format!("#root{{--v:{value};{property}:var(--v)}}"));
        assert_eq!(direct.riv,variable.riv);
        assert_eq!(direct.runtime_requirements,variable.runtime_requirements);
        assert_eq!(direct.runtime_requirements.version,8);
        assert!(direct.runtime_requirements.capabilities.contains(&RuntimeCapability::LayoutCssDistributedSpacingV1));
        let inherited=css(&format!("#root{{{property}:{value}}}#a{{{property}:inherit}}"));
        let count=if property=="justify-content" {inherited.runtime_requirements.layout_justify_content.len()} else {inherited.runtime_requirements.layout_align_content.len()};
        assert_eq!(count,2);
        for reset in ["initial","unset","var(--missing)","var(--bad)"] {
            let output=css(&format!("#root{{--bad:7px;{property}:{value};{property}:{reset}}}"));
            assert!(!output.runtime_requirements.capabilities.contains(&RuntimeCapability::LayoutCssDistributedSpacingV1));
        }
    }
    let combined=css("#root{justify-content:space-around}#a{align-content:center;align-self:flex-end;order:1}");
    assert_eq!(combined.runtime_requirements.version,8,"later older policies must not downgrade version");
    let supported=combined.runtime_requirements.capabilities.iter().copied().collect::<Vec<_>>();
    combined.runtime_requirements.ensure_supported(&supported).unwrap();
}

#[test]
fn distributed_contract_rejects_invalid_versions_capabilities_and_targets() {
    for source in ["#root{justify-content:space-evenly}","#root{align-content:space-evenly}","#root{justify-content:space-around;align-content:space-evenly}"] {
        let output=css(source);
        let requirements=output.runtime_requirements;
        let supported=requirements.capabilities.iter().copied().collect::<Vec<_>>();
        requirements.ensure_supported(&supported).unwrap();
        requirements.ensure_layout_targets(|id| output.source_map.iter().any(|n| n.object_id==id)).unwrap();
        assert!(requirements.ensure_layout_targets(|_| false).is_err());
        let old_host=supported.iter().copied().filter(|v| *v!=RuntimeCapability::LayoutCssDistributedSpacingV1).collect::<Vec<_>>();
        assert_eq!(requirements.ensure_supported(&old_host).unwrap_err().code,"missing-runtime-capability");
        for version in [1,6,7,9] {
            let mut invalid=requirements.clone();invalid.version=version;
            assert!(invalid.ensure_supported(&supported).is_err());
        }
        let mut missing=requirements.clone();missing.capabilities.remove(&RuntimeCapability::LayoutCssDistributedSpacingV1);
        assert!(missing.ensure_supported(&supported).is_err());
        if let Some(entry)=requirements.layout_justify_content.first() {
            let mut duplicate=requirements.clone();duplicate.layout_justify_content.push(entry.clone());
            assert!(duplicate.ensure_supported(&supported).is_err());
            let mut json=serde_json::to_value(&requirements).unwrap();json["layout_justify_content"][0]["alignment"]="center".into();
            assert!(serde_json::from_value::<RuntimeRequirements>(json).is_err());
        }
    }
    let mut empty=RuntimeRequirements::default();empty.version=8;
    assert!(empty.ensure_supported(&[]).is_err());
}
