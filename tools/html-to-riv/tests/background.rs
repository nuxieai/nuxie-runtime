//! Shorthand cascade lowering only; ordinary-runtime visual proof is separate.
use nuxie_html_to_riv::{CompileInput, compile};
fn output(html: &str, css: &str) -> nuxie_html_to_riv::CompileOutput {
    compile(&CompileInput {
        html: html.into(),
        css: css.into(),
        width: 390.,
        height: 160.,
    })
    .unwrap()
}
#[test]
fn shorthand_and_longhand_obey_source_order_specificity_and_important() {
    let html = "<div id=box class=paint></div>";
    for (css, color) in [
        ("#box{background:red;background-color:blue}", "blue"),
        ("#box{background-color:blue;background:red}", "red"),
        (
            "#box{background:red!important;background-color:blue}",
            "red",
        ),
        (
            "#box{background-color:blue!important;background:red}",
            "blue",
        ),
        ("#box{background:red}.paint{background-color:blue}", "red"),
        (
            ".paint{background:red!important}#box{background-color:blue}",
            "red",
        ),
    ] {
        assert_eq!(
            output(html, css),
            output(html, &format!("#box{{background-color:{color}}}")),
            "{css}"
        );
    }
}
#[test]
fn inline_precedence_applies_to_both_shorthand_directions() {
    for (inline, css, color) in [
        ("background:blue", "#box{background-color:red}", "blue"),
        (
            "background:blue",
            "#box{background-color:red!important}",
            "red",
        ),
        (
            "background:blue!important",
            "#box{background-color:red!important}",
            "blue",
        ),
        (
            "background-color:blue!important",
            "#box{background:red!important}",
            "blue",
        ),
    ] {
        let actual = output(&format!("<div id=box style='{inline}'></div>"), css);
        assert_eq!(
            actual,
            output(
                "<div id=box></div>",
                &format!("#box{{background-color:{color}}}")
            )
        );
    }
}
#[test]
fn inherited_shorthand_currentcolor_is_resolved_by_each_recipient() {
    let html = "<section id=parent><div id=child><div id=grandchild></div></div></section>";
    let colors = "#parent{color:red}#child{color:blue}#grandchild{color:green}";
    assert_eq!(
        output(
            html,
            &format!(
                "{colors}#parent{{background:currentColor}}#child,#grandchild{{background:inherit}}"
            )
        ),
        output(
            html,
            "#parent{background-color:red}#child{background-color:blue}#grandchild{background-color:green}"
        )
    );
    // A literal parent paint remains literal despite descendant foreground changes.
    assert_eq!(
        output(
            html,
            &format!("{colors}#parent{{background:red}}#child,#grandchild{{background:inherit}}")
        ),
        output(html, "#parent,#child,#grandchild{background-color:red}")
    );
    assert_eq!(
        output(
            "<div id=box></div>",
            "#box{background:currentColor;color:blue}"
        ),
        output("<div id=box></div>", "#box{background-color:blue}")
    );
}
#[test]
fn alpha_literal_forms_and_full_shorthand_resets_match_longhand_controls() {
    let html = "<div id=box></div>";
    for color in [
        "rebeccapurple",
        "#1234",
        "#12345678",
        "rgba(255,0,0,.5)",
        "hsl(.5turn 100% 50% / 25%)",
        "transparent",
    ] {
        assert_eq!(
            output(html, &format!("#box{{background:{color}}}")),
            output(html, &format!("#box{{background-color:{color}}}"))
        );
    }
    for reset in ["none", "initial", "unset", "NONE"] {
        assert_eq!(
            output(
                html,
                &format!("#box{{background-color:red;background:{reset}}}")
            ),
            output(html, "#box{background-color:transparent}")
        );
    }
}
#[test]
fn unsupported_constituents_are_rejected_with_authored_sources() {
    for value in [
        "red,blue",
        "linear-gradient(red,blue)",
        "url(photo.png)",
        "red center/cover",
        "red no-repeat",
        "red border-box",
        "fixed red",
        "inherit red",
        "red blue",
        "none red",
        "revert",
        "revert-layer",
    ] {
        for inline in [false, true] {
            let (html, css) = if inline {
                (
                    format!("<div id=box style='background:{value}'></div>"),
                    String::new(),
                )
            } else {
                (
                    "<div id=box></div>".into(),
                    format!("#unmatched{{background:{value}}}"),
                )
            };
            let errors = compile(&CompileInput {
                html,
                css,
                width: 390.,
                height: 160.,
            })
            .unwrap_err();
            assert_eq!(errors[0].code, "unsupported-target-semantics", "{value}");
            assert!(
                errors[0]
                    .source
                    .starts_with(if inline { "html#box@style:" } else { "css:" }),
                "{}",
                errors[0].source
            );
            assert!(errors[0].message.contains("background"));
        }
    }
}
