//! Compiler descriptor and admission tests; visual resize proof is separate.
use nuxie_html_to_riv::{CompileInput, compile};
fn request(html: &str, css: &str, width: f32) -> CompileInput {
    CompileInput {
        html: html.into(),
        css: css.into(),
        width,
        height: 160.,
    }
}
fn equivalent(html: &str, actual: &str, expected: &str) {
    for width in [240., 390., 768.] {
        assert_eq!(
            compile(&request(html, actual, width)).unwrap(),
            compile(&request(html, expected, width)).unwrap(),
            "width {width}: {actual}"
        );
    }
}
#[test]
fn inherit_preserves_fractional_percentage_descriptors_across_generations() {
    let html = "<section id=parent><div id=child><div id=grandchild></div></div></section>";
    equivalent(
        html,
        "#parent{width:50%;height:50%}#child,#grandchild{width:inherit;height:inherit}",
        "#parent,#child,#grandchild{width:50%;height:50%}",
    );
    let responsive = compile(&request(
        html,
        "#parent{width:50%;height:50%}#child,#grandchild{width:inherit;height:inherit}",
        390.,
    ))
    .unwrap();
    let baked=compile(&request(html,"#parent{width:50%;height:50%}#child{width:97.5px;height:40px}#grandchild{width:48.75px;height:20px}",390.)).unwrap();
    assert_ne!(
        responsive.riv, baked.riv,
        "used pixel values must not replace inherited percentages"
    );
}
#[test]
fn fixed_lengths_and_host_percentage_sizes_inherit() {
    equivalent(
        "<section id=parent><div id=child></div></section>",
        "#parent{width:120px;height:80px}#child{width:inherit;height:inherit}",
        "#parent,#child{width:120px;height:80px}",
    );
    equivalent(
        "<div id=box></div>",
        "#box{width:inherit;height:inherit;display:inherit;flex-direction:inherit}",
        "#box{width:100%;height:100%;display:flex;flex-direction:column}",
    );
}
#[test]
fn inherited_auto_height_measures_own_contents_instead_of_parent_used_height() {
    let html = "<section id=parent><div id=child><div id=leaf></div></div><div id=sibling></div></section>";
    equivalent(
        html,
        "#parent{width:auto;height:auto}#child{width:inherit;height:inherit}#leaf{height:20px}#sibling{height:30px}",
        "#parent,#child{width:auto;height:auto}#leaf{height:20px}#sibling{height:30px}",
    );
    let actual = compile(&request(
        html,
        "#parent{height:auto}#child{height:inherit}#leaf{height:20px}#sibling{height:30px}",
        390.,
    ))
    .unwrap();
    let frozen = compile(&request(
        html,
        "#parent{height:auto}#child{height:50px}#leaf{height:20px}#sibling{height:30px}",
        390.,
    ))
    .unwrap();
    assert_ne!(actual.riv, frozen.riv);
}
#[test]
fn initial_and_unset_reset_dimensions_to_auto_with_cascade_precedence() {
    let html = "<section id=parent><div id=child class=box style='width:40px;height:40px'><div id=leaf></div></div></section>";
    for keyword in ["initial", "unset", "InItIaL", "UNSET"] {
        equivalent(
            html,
            &format!(
                "#parent{{width:120px;height:100px}}#child{{width:{keyword}!important;height:{keyword}!important}}.box{{width:80px;height:80px}}#leaf{{height:20px}}"
            ),
            "#parent{width:120px;height:100px}#child{width:auto!important;height:auto!important}.box{width:80px;height:80px}#leaf{height:20px}",
        );
    }
    equivalent(
        "<div id=box style='width:initial!important;height:unset!important'></div>",
        "#box{width:90px!important;height:70px!important}",
        "#box{width:auto!important;height:auto!important}",
    );
}
#[test]
fn computed_auto_parent_guard_survives_keyword_resolution() {
    for keyword in ["initial", "unset", "auto"] {
        let css = format!("#parent{{height:100px;height:{keyword}}}#child{{height:50%}}");
        let errors = compile(&request(
            "<div id=parent><div id=child></div></div>",
            &css,
            390.,
        ))
        .unwrap_err();
        assert_eq!(errors[0].code, "unsupported-target-semantics");
        assert!(errors[0].message.contains("auto-height parent"));
    }
}
#[test]
fn malformed_keywords_and_unqualified_display_values_remain_rejected() {
    for (property, value) in [
        ("width", "inherit 20px"),
        ("height", "initial auto"),
        ("width", "unset inherit"),
        ("width", "revert"),
        ("height", "revert-layer"),
        ("width", "in herit"),
        ("display", "initial"),
        ("display", "unset"),
        ("display", "inline"),
        ("flex-direction", "initial"),
        ("flex-direction", "unset"),
        ("flex-direction", "row"),
        ("display", "inherit flex"),
        ("flex-direction", "inherit column"),
    ] {
        let css = format!("#never{{{property}:{value}}}");
        let errors = compile(&request("<div id=box></div>", &css, 390.)).unwrap_err();
        assert_eq!(errors[0].code, "unsupported-target-semantics", "{css}");
        assert!(errors[0].source.starts_with("css:"));
    }
}
