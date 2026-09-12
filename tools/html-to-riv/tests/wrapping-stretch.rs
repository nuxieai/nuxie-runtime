//! Public fixed stretch admission. These API assertions are complemented by
//! the separate native/Chrome geometry and pixel campaign; bytes alone do not
//! establish that visible rectangles retain their fixed size.
use nuxie_html_to_riv::{compile, CompileInput, CompileOutput};

fn input(extra: &str) -> CompileInput {
    CompileInput {
        html: "<div id=p><div id=a></div><div id=b></div><div id=c></div></div>".into(),
        css: format!("#p{{width:20px;height:120px;flex-direction:row;flex-wrap:wrap}}#a,#b,#c{{width:20px;height:20px;background:red}}{extra}"),
        width: 320., height: 200., assets: Default::default(),
    }
}
fn scene(extra: &str) -> CompileOutput { compile(&input(extra)).unwrap() }
fn rejects(extra: &str, reason: &str) {
    let errors = compile(&input(extra)).expect_err("unqualified arithmetic must diagnose");
    assert!(errors.iter().any(|e| e.code == "unsupported-target-semantics" && e.message.contains(reason)), "{errors:?}");
}
#[test]
fn omitted_normal_and_stretch_are_identical_and_deterministic() {
    let default = scene("");
    for content in ["normal", "stretch", "initial", "unset"] {
        assert_eq!(default, scene(&format!("#p{{align-content:{content}}}")));
    }
    assert_eq!(default, scene(""));
    // Positive cross free space must be represented, rather than silently using
    // the existing positional cross-start construction.
    assert_ne!(default.riv, scene("#p{align-content:flex-start}").riv);
}
#[test]
fn both_axes_and_reversals_keep_dom_identities_with_stable_css_order() {
    for direction in ["row", "row-reverse", "column", "column-reverse"] {
        let dimensions = if direction.starts_with("row") { "width:20px;height:120px" } else { "width:120px;height:20px" };
        for wrap in ["wrap", "wrap-reverse"] {
            let css = format!("#p{{{dimensions};flex-direction:{direction};flex-wrap:{wrap}}}#b{{order:-1;align-self:center}}#a,#c{{order:2}}#c{{align-self:flex-end;background:transparent}}");
            let actual = scene(&css);
            assert_eq!(actual, scene(&format!("{css}#p{{align-content:stretch}}")));
            assert_eq!(actual.source_map.iter().map(|n| (n.id.as_str(), n.path.as_str())).collect::<Vec<_>>(),
                [("p", "/0"), ("a", "/0/0"), ("b", "/0/1"), ("c", "/0/2")]);
            assert!(actual.source_map[2].object_id < actual.source_map[1].object_id);
            assert!(actual.source_map[1].object_id < actual.source_map[3].object_id);
        }
    }
}
#[test]
fn fixed_visible_bounds_are_admitted_without_becoming_slot_limits() {
    let authored = "#p{width:20px;height:120px}#a,#b,#c{min-width:20px;max-width:10px;min-height:10px;max-height:10px}";
    let actual = scene(authored);
    assert_eq!(actual, scene(&format!("{authored}#p{{align-content:stretch}}")));
    // The original max-height must not prevent the larger generated line slot.
    // This also retains the authored min-over-max width conflict in the input.
    assert_eq!(actual.source_map.len(), 4);
    assert_ne!(actual.riv, scene(&format!("{authored}#a,#b,#c{{max-height:20px}}")).riv);
}
#[test]
fn invalid_at_computed_value_time_content_uses_normal() {
    assert_eq!(scene(""), scene("#p{align-content:flex-end;--bad:potato;align-content:var(--bad)}"));
    assert_eq!(scene(""), scene("#p{align-content:flex-end;align-content:var(--missing)}"));
    assert_eq!(scene("#p{align-content:stretch}"), scene("#p{--lines:stretch;align-content:var(--lines)}"));
}
#[test]
fn nowrap_does_not_gain_stretch_helpers() {
    let base = "#p{flex-wrap:nowrap}";
    let actual = scene(base);
    for content in ["normal", "stretch", "initial", "unset"] {
        assert_eq!(actual, scene(&format!("{base}#p{{align-content:{content}}}")));
    }
}
#[test]
fn remainder_and_arithmetic_domain_are_explicit_diagnostics() {
    rejects("#p{height:100px}", "Remainder");
    rejects("#p{height:65537px}", "ArithmeticBound");
    rejects("#a{max-height:65537px}", "ArithmeticBound");
}
