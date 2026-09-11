use nuxie_html_to_riv::RuntimeRequirements;
use serde_json::{json, Value};
fn valid() -> Value { json!({"version":5,"capabilities":["layout-css-pixel-bounds-v1"],"layout_pixel_bounds":[2,7]}) }
#[test]
fn layout_paint_requirements_are_portable_and_capability_checked() {
    let value=valid();
    let r:RuntimeRequirements=serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(&r).unwrap(),value);
    assert_eq!(r.ensure_supported(&[]).unwrap_err().code,"missing-runtime-capability");
    r.ensure_supported(&r.capabilities.iter().copied().collect::<Vec<_>>()).unwrap();
}
#[test]
fn malformed_layout_paint_requirements_are_rejected() {
    let mut variants=Vec::new();
    for version in [1,2,3,4,6] {let mut v=valid();v["version"]=json!(version);variants.push(v);}
    for ids in [json!([]),json!([2,2]),json!([-1]),json!([4294967296u64]),json!(["2"]),Value::Null] {let mut v=valid();v["layout_pixel_bounds"]=ids;variants.push(v);}
    let mut v=valid();v["capabilities"]=json!([]);variants.push(v);
    for v in variants {
        if let Ok(r)=serde_json::from_value::<RuntimeRequirements>(v.clone()) {
            assert!(r.ensure_supported(&r.capabilities.iter().copied().collect::<Vec<_>>()).is_err(),"{v}");
        }
    }
}
#[test]
fn layout_targets_and_text_records_are_checked_independently() {
    let mut value=valid();
    value["capabilities"]=json!(["layout-css-pixel-bounds-v1","text-solid-strikethroughs-v1"]);
    value["text_strikethroughs"]=json!([{"object_id":9,"lines":[{"color":4278190080u32,"thickness":1.0,"offset":0.0,"line_baseline":12.0}]}]);
    let r:RuntimeRequirements=serde_json::from_value(value).unwrap();
    r.ensure_supported(&r.capabilities.iter().copied().collect::<Vec<_>>()).unwrap();
    r.ensure_layout_targets(|id|[2,7].contains(&id)).unwrap();
    r.ensure_text_targets(|id|id==9).unwrap();
    assert_eq!(r.ensure_layout_targets(|id|id==2).unwrap_err().code,"invalid-layout-pixel-bounds-target");
    assert!(r.ensure_text_targets(|_|false).is_err());
    let old:RuntimeRequirements=serde_json::from_value(json!({"version":1,"capabilities":[]})).unwrap();
    old.ensure_supported(&[]).unwrap();old.ensure_layout_targets(|_|false).unwrap();
}
#[test]
fn compiler_declares_painted_and_clipped_layouts_only() {
    use nuxie_html_to_riv::{compile, CompileInput, RuntimeCapability};
    let out=compile(&CompileInput{html:"<div id=root><div id=paint></div><div id=clip></div><div id=empty></div></div>".into(),css:"#root{width:100%}#paint{width:20px;height:20px;background:red}#clip{width:20px;height:20px;overflow:hidden}".into(),width:390.,height:320.,..Default::default()}).unwrap();
    assert_eq!(out.runtime_requirements.version,5);
    assert!(out.runtime_requirements.capabilities.contains(&RuntimeCapability::LayoutCssPixelBoundsV1));
    let expected:Vec<_>=out.source_map.iter().filter(|s|s.id=="paint"||s.id=="clip").map(|s|s.object_id).collect();
    assert_eq!(out.runtime_requirements.layout_pixel_bounds,expected);
}
