//! Compare public variable compilation with hand-written literal controls.
//! The controls enumerate CSS results, rather than running another resolver.
use nuxie_html_to_riv::{CompileInput, compile};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    name: String,
    html: String,
    css: String,
    literal_css: String,
}

#[test]
fn public_variables_equal_independent_literal_controls() {
    let cases: Vec<Case> = serde_json::from_str(include_str!(
        "../validation/public-variable-cases.json"
    )).unwrap();
    assert!(cases.len() >= 17);
    for case in cases {
        for (width, height) in [(240., 160.), (390., 200.), (768., 120.)] {
            let input = |css: &str| CompileInput {
                html: case.html.clone(), css: css.into(), width, height,
            };
            let actual = compile(&input(&case.css))
                .unwrap_or_else(|error| panic!("{}: {error:?}", case.name));
            let expected = compile(&input(&case.literal_css))
                .unwrap_or_else(|error| panic!("{} literal: {error:?}", case.name));
            assert_eq!(actual, expected, "{} at {width}x{height}", case.name);
            assert_eq!(actual, compile(&input(&case.css)).unwrap(), "{} deterministic", case.name);
        }
    }
}

#[test]
fn public_variables_cannot_bypass_unsupported_semantics() {
    for css in [
        "#a{--mode:grid;display:var(--mode)}",
        "#a{width:var(--missing)}",
        "#a{--n:10;width:var(--n)px}",
        "#a{--n:teal;width:var(--n)}",
        "#a{--cycle:var(--cycle);background:var(--cycle)}",
        "#a{--n:10px;width:var(n)}",
    ] {
        let input = CompileInput {html:"<div id=a></div>".into(),css:css.into(),width:240.,height:160.};
        let diagnostics = compile(&input).expect_err(css);
        assert!(!diagnostics.is_empty(), "{css}");
        assert!(diagnostics.iter().all(|d| !d.code.is_empty() && !d.message.is_empty()), "{css}: {diagnostics:?}");
    }
}

#[test]
fn exponential_variable_expansion_has_a_bounded_failure() {
    let mut css = String::from("#a{--v0:x;");
    for n in 1..25 {
        css.push_str(&format!("--v{n}:var(--v{}) var(--v{});", n-1, n-1));
    }
    css.push_str("width:var(--v24)}");
    let input = CompileInput {html:"<div id=a></div>".into(),css,width:240.,height:160.};
    assert!(compile(&input).is_err());
}

#[test]
fn disputed_fallback_cycle_graphs_are_explicitly_rejected() {
    let cases: Vec<Case> = serde_json::from_str(include_str!("../validation/public-variable-rejected-cycles.json")).unwrap();
    assert_eq!(cases.len(), 2);
    for case in cases {
        let input = CompileInput { html:case.html,css:case.css,width:240.,height:160. };
        let errors = compile(&input).expect_err(&case.name);
        assert!(errors.iter().any(|e| e.message.contains("fallback")), "{errors:?}");
    }
}
