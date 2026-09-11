use nuxie_html_to_riv::{LayoutBorderRequirement, LayoutOverflowClipMarginRequirement, OverflowClipBox, RuntimeCapability, RuntimeRequirements};

fn requirement() -> RuntimeRequirements {
    RuntimeRequirements { version:22,
        capabilities:[RuntimeCapability::LayoutCssSolidBordersV1].into(),
        layout_borders:vec![LayoutBorderRequirement { object_id:1, color:0x80563cab }],
        ..Default::default() }
}
fn valid(r: &RuntimeRequirements) -> bool {
    r.ensure_supported(&r.capabilities.iter().copied().collect::<Vec<_>>()).is_ok()
}
#[test]
fn border_contract_requires_version_capability_and_unique_layout_targets() {
    let r=requirement(); assert!(valid(&r));
    assert!(r.ensure_supported(&[]).is_err());
    assert!(r.ensure_layout_targets(|id|id==1).is_ok());
    assert!(r.ensure_layout_targets(|_|false).is_err());
    for version in [0,1,20,21,23] {
        let mut bad=r.clone();bad.version=version;assert!(!valid(&bad));
    }
    let mut bad=r.clone();bad.layout_borders.clear();assert!(!valid(&bad));
    bad=r.clone();bad.capabilities.clear();assert!(!valid(&bad));
    bad=r.clone();bad.layout_borders[0].object_id=0;assert!(!valid(&bad));
    bad=r.clone();bad.layout_borders.push(bad.layout_borders[0].clone());assert!(!valid(&bad));
    for color in [0,0x00563cab,0x80563cab,u32::MAX] {
        let mut r=r.clone();r.layout_borders[0].color=color;assert!(valid(&r));
    }
}
#[test]
fn border_transport_roundtrips_and_rejects_malformed_colors_and_fields() {
    let r=requirement();let value=serde_json::to_value(&r).unwrap();
    assert_eq!(serde_json::from_value::<RuntimeRequirements>(value.clone()).unwrap(),r);
    for color in [serde_json::json!(-1),serde_json::json!(4294967296u64),serde_json::json!(1.5),serde_json::json!("red"),serde_json::Value::Null] {
        let mut bad=value.clone();bad["layout_borders"][0]["color"]=color;
        assert!(serde_json::from_value::<RuntimeRequirements>(bad).is_err());
    }
    let mut bad=value.clone();bad["layout_borders"][0]["extra"]=true.into();
    assert!(serde_json::from_value::<RuntimeRequirements>(bad).is_err());
    let mut bad=value;bad["layout_borders"][0].as_object_mut().unwrap().remove("color");
    assert!(serde_json::from_value::<RuntimeRequirements>(bad).is_err());
    assert!(serde_json::to_value(RuntimeRequirements::default()).unwrap().get("layout_borders").is_none());
}
#[test]
fn border_and_clip_margin_requirements_compose_without_loosening_legacy_versions() {
    let mut r=requirement();
    r.layout_overflow_clip_margins.push(LayoutOverflowClipMarginRequirement {
        object_id:1,origin:OverflowClipBox::PaddingBox,pixels:8.
    });
    r.capabilities.insert(RuntimeCapability::LayoutCssOverflowClipMarginV1);
    assert!(valid(&r));
    r.version=21;assert!(!valid(&r));
    r.layout_borders.clear();r.capabilities.remove(&RuntimeCapability::LayoutCssSolidBordersV1);
    assert!(valid(&r));
    r.version=22;assert!(!valid(&r));
}

#[test]
fn side_contract_rejects_wrong_versions_capabilities_and_overlapping_targets() {
    use nuxie_html_to_riv::LayoutBorderSidesRequirement;
    let r = RuntimeRequirements { version:23,
        capabilities:[RuntimeCapability::LayoutCssBorderSidesV1].into(),
        layout_border_sides:vec![LayoutBorderSidesRequirement { object_id:1, colors:[0,1,0x80123456,u32::MAX] }],
        ..Default::default() };
    assert!(valid(&r));
    assert!(r.ensure_supported(&[]).is_err());
    assert!(r.ensure_layout_targets(|id| id==1).is_ok());
    assert!(r.ensure_layout_targets(|_|false).is_err());
    for version in [0,1,21,22,24] { let mut bad=r.clone();bad.version=version;assert!(!valid(&bad)); }
    let mut bad=r.clone();bad.capabilities.clear();assert!(!valid(&bad));
    bad=r.clone();bad.layout_border_sides.clear();assert!(!valid(&bad));
    bad=r.clone();bad.layout_border_sides[0].object_id=0;assert!(!valid(&bad));
    bad=r.clone();bad.layout_border_sides.push(bad.layout_border_sides[0].clone());assert!(!valid(&bad));
    let mut combined=r.clone();combined.layout_borders=requirement().layout_borders;
    combined.capabilities.insert(RuntimeCapability::LayoutCssSolidBordersV1);
    assert!(!valid(&combined));
    combined.layout_borders[0].object_id=2;assert!(valid(&combined));
    let value=serde_json::to_value(&r).unwrap();
    assert_eq!(serde_json::from_value::<RuntimeRequirements>(value.clone()).unwrap(),r);
    for colors in [serde_json::json!([]),serde_json::json!([1,2,3]),serde_json::json!([1,2,3,4,5]),
        serde_json::json!([-1,2,3,4]),serde_json::json!([1.5,2,3,4]),serde_json::json!([null,2,3,4]),
        serde_json::json!([4294967296u64,2,3,4])] {
        let mut bad=value.clone();bad["layout_border_sides"][0]["colors"]=colors;
        assert!(serde_json::from_value::<RuntimeRequirements>(bad).is_err());
    }
    let mut bad=value;bad["layout_border_sides"][0]["extra"]=true.into();
    assert!(serde_json::from_value::<RuntimeRequirements>(bad).is_err());
}

#[test]
fn public_side_compilation_preserves_physical_color_order_and_uniform_version() {
    use nuxie_html_to_riv::{compile,CompileInput};
    let input=|css:&str|CompileInput { html:"<div id='box'></div>".into(),css:css.into(),width:390.,height:320.,..Default::default() };
    let output=compile(&input("#box {width:60%; height:140px; border-style:solid; border-width:2px 8px 14px 20px; border-color:red green blue transparent;}")).unwrap();
    let r=&output.runtime_requirements;
    assert_eq!(r.version,23);assert!(valid(r));assert!(r.layout_borders.is_empty());
    assert_eq!(r.layout_border_sides.len(),1);
    assert_eq!(r.layout_border_sides[0].colors,[0xffff0000,0xff008000,0xff0000ff,0]);
    assert_eq!(r.layout_border_sides[0].object_id,output.source_map.iter().find(|n|n.id=="box").unwrap().object_id);
    let uniform=compile(&input("#box {border:8px solid red;}")).unwrap();
    let expanded=compile(&input("#box {border-style:solid; border-width:8px 8px 8px 8px; border-color:red red red red;}")).unwrap();
    assert_eq!(uniform.runtime_requirements.version,22);
    assert_eq!(uniform.riv,expanded.riv);
    assert_eq!(uniform.runtime_requirements,expanded.runtime_requirements);
    assert_eq!(serde_json::to_value(&uniform.source_map).unwrap(),serde_json::to_value(&expanded.source_map).unwrap());
    let invisible=compile(&input("#box {border-top:8px none red; border-left:20px hidden green;}")).unwrap();
    assert!(invisible.runtime_requirements.layout_borders.is_empty());
    assert!(invisible.runtime_requirements.layout_border_sides.is_empty());
}
