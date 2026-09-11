use nuxie_html_to_riv::{CompileInput, CompileOutput, compile};

fn css(value: &str) -> CompileOutput {
    compile(&CompileInput {
        html: "<div id=root><div id=a></div></div>".into(),
        css: value.into(),
        width: 390.,
        height: 320.,
        ..Default::default()
    })
    .unwrap()
}
fn equivalent(a: &str, b: &str) {
    let a = css(a);
    let b = css(b);
    assert_eq!(a.riv, b.riv);
    assert_eq!(
        serde_json::to_value(a.source_map).unwrap(),
        serde_json::to_value(b.source_map).unwrap()
    );
    assert_eq!(a.runtime_requirements, b.runtime_requirements);
}

#[test]
fn auto_margin_shorthand_cascade_and_variables() {
    for (short, long) in [
        ("auto", "auto auto auto auto"),
        ("auto 5px", "auto 5px auto 5px"),
        ("4px auto 8px", "4px auto 8px auto"),
        ("auto 3px 7px auto", "auto 3px 7px auto"),
    ] {
        let v: Vec<_> = long.split_whitespace().collect();
        let explicit = format!(
            "#a{{margin-top:{};margin-right:{};margin-bottom:{};margin-left:{}}}",
            v[0], v[1], v[2], v[3]
        );
        equivalent(&format!("#a{{margin:{short}}}"), &explicit);
        equivalent(&format!("#a{{--m:{short};margin:var(--m)}}"), &explicit);
    }
    for side in ["top", "right", "bottom", "left"] {
        equivalent(
            &format!("#a{{margin-{side}:AUTO}}"),
            &format!("#a{{margin-{side}:auto}}"),
        );
        equivalent(
            &format!("#root{{margin-{side}:auto}}#a{{margin-{side}:inherit}}"),
            &format!("#root,#a{{margin-{side}:auto}}"),
        );
        for reset in ["initial", "unset", "var(--missing)", "var(--bad)"] {
            equivalent(
                &format!("#a{{--bad:banana;margin-{side}:auto;margin-{side}:{reset}}}"),
                "#a{margin:0}",
            );
        }
    }
    equivalent(
        "#root{margin:auto}#a{margin:inherit}",
        "#root,#a{margin:auto}",
    );
    equivalent("#a{margin:auto!important;margin:5px}", "#a{margin:auto}");
    equivalent(
        "#a{margin:auto;margin-left:3px}",
        "#a{margin:auto auto auto 3px}",
    );
    equivalent("#a{margin-left:3px;margin:auto}", "#a{margin:auto}");
    assert_ne!(css("#a{margin:auto}").riv, css("#a{margin:0}").riv);
}

#[test]
fn auto_margin_preserves_explicit_exclusions() {
    for declaration in [
        "padding:auto",
        "padding-left:auto",
        "margin:auto auto auto auto auto",
        "margin:auto banana",
        "margin:1px, auto",
    ] {
        let errors = compile(&CompileInput {
            html: "<div></div>".into(),
            css: format!("div{{{declaration}}}"),
            width: 390.,
            height: 320.,
            ..Default::default()
        })
        .unwrap_err();
        assert!(
            errors.iter().any(|d| d.code == "unsupported-value"),
            "{declaration}: {errors:?}"
        );
    }
}
