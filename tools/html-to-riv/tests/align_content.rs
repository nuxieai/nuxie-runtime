use nuxie_html_to_riv::{
    AlignContent, CompileInput, RuntimeCapability, RuntimeRequirements, compile,
};

#[test]
fn line_alignment_cascade_resets_and_substitution_publish_the_correct_contract() {
    let compile_css = |css: &str| {
        compile(&CompileInput {
            html: "<div id=root><div id=a></div></div>".into(),
            css: css.into(),
            width: 390.,
            height: 320.,
            ..Default::default()
        })
    };
    for value in [
        "flex-start",
        "center",
        "flex-end",
        "stretch",
        "space-between",
        "space-around",
    ] {
        let direct = compile_css(&format!("#a{{align-content:{value}}}")).unwrap();
        assert_eq!(
            direct.runtime_requirements,
            compile_css(&format!("#a{{--v:{value};align-content:var(--v)}}"))
                .unwrap()
                .runtime_requirements
        );
    }
    let inherited = compile_css("#root{align-content:center}#a{align-content:inherit}").unwrap();
    assert_eq!(inherited.runtime_requirements.layout_align_content.len(), 2);
    assert!(
        inherited
            .runtime_requirements
            .layout_align_content
            .iter()
            .all(|entry| entry.alignment == AlignContent::Center)
    );
    assert_eq!(
        compile_css("#root{align-content:center}")
            .unwrap()
            .runtime_requirements
            .layout_align_content
            .len(),
        1,
        "not implicitly inherited"
    );
    for reset in ["initial", "unset", "var(--missing)", "var(--bad)"] {
        let output = compile_css(&format!(
            "#a{{--bad:7px;align-content:center;align-content:{reset}}}"
        ))
        .unwrap();
        assert_eq!(
            output.runtime_requirements.layout_align_content[0].alignment,
            AlignContent::Stretch,
            "{reset}"
        );
    }
    for invalid in [
        "normal",
        "start",
        "end",
        "safe center",
        "last baseline",
        "calc(1)",
        "var(--unsupported)",
    ] {
        assert!(
            compile_css(&format!(
                "#a{{--unsupported:safe center;align-content:{invalid}}}"
            ))
            .is_err(),
            "{invalid} must not be silently reset"
        );
    }
    let painted = compile_css("#root{align-content:center}#a{background:red}").unwrap();
    assert_eq!(
        painted.runtime_requirements.version, 7,
        "later paint requirements must not downgrade version"
    );
    assert!(!painted.runtime_requirements.layout_pixel_bounds.is_empty());
}

#[test]
fn line_alignment_manifest_versions_capabilities_and_targets_are_checked() {
    let valid = serde_json::json!({"version":7,"capabilities":["layout-css-align-content-v1"],"layout_align_content":[{"object_id":7,"alignment":"center"}]});
    let requirements: RuntimeRequirements = serde_json::from_value(valid.clone()).unwrap();
    assert_eq!(
        requirements.ensure_supported(&[]).unwrap_err().code,
        "missing-runtime-capability"
    );
    requirements
        .ensure_supported(&[RuntimeCapability::LayoutCssAlignContentV1])
        .unwrap();
    requirements.ensure_layout_targets(|id| id == 7).unwrap();
    assert_eq!(
        requirements
            .ensure_layout_targets(|_| false)
            .unwrap_err()
            .code,
        "invalid-layout-align-content-target"
    );
    for version in 1..=6 {
        let mut value = valid.clone();
        value["version"] = version.into();
        assert_eq!(
            serde_json::from_value::<RuntimeRequirements>(value)
                .unwrap()
                .ensure_supported(&[RuntimeCapability::LayoutCssAlignContentV1])
                .unwrap_err()
                .code,
            "invalid-layout-align-content"
        );
    }
    for (field, value) in [
        ("capabilities", serde_json::json!([])),
        ("layout_align_content", serde_json::json!([])),
        (
            "layout_align_content",
            serde_json::json!([{"object_id":7,"alignment":"center"},{"object_id":7,"alignment":"stretch"}]),
        ),
    ] {
        let mut bad = valid.clone();
        bad[field] = value;
        assert_eq!(
            serde_json::from_value::<RuntimeRequirements>(bad)
                .unwrap()
                .ensure_supported(&[RuntimeCapability::LayoutCssAlignContentV1])
                .unwrap_err()
                .code,
            "invalid-layout-align-content"
        );
    }
    let mut invalid = valid.clone();
    invalid["layout_align_content"][0]["alignment"] = "auto".into();
    assert!(serde_json::from_value::<RuntimeRequirements>(invalid).is_err());
    let mut unknown = valid;
    unknown["layout_align_content"][0]["unknown"] = true.into();
    assert!(serde_json::from_value::<RuntimeRequirements>(unknown).is_err());
}

#[test]
fn wrapped_reset_is_explicit_and_combined_requirements_keep_version_seven() {
    let run = |css: &str| compile(&CompileInput {
        html: "<div id=root><div id=a></div></div>".into(), css: css.into(),
        width:390., height:320., ..Default::default()
    }).unwrap();
    let reset = run("#root{flex-wrap:wrap;align-items:center;justify-content:space-between}");
    assert_eq!(reset.runtime_requirements.version,7);
    assert_eq!(reset.runtime_requirements.layout_align_content.len(),1);
    assert_eq!(reset.runtime_requirements.layout_align_content[0].alignment,AlignContent::FlexStart);
    let combined = run("#root{align-content:center}#a{align-self:center;background:red}");
    assert_eq!(combined.runtime_requirements.version,7);
    assert_eq!(combined.runtime_requirements.layout_align_self.len(),1);
    assert!(!combined.runtime_requirements.layout_pixel_bounds.is_empty());
    combined.runtime_requirements.ensure_supported(&[
        RuntimeCapability::LayoutCssAlignContentV1, RuntimeCapability::LayoutCssAlignSelfV1,
        RuntimeCapability::LayoutCssPixelBoundsV1,
    ]).unwrap();
}
