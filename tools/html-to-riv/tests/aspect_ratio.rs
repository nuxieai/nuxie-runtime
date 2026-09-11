use nuxie_html_to_riv::{compile, CompileInput, RuntimeCapability};

fn input(css: &str) -> CompileInput {
    CompileInput { html: "<div id=root><div id=a></div></div>".into(),
        css: css.into(), width:390., height:320., ..Default::default() }
}

#[test]
fn computed_font_size_clamps_before_font_relative_dimensions_and_ratios() {
    for size in ["10001px", "16384px", "1000000px"] {
        let actual = compile(&input(&format!("#root{{font-size:{size}}}#a{{font-size:.5em;width:1em;aspect-ratio:calc(16px / 1em)}}"))).unwrap();
        let expected = compile(&input("#root{font-size:10000px}#a{font-size:5000px;width:1em;aspect-ratio:calc(16px / 1em)}")).unwrap();
        assert_eq!(actual.riv, expected.riv, "{size}");
    }
}

#[test]
fn ratio_font_units_use_final_computed_font_size_and_fixed_root_size() {
    for (declarations, expected) in [
        ("aspect-ratio:calc(1em / 16px)", "1.25"),
        ("aspect-ratio:calc(1em / 16px);font-size:24px", "1.5"),
        ("font-size:12px;aspect-ratio:calc(1em / 16px);font-size:32px", "2"),
        ("font-size:1.25em;aspect-ratio:calc(1em / 1rem)", "1.5625"),
        ("font-size:32px;aspect-ratio:calc(1rem / 16px)", "1"),
        ("--ratio:calc((1em + 1rem) / 16px);aspect-ratio:var(--ratio)", "2.25"),
    ] {
        let css = format!("#root{{font-size:20px}}#a{{{declarations}}}");
        let actual = compile(&input(&css)).unwrap();
        let literal = compile(&input(&format!("{css}#a{{aspect-ratio:{expected}}}"))).unwrap();
        assert_eq!(actual.riv, literal.riv, "{declarations}");
        assert_eq!(actual.runtime_requirements, literal.runtime_requirements, "{declarations}");
    }
}

#[test]
fn inherited_ratios_retain_computed_values_but_custom_math_uses_child_font() {
    let base = "#root{font-size:20px;aspect-ratio:calc(1em / 16px);--ratio:calc(1em / 16px)}#a{font-size:40px}";
    for (value, expected) in [("inherit", "1.25"), ("var(--ratio)", "2.5"), ("var(--missing,calc(1em / 16px))", "2.5")] {
        let actual = compile(&input(&format!("{base}#a{{aspect-ratio:{value}}}"))).unwrap();
        let literal = compile(&input(&format!("{base}#a{{aspect-ratio:{expected}}}"))).unwrap();
        assert_eq!(actual.riv, literal.riv, "{value}");
    }
}

#[test]
fn css_ratio_precision_does_not_admit_percentage_gaps() {
    for property in ["gap", "row-gap", "column-gap"] {
        for value in ["5%", "var(--space)", "var(--missing,5%)"] {
            let css = format!("#root{{--space:5%;{property}:{value}}}#a{{aspect-ratio:2}}");
            assert!(compile(&input(&css)).is_err(), "{property}:{value}");
        }
    }
}

#[test]
fn aspect_ratio_cascade_preserves_auto_and_initial_semantics() {
    let base = compile(&input("#a{aspect-ratio:auto 2}")).unwrap();
    for css in ["#a{aspect-ratio:4/2 auto}", "#a{--ratio:auto 2;aspect-ratio:var(--ratio)}",
        "#a{aspect-ratio:auto 2!important;aspect-ratio:3}"] {
        let other=compile(&input(css)).unwrap();
        assert_eq!(base.riv,other.riv);
        assert_eq!(base.runtime_requirements,other.runtime_requirements);
    }
    let bare=compile(&input("#a{aspect-ratio:2}")).unwrap();
    assert_eq!(bare.riv,base.riv);
    assert!(!bare.runtime_requirements.layout_aspect_ratios[0].content_box);
    assert!(base.runtime_requirements.layout_aspect_ratios[0].content_box);
    for value in ["auto","initial","unset","0","0/1","1/0","var(--missing)"] {
        let result=compile(&input(&format!("#a{{aspect-ratio:{value}}}"))).unwrap();
        assert!(result.runtime_requirements.layout_aspect_ratios.is_empty(),"{value}");
    }
    let inherited=compile(&input("#root{aspect-ratio:2}#a{aspect-ratio:inherit}")).unwrap();
    assert_eq!(inherited.runtime_requirements.layout_aspect_ratios.len(),2);
    let non_inherited=compile(&input("#root{aspect-ratio:2}")).unwrap();
    assert_eq!(non_inherited.runtime_requirements.layout_aspect_ratios.len(),1);
}

#[test]
fn aspect_ratio_manifest_rejects_incompatible_or_malformed_contracts() {
    let result=compile(&input("#a{aspect-ratio:auto 2;box-sizing:content-box;width:50%}")).unwrap();
    let good=result.runtime_requirements;
    assert_eq!(good.version,15);
    let caps=good.capabilities.iter().copied().collect::<Vec<_>>();
    good.ensure_supported(&caps).unwrap();
    assert!(good.ensure_supported(&[]).is_err());
    for version in 1..14 {
        let mut bad=good.clone();bad.version=version;
        assert_eq!(bad.ensure_supported(&caps).unwrap_err().code,"invalid-layout-aspect-ratio");
    }
    let mut bad=good.clone();bad.layout_aspect_ratios.clear();
    assert!(bad.ensure_supported(&caps).is_err());
    let mut bad=good.clone();bad.capabilities.remove(&RuntimeCapability::LayoutCssAspectRatioV1);
    assert!(bad.ensure_supported(&caps).is_err());
    let mut bad=good.clone();bad.layout_aspect_ratios.push(bad.layout_aspect_ratios[0].clone());
    assert!(bad.ensure_supported(&caps).is_err());
    assert_eq!(good.ensure_layout_targets(|_|false).unwrap_err().code,"invalid-layout-aspect-ratio-target");
    let mut serialized=serde_json::to_value(&good).unwrap();
    serialized["layout_aspect_ratios"][0].as_object_mut().unwrap().remove("content_box");
    assert!(serde_json::from_value::<nuxie_html_to_riv::RuntimeRequirements>(serialized).is_err());
}

#[test]
fn image_authored_ratio_admission_preserves_auto_dimensions() {
    let image = |css: &str| CompileInput {
        html: "<img id=photo src=\"asset:photo\">".into(), css: css.into(), width:390., height:320.,
        assets: [("photo".into(), nuxie_html_to_riv::Asset::Image {
            bytes: include_bytes!("assets/quadrants.png").to_vec(),
        })].into(), ..Default::default()
    };
    for css in ["img{width:60%;aspect-ratio:3/2}", "img{height:80px;aspect-ratio:3/2}"] {
        let result = compile(&image(css)).unwrap();
        assert_eq!(result.runtime_requirements.layout_aspect_ratios.len(), 1);
        assert!(!result.runtime_requirements.layout_aspect_ratios[0].content_box);
    }
    for css in ["img{aspect-ratio:3/2}", "img{width:60%;aspect-ratio:auto 3/2}",
        "img{height:80px;aspect-ratio:0}", "img{width:60%}"] {
        let result = compile(&image(css)).unwrap();
        let requirement = &result.runtime_requirements.layout_aspect_ratios[0];
        let bare = css == "img{aspect-ratio:3/2}";
        assert_eq!(requirement.pair, Some(if bare { [3, 2] } else { [1, 1] }), "{css}");
        assert_eq!(requirement.content_box, !bare, "{css}");
    }
}

#[test]
fn ratio_math_matches_literals_through_cascade_and_substitution() {
    for (expression, literal) in [("calc(3 / 2)", "1.5"),
        ("calc(16777217 - 16777216)", "1"), ("calc(1e40 / 1e40)", "1"),
        ("calc(1e-50 / 1e-50)", "1"), ("min(3, 2)", "2"),
        ("max(1, 2)", "2"), ("clamp(1, 3, 2)", "2"),
        ("calc(-1)", "0"), ("3 / calc(-2)", "3/0"),
        ("auto calc((2 + 4) / 3)", "auto 2"),
        ("min(calc(2 * 2), 3) / max(1, 2)", "1.5")] {
        let expected = compile(&input(&format!("#a{{aspect-ratio:{literal}}}"))).unwrap();
        for value in [expression.to_string(), "var(--ratio)".into(),
            format!("var(--missing, {expression})")] {
            let css = format!("#a{{--ratio:{expression};aspect-ratio:3;aspect-ratio:{value}}}");
            let result = compile(&input(&css)).unwrap();
            assert_eq!(result.riv, expected.riv, "{css}");
            assert_eq!(result.runtime_requirements, expected.runtime_requirements, "{css}");
        }
    }
}

#[test]
fn ratio_math_rejects_invalid_and_unrepresentable_values() {
    for expression in ["calc(1px)", "calc(50%)", "calc(1+2)", "calc(1 + )",
        "calc(1/**/+/**/2)", "min()", "clamp(1, 2)", "calc(2) / -1",
        "calc(2", "calc((2)"] {
        assert!(compile(&input(&format!("#a{{aspect-ratio:{expression}}}"))).is_err(), "{expression}");
    }
}

#[test]
fn tiny_ratio_components_match_chrome_before_division() {
    let expected = compile(&input("#a{aspect-ratio:0}")).unwrap();
    for component in ["1e-7", "1e-40", "1e-100", "0.00000095367431640625",
        "0.0000009536743732496689", "calc(1e-20 * 1e-20)"] {
        for expression in [component.to_string(), format!("{component}/{component}"),
            format!("1/{component}")] {
            for value in [expression.clone(), "var(--ratio)".into(),
                format!("var(--missing, {expression})")] {
                let css=format!("#a{{--ratio:{expression};aspect-ratio:{value}}}");
                let actual=compile(&input(&css)).unwrap();
                assert_eq!(actual.riv, expected.riv, "{css}");
                assert_eq!(actual.runtime_requirements, expected.runtime_requirements, "{css}");
            }
        }
    }
    // Equal components one float step above the clamp must retain a unit ratio.
    let unit=compile(&input("#a{aspect-ratio:1}")).unwrap();
    for component in ["0.0000009536744300930877", "calc(0.0000009536744300930877)"] {
        let actual=compile(&input(&format!("#a{{aspect-ratio:{component}/{component}}}"))).unwrap();
        assert_eq!(actual.riv,unit.riv);
    }
}

#[test]
fn chrome_ratio_approximation_can_degenerate_before_layout() {
    let automatic=compile(&input("#a{aspect-ratio:0}")).unwrap();
    for expression in ["0.0000009536744300930877", "calc(0.0000009536744300930877)",
        "0.0000009536744300930877 / 1"] {
        let actual=compile(&input(&format!("#a{{aspect-ratio:{expression}}}"))).unwrap();
        assert_eq!(actual.runtime_requirements, automatic.runtime_requirements, "{expression}");
        assert_eq!(actual.riv, automatic.riv, "{expression}");
    }
    // An exactly representable component pair takes Chrome's lossless path.
    let exact=compile(&input("#a{aspect-ratio:1/1048576}")).unwrap();
    assert_eq!(exact.runtime_requirements.layout_aspect_ratios.len(),1);
}

#[test]
fn nonfinite_ratio_math_uses_chrome_layout_conversion() {
    for (expression, expected) in [("calc(infinity)", "33554432"),
        ("calc(1 / 0)", "33554432"), ("1e40", "33554432"),
        ("calc(1e40)", "33554432"), ("1e400", "33554432"),
        ("1 / calc(infinity)", "1 / 33554432"),
        ("calc(infinity) / calc(infinity)", "1"),
        ("calc(infinity / infinity)", "0"), ("calc(0 * infinity)", "0"),
        ("calc(1e400 / 1e400)", "1"), ("calc(1e400 - 1e400)", "0")] {
        let expected=compile(&input(&format!("#a{{aspect-ratio:{expected}}}"))).unwrap();
        for value in [expression.to_string(), "var(--ratio)".into(), format!("var(--missing, {expression})")] {
            let css=format!("#a{{--ratio:{expression};aspect-ratio:{value}}}");
            let actual=compile(&input(&css)).unwrap();
            assert_eq!(actual.riv,expected.riv,"{css}");
            assert_eq!(actual.runtime_requirements,expected.runtime_requirements,"{css}");
        }
    }
}

#[test]
fn invalid_scalar_ratio_substitution_resets_instead_of_falling_back() {
    let expected=compile(&input("#a{aspect-ratio:auto}")).unwrap();
    for expression in ["-1", "1/-2", "auto auto", "2/3/4", "calc(1+2)",
        "calc(1 + )", "min()", "clamp(1, 2)", "garbage"] {
        for value in ["var(--ratio)".to_string(), format!("var(--missing, {expression})")] {
            let css=format!("#a{{--ratio:{expression};aspect-ratio:2;aspect-ratio:{value}}}");
            let actual=compile(&input(&css)).unwrap();
            assert_eq!(actual.riv,expected.riv,"{css}");
            assert_eq!(actual.runtime_requirements,expected.runtime_requirements,"{css}");
        }
    }
    // Valid CSS outside our scalar evaluator retains an explicit diagnostic.
    for expression in ["sin(1)", "calc(1vw / 1px)"] {
        assert!(compile(&input(&format!("#a{{--ratio:{expression};aspect-ratio:var(--ratio)}}"))).is_err());
    }
}

#[test]
fn ratio_substitution_preserves_escaped_css_wide_keywords() {
    let expected=compile(&input("#root{aspect-ratio:2}#a{aspect-ratio:inherit}")).unwrap();
    let actual=compile(&input(r"#root{aspect-ratio:2}#a{aspect-ratio:var(--missing, \69 nherit)}")).unwrap();
    assert_eq!(actual.riv,expected.riv);
    assert_eq!(actual.runtime_requirements,expected.runtime_requirements);
}

#[test]
fn absolute_typed_ratio_math_and_invalid_substitution_match_chrome() {
    for (expression, expected) in [("calc(1px / 1px)", "1"),
        ("calc(1in / 1px)", "96"), ("calc(1s / 500ms)", "2"),
        ("calc(1% / 1%)", "1"), ("calc(1px * 1s / 1px / 1s)", "1"),
        ("calc(min(1in, 100px) / 1px)", "96")] {
        let expected=compile(&input(&format!("#a{{aspect-ratio:{expected}}}"))).unwrap();
        for value in [expression.to_string(), "var(--ratio)".into(), format!("var(--missing, {expression})")] {
            let actual=compile(&input(&format!("#a{{--ratio:{expression};aspect-ratio:{value}}}"))).unwrap();
            assert_eq!(actual.riv,expected.riv,"{expression}");
        }
    }
    let expected=compile(&input("#a{aspect-ratio:auto}")).unwrap();
    for expression in ["calc(1px)", "calc(1px + 1px)", "calc(1px * 2)",
        "calc(1px / 2)", "calc(1px * 1px)", "calc(1px + 1)",
        "calc(1px - 1px)", "calc(0 * 1px)", "min(1px, 2px)",
        "clamp(1px, 2px, 3px)", "clamp(1, 2px, 3)", "calc(1px / 1s)"] {
        for value in ["var(--ratio)".to_string(), format!("var(--missing, {expression})")] {
            let actual=compile(&input(&format!("#a{{--ratio:{expression};aspect-ratio:2;aspect-ratio:{value}}}"))).unwrap();
            assert_eq!(actual.riv,expected.riv,"{expression}");
        }
    }
}

#[test]
fn unit_conversion_extremes_follow_chrome_evaluation_order() {
    for (expression, expected) in [("calc(1e308in / 1e308in)", "1"),
        ("calc(1e308in / 1e308px)", "96"),
        ("calc(1e308px / 1e308in)", "1/96"),
        ("calc(1e-320ms / 1e-320ms)", "calc(infinity)"),
        ("calc(1e40 / 1e39)", "1"), ("calc(1e40 - 1e39)", "0")] {
        let expected=compile(&input(&format!("#a{{aspect-ratio:{expected}}}"))).unwrap();
        for value in [expression.to_string(), "var(--ratio)".into(), format!("var(--missing, {expression})")] {
            let actual=compile(&input(&format!("#a{{--ratio:{expression};aspect-ratio:{value}}}"))).unwrap();
            assert_eq!(actual.riv,expected.riv,"{expression}");
            assert_eq!(actual.runtime_requirements,expected.runtime_requirements,"{expression}");
        }
    }
}

#[test]
fn exact_ratio_pair_contract_is_versioned_and_validated() {
    let result = compile(&input("#root{font-size:.001px;aspect-ratio:calc(1em / 16px)}#a{aspect-ratio:inherit}")).unwrap();
    let good = result.runtime_requirements;
    assert_eq!(good.version, 15);
    assert_eq!(good.layout_aspect_ratios.len(), 2);
    assert!(good.layout_aspect_ratios.iter().all(|entry| entry.pair == Some([1, 15999])));
    let caps = good.capabilities.iter().copied().collect::<Vec<_>>();
    good.ensure_supported(&caps).unwrap();
    let old_caps = caps.iter().copied().filter(|c| *c != RuntimeCapability::LayoutCssAspectRatioPairV1).collect::<Vec<_>>();
    assert!(good.ensure_supported(&old_caps).is_err());
    for pair in [None, Some([0, 1]), Some([1, 0]), Some([u32::MAX, 1]), Some([1, u32::MAX])] {
        let mut bad = good.clone();
        bad.layout_aspect_ratios[0].pair = pair;
        assert_eq!(bad.ensure_supported(&caps).unwrap_err().code, "invalid-layout-aspect-ratio-pair");
    }
    let mut bad = good.clone();
    bad.version = 14;
    assert_eq!(bad.ensure_supported(&caps).unwrap_err().code, "invalid-layout-aspect-ratio-pair");
    let mut legacy = good.clone();
    legacy.version = 14;
    legacy.capabilities.remove(&RuntimeCapability::LayoutCssAspectRatioPairV1);
    for entry in &mut legacy.layout_aspect_ratios { entry.pair = None; }
    legacy.ensure_supported(&old_caps).unwrap();
    let serialized = serde_json::to_value(&legacy).unwrap();
    assert!(serialized["layout_aspect_ratios"][0].get("pair").is_none());
    let decoded: nuxie_html_to_riv::RuntimeRequirements = serde_json::from_value(serialized).unwrap();
    assert_eq!(decoded, legacy);
}
