use nuxie_html_to_riv::{LayoutLinearGradientRequirement, LinearGradientDirection as Direction,
    LinearGradientPosition as Position, LinearGradientStop as Stop, RuntimeCapability as Cap,
    RuntimeRequirements, LayoutGroupOpacityRequirement};

fn requirement() -> RuntimeRequirements {
    RuntimeRequirements { version: 26, capabilities: [Cap::LayoutCssLinearGradientV1].into(),
        layout_linear_gradients: vec![LayoutLinearGradientRequirement { object_id: 1,
            direction: Direction::Corner { right: true, bottom: false },
            stops: vec![Stop { color: 0x00ff0000, position: Some(Position::Pixels(-20.)) },
                Stop { color: 0xff0000ff, position: None },
                Stop { color: 0x8000ff00, position: Some(Position::Percent(140.)) }] }],
        ..Default::default() }
}
fn valid(r: &RuntimeRequirements) -> bool {
    r.ensure_supported(&r.capabilities.iter().copied().collect::<Vec<_>>()).is_ok()
}

#[test]
fn gradient_contract_preserves_authored_stops_and_checks_host_and_targets() {
    let r = requirement(); assert!(valid(&r));
    let value = serde_json::to_value(&r).unwrap();
    assert_eq!(value["layout_linear_gradients"][0]["stops"][0]["position"], serde_json::json!({"pixels":-20.}));
    assert_eq!(serde_json::from_value::<RuntimeRequirements>(value).unwrap(), r);
    assert_eq!(r.ensure_supported(&[]).unwrap_err().code, "missing-runtime-capability");
    assert!(r.ensure_layout_targets(|id| id == 1).is_ok());
    assert_eq!(r.ensure_layout_targets(|_| false).unwrap_err().code, "invalid-layout-linear-gradient-target");
    for version in (0..26).chain([27, u32::MAX]) {
        let mut bad = r.clone(); bad.version = version; assert!(!valid(&bad));
    }
    let mut bad = r.clone(); bad.capabilities.clear(); assert!(!valid(&bad));
    bad = r.clone(); bad.layout_linear_gradients.clear(); assert!(!valid(&bad));
    bad = r.clone(); bad.layout_linear_gradients[0].object_id = 0; assert!(!valid(&bad));
    bad = r.clone(); bad.layout_linear_gradients.push(bad.layout_linear_gradients[0].clone()); assert!(!valid(&bad));
}

#[test]
fn gradient_contract_rejects_nonfinite_values_but_retains_signed_unordered_positions() {
    let r = requirement();
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut bad = r.clone(); bad.layout_linear_gradients[0].direction = Direction::Degrees(value); assert!(!valid(&bad));
        for position in [Position::Pixels(value), Position::Percent(value)] {
            bad = r.clone(); bad.layout_linear_gradients[0].stops[0].position = Some(position); assert!(!valid(&bad));
        }
    }
    for count in [0, 1, 2, 256, 257] {
        let mut changed = r.clone(); changed.layout_linear_gradients[0].stops = vec![r.layout_linear_gradients[0].stops[0].clone(); count];
        assert_eq!(valid(&changed), (2..=256).contains(&count));
    }
    let mut changed = r;
    changed.layout_linear_gradients[0].direction = Direction::Degrees(-450.);
    changed.layout_linear_gradients[0].stops[1].position = Some(Position::Pixels(-100.));
    assert!(valid(&changed));
}

#[test]
fn gradient_version_coexists_with_opacity_without_weakening_older_contracts() {
    let mut r = requirement();
    r.layout_group_opacity.push(LayoutGroupOpacityRequirement { object_id: 1, opacity: 0.5 });
    r.capabilities.insert(Cap::LayoutCssGroupOpacityV1); assert!(valid(&r));
    r.layout_linear_gradients.clear(); r.capabilities.remove(&Cap::LayoutCssLinearGradientV1); assert!(!valid(&r));
    r.version = 25; assert!(valid(&r));
    r.layout_group_opacity[0].opacity = 1.; assert!(!valid(&r));
}

#[test]
fn gradient_transport_rejects_unknown_and_ambiguous_payloads() {
    let value = serde_json::to_value(requirement()).unwrap();
    for direction in [serde_json::json!({"degrees":20,"corner":{"right":true,"bottom":true}}),
        serde_json::json!({"corner":{"right":true,"bottom":true,"extra":1}}), serde_json::json!({"degrees":"20"})] {
        let mut bad = value.clone(); bad["layout_linear_gradients"][0]["direction"] = direction;
        assert!(serde_json::from_value::<RuntimeRequirements>(bad).is_err());
    }
    let mut bad = value; bad["layout_linear_gradients"][0]["stops"][0]["extra"] = true.into();
    assert!(serde_json::from_value::<RuntimeRequirements>(bad).is_err());
    assert!(serde_json::to_value(RuntimeRequirements::default()).unwrap().get("layout_linear_gradients").is_none());
}

#[test]
fn public_gradient_emission_matches_independent_reference_paints() {
    use nuxie_html_to_riv::{compile,CompileInput};
    let oracle: serde_json::Value = serde_json::from_str(include_str!("assets/linear-gradient-initial-oracle.json")).unwrap();
    for case in oracle["cases"].as_array().unwrap() {
        let output = compile(&CompileInput { html:case["html"].as_str().unwrap().into(),
            css:case["css"].as_str().unwrap().into(),width:390.,height:320.,..Default::default() }).unwrap();
        let r=&output.runtime_requirements; assert_eq!(r.version,26); assert!(valid(r));
        let id=output.source_map.iter().find(|n|n.id=="gradient").unwrap().object_id;
        assert!(r.layout_pixel_bounds.contains(&id));
        assert_eq!(r.layout_linear_gradients.len(),1);
        let g=&r.layout_linear_gradients[0];assert_eq!(g.object_id,id);
        let spec=&case["diagnosticGradient"];
        let expected=if let Some(v)=spec["direction"]["degrees"].as_f64() {Direction::Degrees((v as f32).rem_euclid(360.))}
            else {Direction::Corner {right:spec["direction"]["corner"][0].as_bool().unwrap(),bottom:spec["direction"]["corner"][1].as_bool().unwrap()}};
        assert_eq!(g.direction,expected,"{}",case["name"]);
        assert_eq!(g.stops.len(),spec["colors"].as_array().unwrap().len());
        for (i,stop) in g.stops.iter().enumerate() {
            assert_eq!(u64::from(stop.color),spec["colors"][i].as_u64().unwrap());
            let position=&spec["positions"][i];
            let expected=if position.is_null(){None}else if let Some(v)=position["pixels"].as_f64(){Some(Position::Pixels(v as f32))}
                else {Some(Position::Percent(position["percent"].as_f64().unwrap() as f32))};
            assert_eq!(stop.position,expected);
        }
    }
}

#[test]
fn public_gradient_resolves_current_color_relative_lengths_and_cascade() {
    use nuxie_html_to_riv::{compile,CompileInput};
    let output=compile(&CompileInput {html:"<div id=a><div id=b></div></div>".into(),
        css:"#a{color:lime;font-size:20px;background-image:linear-gradient(currentColor 2em,blue 2rem);opacity:.5}#b{color:red;font-size:10px;background-image:inherit}".into(),width:390.,height:320.,..Default::default()}).unwrap();
    let r=&output.runtime_requirements; assert!(valid(r));assert_eq!(r.version,26);
    assert_eq!(r.layout_group_opacity.len(),1);assert_eq!(r.layout_linear_gradients.len(),2);
    for (g,color) in r.layout_linear_gradients.iter().zip([0xff00ff00,0xffff0000]) {
        assert_eq!(g.stops[0],Stop{color,position:Some(Position::Pixels(40.))});
        assert_eq!(g.stops[1].position,Some(Position::Pixels(32.)));
    }
    for reset in ["none","initial","unset"] {
        let output=compile(&CompileInput {html:"<div></div>".into(),css:format!("div{{background-image:linear-gradient(red,blue);background-image:{reset}}}"),width:390.,height:320.,..Default::default()}).unwrap();
        assert!(output.runtime_requirements.layout_linear_gradients.is_empty());
    }
}
