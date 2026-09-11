use nuxie_html_to_riv::{Asset, CompileInput, CompileOutput, compile};
fn scene(css: &str) -> CompileOutput {
    compile(&CompileInput {
        html: "<div id=root><p id=a>agypqj M</p><p id=b>child</p></div>".into(),
        css: css.into(),
        width: 390.0,
        height: 320.0,
        assets: [(
            "font".into(),
            Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        )]
        .into(),
    })
    .unwrap()
}
#[test]
fn strikethrough_longhand_and_combined_shorthand_emit_distinct_layers() {
    let plain = scene("p{font-size:20px;line-height:30px;text-decoration:line-through red 2px}");
    assert_eq!(plain.runtime_requirements.version, 11);
    assert!(plain.runtime_requirements.text_underlines.is_empty());
    for entry in &plain.runtime_requirements.text_strikethroughs {
        let line = &entry.lines[0];
        assert_eq!(line.color, 0xffff0000);
        assert_eq!(line.thickness, 2.0);
        assert_eq!(line.line_baseline, 22.0);
        assert_eq!(line.offset, -19.0 / 3.0 - 1.0);
    }
    for value in [
        "underline line-through red 2px",
        "line-through 2px red underline",
        "RED UNDERLINE SOLID LINE-THROUGH 2PX",
    ] {
        let output = scene(&format!(
            "p{{font-size:20px;line-height:30px;text-decoration:{value}}}"
        ));
        assert_eq!(
            output.runtime_requirements.text_strikethroughs,
            plain.runtime_requirements.text_strikethroughs
        );
        assert_eq!(output.runtime_requirements.text_underlines.len(), 2);
        assert_eq!(output.runtime_requirements.version, 11);
    }
}
#[test]
fn propagated_none_preserves_ancestor_and_new_origin_adds_another_strike() {
    let output = scene(
        "#root{color:red;text-decoration:line-through 2px}#a{color:blue;text-decoration:none}#b{color:green;text-decoration:line-through}",
    );
    let entries = &output.runtime_requirements.text_strikethroughs;
    assert_eq!(entries[0].lines.len(), 1);
    assert_eq!(entries[0].lines[0].color, 0xffff0000);
    assert_eq!(entries[1].lines.len(), 2);
    assert_eq!(entries[1].lines[1].color, 0xff008000);
}
#[test]
fn strike_ignores_underline_offset_and_skip_and_retains_version_with_later_underline() {
    let base = scene("p{text-decoration:line-through 2px}");
    let variant = scene(
        "p{text-decoration:line-through 2px;text-underline-offset:100%;text-decoration-skip-ink:all}",
    );
    assert_eq!(base.runtime_requirements, variant.runtime_requirements);
    let mixed =
        scene("#a{text-decoration:line-through}#b{text-decoration:underline;white-space:pre-wrap}");
    assert_eq!(mixed.runtime_requirements.version, 11);
    mixed
        .runtime_requirements
        .ensure_supported(
            &mixed
                .runtime_requirements
                .capabilities
                .iter()
                .copied()
                .collect::<Vec<_>>(),
        )
        .unwrap();
}
#[test]
fn duplicate_and_unsupported_decoration_components_are_rejected() {
    for value in [
        "none line-through",
        "line-through line-through",
        "line-through underline underline",
        "overline",
        "line-through dashed",
        "line-through 2px 3px",
    ] {
        assert!(
            compile(&CompileInput {
                html: "<p></p>".into(),
                css: format!("p{{text-decoration:{value}}}"),
                width: 390.0,
                height: 320.0,
                ..Default::default()
            })
            .is_err(),
            "{value}"
        );
    }
}
#[test]
fn explicit_inheritance_copies_both_line_flags_and_initial_resets_them() {
    let output = scene(
        "#root{text-decoration:underline line-through}#a{text-decoration-line:inherit}#b{text-decoration-line:initial}",
    );
    assert_eq!(
        output.runtime_requirements.text_strikethroughs[0]
            .lines
            .len(),
        2
    );
    assert_eq!(
        output.runtime_requirements.text_underlines[0].lines.len(),
        2
    );
    assert_eq!(
        output.runtime_requirements.text_strikethroughs[1]
            .lines
            .len(),
        1
    );
}

#[test]
fn accepted_metric_extremes_emit_host_valid_requirements() {
    // Public compiler output must satisfy the same contract the host validates.
    // These are transport-boundary checks, not claims of pixel qualification.
    for size in [0.1, 1.0, 8.0, 128.0, 800_000.0] {
        for thickness in ["auto", "from-font", "0px", "1000000px"] {
            let output = scene(&format!(
                "p{{font-size:{size}px;line-height:1000000px;text-decoration:line-through red {thickness}}}"
            ));
            let requirements = &output.runtime_requirements;
            requirements
                .ensure_supported(&requirements.capabilities.iter().copied().collect::<Vec<_>>())
                .unwrap();
            let encoded = serde_json::to_vec(requirements).unwrap();
            let decoded: nuxie_html_to_riv::RuntimeRequirements =
                serde_json::from_slice(&encoded).unwrap();
            assert_eq!(&decoded, requirements);
            assert_eq!(requirements.text_strikethroughs.len(), 2);
        }
    }
}
