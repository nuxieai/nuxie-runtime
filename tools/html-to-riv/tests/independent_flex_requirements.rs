use nuxie_html_to_riv::{LayoutFlexFactorsRequirement, RuntimeCapability, RuntimeRequirements};

fn valid() -> RuntimeRequirements {
    RuntimeRequirements {
        version: 9,
        capabilities: [RuntimeCapability::LayoutCssFlexFactorsV1]
            .into_iter()
            .collect(),
        layout_flex_factors: vec![LayoutFlexFactorsRequirement {
            object_id: 2,
            grow: 0.,
            shrink: 1.,
        }],
        ..Default::default()
    }
}
#[test]
fn independent_factor_contract_requires_valid_payload_and_host() {
    let good = valid();
    good.ensure_supported(&[RuntimeCapability::LayoutCssFlexFactorsV1])
        .unwrap();
    good.ensure_layout_targets(|id| id == 2).unwrap();
    assert!(good.ensure_supported(&[]).is_err());
    assert!(good.ensure_layout_targets(|_| false).is_err());
    for version in 1..=8 {
        let mut bad = valid();
        bad.version = version;
        assert!(
            bad.ensure_supported(&[RuntimeCapability::LayoutCssFlexFactorsV1])
                .is_err()
        );
    }
    for mode in 0..5 {
        let mut bad = valid();
        match mode {
            0 => bad.layout_flex_factors.clear(),
            1 => bad.capabilities.clear(),
            2 => bad
                .layout_flex_factors
                .push(bad.layout_flex_factors[0].clone()),
            3 => bad.layout_flex_factors[0].grow = f32::NAN,
            _ => bad.layout_flex_factors[0].shrink = -1.,
        }
        assert!(
            bad.ensure_supported(&[RuntimeCapability::LayoutCssFlexFactorsV1])
                .is_err()
        );
    }
    let bytes = serde_json::to_vec(&good).unwrap();
    assert_eq!(
        serde_json::from_slice::<RuntimeRequirements>(&bytes).unwrap(),
        good
    );
    for invalid in [-1., f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        for grow in [true, false] {
            let mut bad = valid();
            if grow { bad.layout_flex_factors[0].grow = invalid; }
            else { bad.layout_flex_factors[0].shrink = invalid; }
            assert!(bad.ensure_supported(&[RuntimeCapability::LayoutCssFlexFactorsV1]).is_err());
        }
    }
    let mut unexpected = serde_json::to_value(&good).unwrap();
    unexpected["layout_flex_factors"][0]["unknown"] = serde_json::json!(1);
    assert!(serde_json::from_value::<RuntimeRequirements>(unexpected).is_err());
}
