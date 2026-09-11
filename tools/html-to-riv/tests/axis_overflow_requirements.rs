use nuxie_html_to_riv::{LayoutAxisOverflowRequirement, LayoutStackingRequirement, OverflowAxis, RuntimeCapability, RuntimeRequirements};

fn requirements() -> RuntimeRequirements {
    RuntimeRequirements { version:20, capabilities:[RuntimeCapability::LayoutCssAxisOverflowV1].into(),
        layout_axis_overflow:vec![LayoutAxisOverflowRequirement {object_id:1,axis:OverflowAxis::X}], ..Default::default() }
}
#[test]
fn axis_contract_validates_version_capability_and_target_identity() {
    let valid=requirements();
    valid.ensure_supported(&[RuntimeCapability::LayoutCssAxisOverflowV1]).unwrap();
    valid.ensure_layout_targets(|id|id==1).unwrap();
    assert!(valid.ensure_supported(&[]).is_err());
    assert!(valid.ensure_layout_targets(|_|false).is_err());
    let mut invalid=valid.clone();invalid.version=19;assert!(invalid.ensure_supported(&[RuntimeCapability::LayoutCssAxisOverflowV1]).is_err());
    invalid=valid.clone();invalid.capabilities.clear();assert!(invalid.ensure_supported(&[]).is_err());
    invalid=valid.clone();invalid.layout_axis_overflow.clear();assert!(invalid.ensure_supported(&[RuntimeCapability::LayoutCssAxisOverflowV1]).is_err());
    invalid=valid.clone();invalid.layout_axis_overflow[0].object_id=0;assert!(invalid.ensure_supported(&[RuntimeCapability::LayoutCssAxisOverflowV1]).is_err());
    invalid=valid.clone();invalid.layout_axis_overflow.push(valid.layout_axis_overflow[0].clone());assert!(invalid.ensure_supported(&[RuntimeCapability::LayoutCssAxisOverflowV1]).is_err());
    let mut combined=valid;combined.layout_stacking.push(LayoutStackingRequirement {object_id:1,level:0});combined.capabilities.insert(RuntimeCapability::LayoutCssStackingV1);
    combined.ensure_supported(&combined.capabilities.iter().copied().collect::<Vec<_>>()).unwrap();
}
#[test]
fn axis_contract_serializes_strict_axis_values_and_preserves_older_empty_shape() {
    let mut valid=requirements();valid.layout_axis_overflow.push(LayoutAxisOverflowRequirement {object_id:2,axis:OverflowAxis::Y});
    let value=serde_json::to_value(&valid).unwrap();assert_eq!(value["layout_axis_overflow"][0]["axis"],"x");assert_eq!(value["layout_axis_overflow"][1]["axis"],"y");
    assert_eq!(serde_json::from_value::<RuntimeRequirements>(value.clone()).unwrap(),valid);
    for axis in [serde_json::json!("both"),serde_json::json!("X"),serde_json::json!(0),serde_json::Value::Null] {
        let mut invalid=value.clone();invalid["layout_axis_overflow"][0]["axis"]=axis;assert!(serde_json::from_value::<RuntimeRequirements>(invalid).is_err());
    }
    assert!(serde_json::to_value(RuntimeRequirements::default()).unwrap().get("layout_axis_overflow").is_none());
}
