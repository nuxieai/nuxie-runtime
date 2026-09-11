use nuxie_html_to_riv::{LayoutOverflowClipMarginRequirement, OverflowClipBox, LayoutAxisOverflowRequirement, OverflowAxis, RuntimeCapability, RuntimeRequirements};
fn requirements() -> RuntimeRequirements {
    RuntimeRequirements {version:21,capabilities:[RuntimeCapability::LayoutCssOverflowClipMarginV1].into(),
        layout_overflow_clip_margins:vec![LayoutOverflowClipMarginRequirement{object_id:1,origin:OverflowClipBox::ContentBox,pixels:8.}],..Default::default()}
}
fn valid(req: &RuntimeRequirements) -> bool {
    req.ensure_supported(&req.capabilities.iter().copied().collect::<Vec<_>>()).is_ok()
}
#[test]
fn clip_margin_contract_checks_version_offsets_targets_and_axis_conflicts() {
    let req=requirements();assert!(valid(&req));
    assert!(req.ensure_supported(&[]).is_err());
    assert!(req.ensure_layout_targets(|id| id==1).is_ok());
    assert!(req.ensure_layout_targets(|_| false).is_err());
    for version in [0,1,19,20,22] {let mut r=req.clone();r.version=version;assert!(!valid(&r));}
    for pixels in [f32::NAN,f32::INFINITY,f32::NEG_INFINITY] {
        let mut r=req.clone();r.layout_overflow_clip_margins[0].pixels=pixels;assert!(!valid(&r));
    }
    let mut r=req.clone();r.layout_overflow_clip_margins.clear();assert!(!valid(&r));
    r=req.clone();r.capabilities.clear();assert!(!valid(&r));
    r=req.clone();r.layout_overflow_clip_margins[0].object_id=0;assert!(!valid(&r));
    r=req.clone();r.layout_overflow_clip_margins.push(r.layout_overflow_clip_margins[0].clone());assert!(!valid(&r));
    r=req.clone();r.capabilities.insert(RuntimeCapability::LayoutCssAxisOverflowV1);
    r.layout_axis_overflow.push(LayoutAxisOverflowRequirement{object_id:1,axis:OverflowAxis::X});assert!(!valid(&r));
    r.layout_axis_overflow[0].object_id=2;assert!(valid(&r));
    r.version=20;assert!(!valid(&r));
}
#[test]
fn clip_margin_transport_roundtrips_and_rejects_unknown_shape() {
    let req=requirements();let value=serde_json::to_value(&req).unwrap();
    assert_eq!(value["layout_overflow_clip_margins"][0]["origin"],"content-box");
    assert_eq!(serde_json::from_value::<RuntimeRequirements>(value.clone()).unwrap(),req);
    for origin in [serde_json::json!("content"),serde_json::json!("CONTENT-BOX"),serde_json::json!(0),serde_json::Value::Null] {
        let mut r=value.clone();r["layout_overflow_clip_margins"][0]["origin"]=origin;
        assert!(serde_json::from_value::<RuntimeRequirements>(r).is_err());
    }
    let mut r=value.clone();r["layout_overflow_clip_margins"][0]["extra"]=true.into();
    assert!(serde_json::from_value::<RuntimeRequirements>(r).is_err());
    assert!(serde_json::to_value(RuntimeRequirements::default()).unwrap().get("layout_overflow_clip_margins").is_none());
}
