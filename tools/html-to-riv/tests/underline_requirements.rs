use nuxie_html_to_riv::{RuntimeCapability, RuntimeRequirements};
use serde_json::{Value, json};

fn valid() -> Value {
    json!({"version":3,"capabilities":["text-solid-underlines-v1"],
    "text_underlines":[{"object_id":6,"lines":[
        {"color":4294901760u32,"thickness":2.0,"offset":1.0,"skip_ink":"auto"},
        {"color":4278190335u32,"thickness":1.0,"offset":-2.0,"skip_ink":"none"}
    ]}]})
}
#[test]
fn underline_transport_roundtrips_and_requires_capability_and_text_targets() {
    let value = valid();
    let requirements: RuntimeRequirements = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(&requirements).unwrap(), value);
    assert_eq!(
        requirements.ensure_supported(&[]).unwrap_err().code,
        "missing-runtime-capability"
    );
    requirements
        .ensure_supported(&[RuntimeCapability::TextSolidUnderlinesV1])
        .unwrap();
    requirements.ensure_text_targets(|id| id == 6).unwrap();
    assert_eq!(
        requirements
            .ensure_text_targets(|_| false)
            .unwrap_err()
            .code,
        "invalid-text-underline-target"
    );
    for scalar in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut bad = requirements.clone();
        bad.text_underlines[0].lines[0].thickness = scalar;
        assert!(
            bad.ensure_supported(&[RuntimeCapability::TextSolidUnderlinesV1])
                .is_err()
        );
        bad.text_underlines[0].lines[0].thickness = 2.0;
        bad.text_underlines[0].lines[0].offset = scalar;
        assert!(
            bad.ensure_supported(&[RuntimeCapability::TextSolidUnderlinesV1])
                .is_err()
        );
    }
}
#[test]
fn underline_transport_rejects_malformed_records_before_target_installation() {
    let mut variants = Vec::new();
    for (field, values) in [
        (
            "thickness",
            vec![json!(0), json!(-1), json!(1_000_001), Value::Null],
        ),
        (
            "offset",
            vec![json!(1_000_001), json!(-1_000_001), Value::Null],
        ),
        ("color", vec![json!(-1), json!(4294967296u64)]),
        ("skip_ink", vec![json!("future"), Value::Null]),
    ] {
        for value in values {
            let mut bad = valid();
            bad["text_underlines"][0]["lines"][0][field] = value;
            variants.push(bad);
        }
    }
    for version in [1, 2, 4] {
        let mut bad = valid();
        bad["version"] = json!(version);
        variants.push(bad);
    }
    for field in ["text_underlines", "capabilities"] {
        let mut bad = valid();
        bad[field] = json!([]);
        variants.push(bad);
    }
    let mut bad = valid();
    bad["text_underlines"][0]["lines"] = json!([]);
    variants.push(bad);
    let mut bad = valid();
    bad["text_underlines"][0]["lines"][0]["extra"] = json!(true);
    variants.push(bad);
    let mut bad = valid();
    bad["text_underlines"][0]["extra"] = json!(true);
    variants.push(bad);
    let mut bad = valid();
    bad["text_underlines"] = json!([bad["text_underlines"][0], bad["text_underlines"][0]]);
    variants.push(bad);
    for bad in variants {
        if let Ok(requirements) = serde_json::from_value::<RuntimeRequirements>(bad.clone()) {
            assert!(
                requirements
                    .ensure_text_targets(|_| panic!("must reject before installation"))
                    .is_err(),
                "{bad}"
            );
        }
    }
}
#[test]
fn underline_transport_composes_with_existing_occurrence_policies() {
    let mut value = valid();
    value["capabilities"] = json!([
        "text-solid-underlines-v1",
        "text-css-pre-wrap-v1",
        "text-css-tabs-v1",
        "text-css-wrapped-tabs-v1"
    ]);
    value["text_policies"] = json!([{"object_id":6,"policy":"css-pre-wrap-v1"}]);
    let capabilities = [
        RuntimeCapability::TextSolidUnderlinesV1,
        RuntimeCapability::TextCssPreWrapV1,
        RuntimeCapability::TextCssTabsV1,
        RuntimeCapability::TextCssWrappedTabsV1,
    ];
    let requirements: RuntimeRequirements = serde_json::from_value(value.clone()).unwrap();
    requirements.ensure_supported(&capabilities).unwrap();
    requirements.ensure_text_targets(|id| id == 6).unwrap();
    value["text_policies"] = json!([]);
    let requirements: RuntimeRequirements = serde_json::from_value(value).unwrap();
    assert_eq!(
        requirements
            .ensure_supported(&capabilities)
            .unwrap_err()
            .code,
        "invalid-text-policies"
    );
}
