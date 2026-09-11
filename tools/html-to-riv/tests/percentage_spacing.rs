use nuxie_html_to_riv::{compile, CompileInput};

fn input(css: &str) -> CompileInput {
    CompileInput { html: "<div id=root><div id=a></div></div>".into(),
        css: css.into(), width: 390., height: 320., ..Default::default() }
}
fn equivalent(a: &str, b: &str) {
    let a = compile(&input(a)).unwrap();
    let b = compile(&input(b)).unwrap();
    assert_eq!(a.riv, b.riv);
    assert_eq!(a.runtime_requirements, b.runtime_requirements);
}

#[test]
fn shorthand_variables_and_physical_longhands_agree() {
    for property in ["padding", "margin"] {
        for (short, expanded) in [
            ("5%", ["5%", "5%", "5%", "5%"]),
            ("5% 7px", ["5%", "7px", "5%", "7px"]),
            ("5% 7px 11%", ["5%", "7px", "11%", "7px"]),
            ("5% 7px 11% 13%", ["5%", "7px", "11%", "13%"]),
        ] {
            let long = ["top", "right", "bottom", "left"].into_iter().zip(expanded)
                .map(|(side, value)| format!("{property}-{side}:{value};")).collect::<String>();
            equivalent(&format!("#a{{{property}:{short}}}"), &format!("#a{{{long}}}"));
            equivalent(&format!("#a{{--space:{short};{property}:var(--space)}}"), &format!("#a{{{long}}}"));
        }
        equivalent(&format!("#root{{{property}:5%}}#a{{{property}:inherit}}"),
            &format!("#root,#a{{{property}:5%}}"));
        equivalent(&format!("#a{{font-size:20px;{property}:5% 1em}}"),
            &format!("#a{{font-size:20px;{property}:5% 20px}}"));
        equivalent(&format!("#a{{{property}:5%!important;{property}:7px}}"), &format!("#a{{{property}:5%}}"));
        equivalent(&format!("#a{{{property}:5%;{property}:unset}}"), &format!("#a{{{property}:0}}"));
    }
    equivalent("#a{margin:5% auto 7% 9px}",
        "#a{margin-top:5%;margin-right:auto;margin-bottom:7%;margin-left:9px}");
}

#[test]
fn percentage_spacing_profile_bounds_are_explicit() {
    for property in ["padding", "margin"] {
        for value in ["0%", "100%", "10000%"] {
            compile(&input(&format!("#a{{{property}:{value}}}"))).unwrap();
        }
        for value in ["10001%", "1e30%"] {
            assert!(compile(&input(&format!("#a{{{property}:{value}}}"))).is_err(), "{property}:{value}");
        }
    }
    assert!(compile(&input("#a{padding:auto}")).is_err());
    assert!(compile(&input("#a{padding:-1%}")).is_err());
}

#[test]
fn percentage_spacing_requires_validated_version_16_targets() {
    use nuxie_html_to_riv::RuntimeCapability;
    let output = compile(&input("#a{padding:3%;margin:2%}" )).unwrap();
    let valid = output.runtime_requirements;
    assert_eq!(valid.version, 16);
    assert_eq!(valid.layout_percentage_spacing.len(), 1);
    let supported: Vec<_> = valid.capabilities.iter().copied().collect();
    valid.ensure_supported(&supported).unwrap();
    let missing: Vec<_> = supported.iter().copied()
        .filter(|c| *c != RuntimeCapability::LayoutCssPercentageSpacingV1).collect();
    assert!(valid.ensure_supported(&missing).is_err());
    assert!(valid.ensure_layout_targets(|_| false).is_err());
    for change in 0..4 {
        let mut invalid = valid.clone();
        match change {
            0 => invalid.version = 15,
            1 => invalid.layout_percentage_spacing.clear(),
            2 => invalid.layout_percentage_spacing.push(invalid.layout_percentage_spacing[0]),
            _ => { invalid.capabilities.remove(&RuntimeCapability::LayoutCssPercentageSpacingV1); }
        }
        assert!(invalid.ensure_supported(&supported).is_err());
    }
    let plain = compile(&input("#a{padding:3px}" )).unwrap();
    assert!(plain.runtime_requirements.layout_percentage_spacing.is_empty());
    assert!(plain.runtime_requirements.version < 16);
    let combined = compile(&input("#a{padding:3%;aspect-ratio:2}" )).unwrap();
    let requirements = combined.runtime_requirements;
    requirements.ensure_supported(&requirements.capabilities.iter().copied().collect::<Vec<_>>()).unwrap();
}
