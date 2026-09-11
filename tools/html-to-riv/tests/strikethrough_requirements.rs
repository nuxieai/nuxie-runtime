use nuxie_html_to_riv::{RuntimeCapability, RuntimeRequirements};
use serde_json::{Value, json};

fn valid() -> Value {
    json!({"version":4,"capabilities":["text-solid-strikethroughs-v1"],
    "text_strikethroughs":[{"object_id":6,"lines":[
        {"color":4294901760u32,"thickness":2.0,"offset":1.0,"line_baseline":22.0},
        {"color":4278190335u32,"thickness":1.0,"offset":-2.0,"line_baseline":22.0}
    ]}]})
}
#[test]
fn strikethrough_transport_roundtrips_and_requires_capability_and_text_targets() {
    let value = valid();
    let requirements: RuntimeRequirements = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(&requirements).unwrap(), value);
    assert_eq!(
        requirements.ensure_supported(&[]).unwrap_err().code,
        "missing-runtime-capability"
    );
    requirements
        .ensure_supported(&[RuntimeCapability::TextSolidStrikethroughsV1])
        .unwrap();
    requirements.ensure_text_targets(|id| id == 6).unwrap();
    assert_eq!(
        requirements
            .ensure_text_targets(|_| false)
            .unwrap_err()
            .code,
        "invalid-text-strikethrough-target"
    );
    for scalar in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut bad = requirements.clone();
        bad.text_strikethroughs[0].lines[0].thickness = scalar;
        assert!(
            bad.ensure_supported(&[RuntimeCapability::TextSolidStrikethroughsV1])
                .is_err()
        );
        let mut bad_baseline = requirements.clone();
        bad_baseline.text_strikethroughs[0].lines[0].line_baseline = scalar;
        assert!(
            bad_baseline
                .ensure_supported(&[RuntimeCapability::TextSolidStrikethroughsV1])
                .is_err()
        );
        bad.text_strikethroughs[0].lines[0].thickness = 2.0;
        bad.text_strikethroughs[0].lines[0].offset = scalar;
        assert!(
            bad.ensure_supported(&[RuntimeCapability::TextSolidStrikethroughsV1])
                .is_err()
        );
    }
}
#[test]
fn strikethrough_transport_rejects_malformed_records_before_target_installation() {
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
        (
            "line_baseline",
            vec![json!(1_000_001), json!(-1_000_001), Value::Null],
        ),
        ("skip_ink", vec![json!("auto"), Value::Null]),
    ] {
        for value in values {
            let mut bad = valid();
            bad["text_strikethroughs"][0]["lines"][0][field] = value;
            variants.push(bad);
        }
    }
    for version in [1, 2, 3, 5] {
        let mut bad = valid();
        bad["version"] = json!(version);
        variants.push(bad);
    }
    for field in ["text_strikethroughs", "capabilities"] {
        let mut bad = valid();
        bad[field] = json!([]);
        variants.push(bad);
    }
    let mut bad = valid();
    bad["text_strikethroughs"][0]["lines"] = json!([]);
    variants.push(bad);
    let mut bad = valid();
    bad["text_strikethroughs"][0]["lines"][0]["extra"] = json!(true);
    variants.push(bad);
    let mut bad = valid();
    bad["text_strikethroughs"][0]["extra"] = json!(true);
    variants.push(bad);
    let mut bad = valid();
    bad["text_strikethroughs"] =
        json!([bad["text_strikethroughs"][0], bad["text_strikethroughs"][0]]);
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
fn strikethrough_transport_composes_with_existing_occurrence_policies() {
    let mut value = valid();
    value["capabilities"] = json!([
        "text-solid-strikethroughs-v1",
        "text-css-pre-wrap-v1",
        "text-css-tabs-v1",
        "text-css-wrapped-tabs-v1"
    ]);
    value["text_policies"] = json!([{"object_id":6,"policy":"css-pre-wrap-v1"}]);
    let capabilities = [
        RuntimeCapability::TextSolidStrikethroughsV1,
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

#[test]
fn strikethrough_and_underline_share_targets_but_keep_separate_capabilities() {
    let mut value = valid();
    value["capabilities"] = json!(["text-solid-underlines-v1", "text-solid-strikethroughs-v1"]);
    value["text_underlines"] = json!([{"object_id":6,"lines":[{"color":4294901760u32,"thickness":2.0,"offset":1.0,"skip_ink":"auto"}]}]);
    let requirements: RuntimeRequirements = serde_json::from_value(value).unwrap();
    requirements
        .ensure_supported(&[
            RuntimeCapability::TextSolidUnderlinesV1,
            RuntimeCapability::TextSolidStrikethroughsV1,
        ])
        .unwrap();
    requirements.ensure_text_targets(|id| id == 6).unwrap();
    for capability in [
        RuntimeCapability::TextSolidUnderlinesV1,
        RuntimeCapability::TextSolidStrikethroughsV1,
    ] {
        assert_eq!(
            requirements
                .ensure_supported(&[capability])
                .unwrap_err()
                .code,
            "missing-runtime-capability"
        );
    }
}
