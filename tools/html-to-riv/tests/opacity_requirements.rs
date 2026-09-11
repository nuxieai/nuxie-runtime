use nuxie_html_to_riv::{LayoutGroupOpacityRequirement, RuntimeCapability, RuntimeRequirements,
    LayoutCornerRadiiRequirement, CornerRadiusValue};

fn requirement() -> RuntimeRequirements {
    RuntimeRequirements { version: 25,
        capabilities: [RuntimeCapability::LayoutCssGroupOpacityV1].into(),
        layout_group_opacity: vec![LayoutGroupOpacityRequirement { object_id: 1, opacity: 0.5 }],
        ..Default::default() }
}
fn valid(r: &RuntimeRequirements) -> bool {
    r.ensure_supported(&r.capabilities.iter().copied().collect::<Vec<_>>()).is_ok()
}

#[test]
fn opacity_contract_checks_version_capability_targets_and_normalized_alpha() {
    let r = requirement(); assert!(valid(&r));
    assert_eq!(r.ensure_supported(&[]).unwrap_err().code, "missing-runtime-capability");
    assert!(r.ensure_layout_targets(|id| id == 1).is_ok());
    assert_eq!(r.ensure_layout_targets(|_| false).unwrap_err().code, "invalid-layout-group-opacity-target");
    for version in (0..25).chain([26, u32::MAX]) {
        let mut bad = r.clone(); bad.version = version; assert!(!valid(&bad), "{version}");
    }
    for alpha in [-1., 1., 2., f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut bad = r.clone(); bad.layout_group_opacity[0].opacity = alpha; assert!(!valid(&bad), "{alpha}");
    }
    for alpha in [0., f32::MIN_POSITIVE, 0.99999994] {
        let mut good = r.clone(); good.layout_group_opacity[0].opacity = alpha; assert!(valid(&good));
    }
    let mut bad = r.clone(); bad.capabilities.clear(); assert!(!valid(&bad));
    bad = r.clone(); bad.layout_group_opacity.clear(); assert!(!valid(&bad));
    bad = r.clone(); bad.layout_group_opacity[0].object_id = 0; assert!(!valid(&bad));
    bad = r.clone(); bad.layout_group_opacity.push(bad.layout_group_opacity[0].clone()); assert!(!valid(&bad));
}

#[test]
fn opacity_transport_roundtrips_and_rejects_ambiguous_entries() {
    let r = requirement(); let value = serde_json::to_value(&r).unwrap();
    assert_eq!(value["capabilities"], serde_json::json!(["layout-css-group-opacity-v1"]));
    assert_eq!(value["layout_group_opacity"], serde_json::json!([{"object_id":1,"opacity":0.5}]));
    assert_eq!(serde_json::from_value::<RuntimeRequirements>(value.clone()).unwrap(), r);
    for entry in [serde_json::json!({"object_id":1}), serde_json::json!({"object_id":1,"opacity":"0.5"}),
        serde_json::json!({"object_id":1,"opacity":0.5,"extra":true})] {
        let mut bad = value.clone(); bad["layout_group_opacity"] = serde_json::json!([entry]);
        assert!(serde_json::from_value::<RuntimeRequirements>(bad).is_err());
    }
    assert!(serde_json::to_value(RuntimeRequirements::default()).unwrap().get("layout_group_opacity").is_none());
}

#[test]
fn opacity_coexists_with_elliptical_corners_without_weakening_old_versions() {
    let mut r = requirement();
    r.layout_corner_radii.push(LayoutCornerRadiiRequirement { object_id: 1,
        radii: [[CornerRadiusValue::Pixels(12.), CornerRadiusValue::Percent(50.)]; 4] });
    r.capabilities.insert(RuntimeCapability::LayoutCssCornerRadiiV1);
    assert!(valid(&r));
    r.layout_group_opacity.clear(); r.capabilities.remove(&RuntimeCapability::LayoutCssGroupOpacityV1);
    assert!(!valid(&r));
    r.version = 24; assert!(valid(&r));
    r.capabilities.insert(RuntimeCapability::LayoutCssGroupOpacityV1); assert!(!valid(&r));
}


#[test]
fn public_opacity_emits_computed_groups_without_changing_rive_geometry() {
    use nuxie_html_to_riv::{compile, CompileInput};
    let input = CompileInput { html: "<div id=a><div id=b></div></div>".into(),
        css: "#a{width:75%;height:120px;background:blue}#b{width:80px;height:80px;background:red}".into(),
        width:390., height:320., ..Default::default() };
    let baseline = compile(&input).unwrap();
    for (value, alpha) in [("0",0.), (".5",0.5), ("50%",0.5), ("-2",0.), ("var(--alpha, .25)",0.25)] {
        let mut request = input.clone(); request.css += &format!("#a{{opacity:{value}}}#b{{opacity:inherit}}");
        let output = compile(&request).unwrap();
        assert_eq!(output.riv, baseline.riv, "opacity must not change geometry or paint alpha");
        assert_eq!(output.runtime_requirements.version,25);
        let expected = ["a","b"].map(|id| LayoutGroupOpacityRequirement {
            object_id:output.source_map.iter().find(|n| n.id == id).unwrap().object_id, opacity:alpha });
        assert_eq!(output.runtime_requirements.layout_group_opacity,expected);
        assert!(valid(&output.runtime_requirements));
    }
    for value in ["1", "150%", "initial", "unset"] {
        let mut request=input.clone(); request.css += &format!("#a{{opacity:{value}}}");
        let output=compile(&request).unwrap();
        assert_eq!(output.riv,baseline.riv);
        assert_eq!(output.runtime_requirements,baseline.runtime_requirements);
    }
}

#[test]
fn public_opacity_cascade_preserves_noninheritance_and_rejects_excluded_math() {
    use nuxie_html_to_riv::{compile, CompileInput};
    let mut input=CompileInput { html:"<div id=a><div id=b></div></div>".into(),
        css:"#a{opacity:25%!important}#a{opacity:.8}#b{opacity:var(--missing, 2px)}".into(),
        width:240.,height:320.,..Default::default() };
    let output=compile(&input).unwrap();
    assert_eq!(output.runtime_requirements.layout_group_opacity.len(),1);
    assert_eq!(output.runtime_requirements.layout_group_opacity[0].opacity,0.25);
    for value in ["calc(.5)","var(--alpha, calc(.5))",".5 .2","2px"] {
        input.css=format!("#a{{opacity:{value}}}");
        assert!(compile(&input).is_err(),"{value}");
    }
}


#[test]
fn public_opacity_preserves_authored_precision_until_clamping() {
    use nuxie_html_to_riv::{compile, CompileInput};
    for (value, expected) in [("1e100",1.),("1e100%",1.),("-1e100",0.),
        ("99.999997%",(99.999997_f64/100.) as f32)] {
        for authored in [value.to_string(),format!("var(--alpha, {value})")] {
            let output=compile(&CompileInput { html:"<div></div>".into(),css:format!("div{{opacity:{authored}}}"),
                width:240.,height:320.,..Default::default() }).unwrap();
            let groups=&output.runtime_requirements.layout_group_opacity;
            if expected==1. { assert!(groups.is_empty(),"{authored}"); }
            else { assert_eq!(groups[0].opacity,expected,"{authored}"); }
        }
    }
}
