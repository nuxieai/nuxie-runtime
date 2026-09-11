use nuxie_html_to_riv::{CornerRadiusValue::{self, Pixels, Percent}, LayoutCornerRadiiRequirement,
    LayoutBorderRequirement, LayoutBorderSidesRequirement, RuntimeCapability, RuntimeRequirements};

fn requirement() -> RuntimeRequirements {
    RuntimeRequirements { version:24,
        capabilities:[RuntimeCapability::LayoutCssCornerRadiiV1].into(),
        layout_corner_radii:vec![LayoutCornerRadiiRequirement { object_id:1,
            radii:[[Pixels(12.),Percent(50.)],[Percent(120.),Pixels(0.)],[Pixels(8.),Pixels(16.)],[Percent(5.),Percent(10.)]] }],
        ..Default::default() }
}
fn valid(r: &RuntimeRequirements) -> bool {
    r.ensure_supported(&r.capabilities.iter().copied().collect::<Vec<_>>()).is_ok()
}

#[test]
fn corner_contract_requires_matching_version_capability_and_valid_layout_targets() {
    let r=requirement(); assert!(valid(&r));
    assert_eq!(r.ensure_supported(&[]).unwrap_err().code,"missing-runtime-capability");
    assert!(r.ensure_layout_targets(|id| id==1).is_ok());
    assert_eq!(r.ensure_layout_targets(|_|false).unwrap_err().code,"invalid-layout-corner-radii-target");
    for version in (0..24).chain([25]) {
        let mut bad=r.clone(); bad.version=version; assert!(!valid(&bad),"version {version}");
    }
    let mut bad=r.clone(); bad.capabilities.clear(); assert!(!valid(&bad));
    bad=r.clone(); bad.layout_corner_radii.clear(); assert!(!valid(&bad));
    bad=r.clone(); bad.layout_corner_radii[0].object_id=0; assert!(!valid(&bad));
    bad=r.clone(); bad.layout_corner_radii.push(bad.layout_corner_radii[0].clone()); assert!(!valid(&bad));
    for corner in 0..4 { for axis in 0..2 { for invalid in [-1., f32::NAN, f32::INFINITY] {
        for value in [Pixels(invalid),Percent(invalid)] {
            let mut bad=r.clone(); bad.layout_corner_radii[0].radii[corner][axis]=value;
            assert!(!valid(&bad));
        }
    } } }
    for value in [Pixels(0.),Percent(0.),Pixels(f32::MAX),Percent(f32::MAX)] {
        let mut r=r.clone(); r.layout_corner_radii[0].radii=[[value;2];4]; assert!(valid(&r));
    }
}

#[test]
fn corner_transport_roundtrips_and_rejects_ambiguous_axes_and_wrong_shapes() {
    let r=requirement(); let value=serde_json::to_value(&r).unwrap();
    assert_eq!(value["layout_corner_radii"][0]["radii"][0],serde_json::json!([{ "pixels":12. },{"percent":50.}]));
    assert_eq!(serde_json::from_value::<RuntimeRequirements>(value.clone()).unwrap(),r);
    for axis in [serde_json::json!({"pixels":1,"percent":2}),serde_json::json!({"em":1}),
        serde_json::json!({"pixels":null}),serde_json::json!({"pixels":"1"}),serde_json::json!({}),
        serde_json::json!(12),serde_json::json!([12,20])] {
        let mut bad=value.clone(); bad["layout_corner_radii"][0]["radii"][0][0]=axis;
        assert!(serde_json::from_value::<RuntimeRequirements>(bad).is_err());
    }
    for count in [0,1,3,5] {
        let mut bad=value.clone(); bad["layout_corner_radii"][0]["radii"]=serde_json::json!(vec![[Pixels(0.);2];count]);
        assert!(serde_json::from_value::<RuntimeRequirements>(bad).is_err());
    }
    for count in [0,1,3] {
        let mut bad=value.clone(); bad["layout_corner_radii"][0]["radii"][0]=serde_json::json!(vec![Percent(0.);count]);
        assert!(serde_json::from_value::<RuntimeRequirements>(bad).is_err());
    }
    let mut bad=value.clone(); bad["layout_corner_radii"][0]["extra"]=true.into();
    assert!(serde_json::from_value::<RuntimeRequirements>(bad).is_err());
    let mut bad=value; bad["layout_corner_radii"][0].as_object_mut().unwrap().remove("radii");
    assert!(serde_json::from_value::<RuntimeRequirements>(bad).is_err());
    assert!(serde_json::to_value(RuntimeRequirements::default()).unwrap().get("layout_corner_radii").is_none());
    // Negative JSON numbers deserialize structurally, then fail semantic validation.
    assert_eq!(serde_json::from_str::<CornerRadiusValue>("{\"percent\":-1}").unwrap(),Percent(-1.));
}

#[test]
fn corners_compose_with_both_border_policies_without_relaxing_legacy_versions() {
    let mut r=requirement();
    r.layout_borders.push(LayoutBorderRequirement {object_id:1,color:0xffabcdef});
    r.layout_border_sides.push(LayoutBorderSidesRequirement {object_id:2,colors:[0xff123456;4]});
    r.capabilities.extend([RuntimeCapability::LayoutCssSolidBordersV1,RuntimeCapability::LayoutCssBorderSidesV1]);
    assert!(valid(&r)); assert!(r.ensure_layout_targets(|id|id==1||id==2).is_ok());
    r.version=23; assert!(!valid(&r));
    r.layout_corner_radii.clear(); r.capabilities.remove(&RuntimeCapability::LayoutCssCornerRadiiV1);
    assert!(valid(&r));
    r.version=24; assert!(!valid(&r));
    r.version=22; assert!(!valid(&r));
    r.layout_border_sides.clear(); r.capabilities.remove(&RuntimeCapability::LayoutCssBorderSidesV1);
    assert!(valid(&r));
}

#[test]
fn public_compiler_emits_computed_axes_and_preserves_circular_artifacts() {
    use nuxie_html_to_riv::{compile,CompileInput};
    let build=|radius:&str|compile(&CompileInput {
        html:"<div id='box'></div>".into(),
        css:format!("#box{{width:60%;height:80px;font-size:20px;border:4px solid red;border-radius:{radius};}}"),
        width:390.,height:320.,..Default::default()
    }).unwrap();
    let output=build("1em 25% / 50% .5rem");
    let r=&output.runtime_requirements;
    assert_eq!(r.version,24); assert!(valid(r));
    assert_eq!(r.layout_corner_radii,vec![LayoutCornerRadiiRequirement {
        object_id:output.source_map.iter().find(|n|n.id=="box").unwrap().object_id,
        radii:[[Pixels(20.),Percent(50.)],[Percent(25.),Pixels(8.)],[Pixels(20.),Percent(50.)],[Percent(25.),Pixels(8.)]]
    }]);
    assert_eq!(r.layout_borders.len(),1);
    // Different viewport dimensions retain unresolved percentages in the transport.
    for radius in ["0", "8px", "4px 8px 12px 16px"] {
        let circular=build(radius); let slash=build(&format!("{radius} / {radius}"));
        assert_eq!(circular.riv,slash.riv);
        assert_eq!(circular.runtime_requirements,slash.runtime_requirements);
        assert!(circular.runtime_requirements.layout_corner_radii.is_empty());
        assert_eq!(circular.runtime_requirements.version,22);
    }
}
