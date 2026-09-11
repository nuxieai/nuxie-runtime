use nuxie_html_to_riv::{compile, CompileInput};

#[test]
fn historical_indefinite_overflow_reproducer_requires_corrected_runtime() {
    let cases: serde_json::Value = serde_json::from_str(include_str!(
        "../validation/indefinite-basis-cases.json")).unwrap();
    let case = &cases[0];
    let output = compile(&CompileInput {
        html: case["html"].as_str().unwrap().into(),
        css: case["css"].as_str().unwrap().into(),
        width: 390., height: 320., ..Default::default()
    }).unwrap();
    assert_eq!(output.runtime_requirements.version, 12);
    assert!(output.runtime_requirements.capabilities.contains(
        &nuxie_html_to_riv::RuntimeCapability::LayoutCssIndefiniteBasisV1));
}

#[test]
fn indefinite_percentage_cascade_emits_version12_without_child_downgrade() {
    use nuxie_html_to_riv::{Asset, RuntimeCapability};
    let input = |declaration| {
        let mut input = CompileInput {
            html: "<div id=host><div id=row><div id=child><p id=text>Measured words</p></div></div></div>".into(),
            css: format!("#host{{width:100%;align-items:flex-start}}#row{{flex-direction:row}}#child{{{declaration}}}"),
            width: 390., height: 320., ..Default::default()
        };
        input.assets.insert("inter".into(), Asset::Font {
            family: "Inter".into(), weight: 400,
            bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
        });
        input
    };
    let base = compile(&input("flex:1 1 30%")).unwrap();
    for declaration in ["flex-grow:1;flex-shrink:1;flex-basis:30%",
        "--basis:30%;flex:1 1 var(--basis)", "flex:1 1 30%!important;flex:none"] {
        let alternate = compile(&input(declaration)).unwrap();
        assert_eq!(alternate.riv, base.riv);
        assert_eq!(alternate.runtime_requirements, base.runtime_requirements);
    }
    assert_eq!(base.runtime_requirements.version, 12);
    assert!(base.runtime_requirements.capabilities.contains(&RuntimeCapability::LayoutCssIndefiniteBasisV1));
    assert!(!base.runtime_requirements.layout_intrinsic_sizing.is_empty());
    let old_host = base.runtime_requirements.capabilities.iter().copied()
        .filter(|c| *c != RuntimeCapability::LayoutCssIndefiniteBasisV1).collect::<Vec<_>>();
    assert_eq!(base.runtime_requirements.ensure_supported(&old_host).unwrap_err().code,
        "missing-runtime-capability");
    let zero_percent = compile(&input("flex:1 1 0%")).unwrap();
    let zero_pixels = compile(&input("flex:1 1 0px")).unwrap();
    assert_ne!(zero_percent.riv, zero_pixels.riv);
    assert!(!zero_pixels.runtime_requirements.capabilities.contains(&RuntimeCapability::LayoutCssIndefiniteBasisV1));
}

#[test]
fn indefinite_basis_contract_rejects_old_hosts_and_inconsistent_versions() {
    use nuxie_html_to_riv::{RuntimeCapability, RuntimeRequirements};
    let capability = RuntimeCapability::LayoutCssIndefiniteBasisV1;
    let mut requirements = RuntimeRequirements::default();
    requirements.version = 12;
    requirements.capabilities.insert(capability);
    requirements.ensure_supported(&[capability]).unwrap();
    assert_eq!(requirements.ensure_supported(&[]).unwrap_err().code,
        "missing-runtime-capability");
    for version in 1..=11 {
        let mut invalid = requirements.clone();
        invalid.version = version;
        assert_eq!(invalid.ensure_supported(&[capability]).unwrap_err().code,
            "invalid-layout-indefinite-basis");
    }
    let mut invalid = requirements.clone();
    invalid.capabilities.clear();
    assert_eq!(invalid.ensure_supported(&[capability]).unwrap_err().code,
        "invalid-layout-indefinite-basis");
    requirements.layout_intrinsic_sizing.push(3);
    assert!(requirements.ensure_supported(&[capability]).is_err());
    requirements.capabilities.insert(RuntimeCapability::LayoutCssIntrinsicSizingV1);
    let supported = [capability, RuntimeCapability::LayoutCssIntrinsicSizingV1];
    requirements.ensure_supported(&supported).unwrap();
    requirements.layout_intrinsic_sizing.push(3);
    assert_eq!(requirements.ensure_supported(&supported).unwrap_err().code,
        "invalid-layout-intrinsic-sizing");
}

#[test]
fn equal_percentage_factors_preserve_authored_main_dimension() {
    for factor in ["0", ".5", "1"] {
        let input = |width| CompileInput {
            html: "<div id=root><div id=child><div id=inner></div></div></div>".into(),
            css: format!("#root{{width:100%;flex-direction:row}}#child{{width:{width}px;flex:{factor} {factor} 30%}}#inner{{width:80px;height:20px}}"),
            width: 390., height: 320., ..Default::default()
        };
        let first = compile(&input(110)).unwrap();
        let second = compile(&input(120)).unwrap();
        // Even with a definite current basis, authored width is an independent
        // intrinsic sizing input; compilation must not erase it.
        assert_ne!(first.riv, second.riv, "factor={factor}");
        let child = first.source_map.iter().find(|node| node.id == "child").unwrap();
        let factors = first.runtime_requirements.layout_flex_factors.iter()
            .find(|entry| entry.object_id == child.object_id).unwrap();
        let expected: f32 = factor.parse().unwrap();
        assert_eq!((factors.grow, factors.shrink), (expected, expected));
        let capabilities = first.runtime_requirements.capabilities.iter().copied().collect::<Vec<_>>();
        first.runtime_requirements.ensure_supported(&capabilities).unwrap();
    }
}
