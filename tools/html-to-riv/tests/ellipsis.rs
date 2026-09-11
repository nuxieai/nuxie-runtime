use nuxie_html_to_riv::{Asset, CompileInput, RuntimeCapability, TextPolicy, compile};

fn source(html: &str, css: &str) -> CompileInput {
    let mut source = CompileInput { html: html.into(), css: css.into(), width: 390.0, height: 200.0, ..Default::default() };
    source.assets.insert("font".into(), Asset::Font { family: "Inter".into(), weight: 400, bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec() });
    source
}
const BASE: &str = "p{display:block;font:24px/40px Inter;white-space:nowrap;overflow:hidden;width:100%}";

#[test]
fn ellipsis_emits_one_checked_occurrence_and_keeps_source_text() {
    let normal = compile(&source("<p>ffi long title</p>", BASE)).unwrap();
    let ellipsis = compile(&source("<p>ffi long title</p>", &format!("{BASE}p{{text-overflow:ellipsis}}"))).unwrap();
    assert_eq!(normal.riv, ellipsis.riv, "ellipsis is installed at runtime, not baked into text or layout");
    assert_eq!(serde_json::to_value(normal.source_map).unwrap(), serde_json::to_value(ellipsis.source_map).unwrap());
    let requirements = &ellipsis.runtime_requirements;
    assert_eq!(requirements.version, 5);
    assert_eq!(requirements.text_policies.len(), 1);
    assert_eq!(requirements.text_policies[0].policy, TextPolicy::CssSingleLineEllipsisV1);
    requirements.ensure_supported(&[RuntimeCapability::LayoutCssPixelBoundsV1, RuntimeCapability::TextCssShapingPrecisionV1, RuntimeCapability::TextCssSingleLineEllipsisV1]).unwrap();
    assert_eq!(requirements.ensure_supported(&[RuntimeCapability::TextCssShapingPrecisionV1]).unwrap_err().code, "missing-runtime-capability");
}

#[test]
fn ellipsis_obeys_cascade_and_noninherited_css_wide_keywords() {
    for (rule, expected) in [("", false), ("text-overflow:inherit", true), ("text-overflow:unset", false), ("text-overflow:initial", false), ("text-overflow:ellipsis;text-overflow:clip", false), ("text-overflow:ellipsis!important;text-overflow:clip", true)] {
        let output = compile(&source("<div><p>long text</p></div>", &format!("{BASE}div{{text-overflow:ellipsis}}p{{{rule}}}"))).unwrap();
        assert_eq!(output.runtime_requirements.capabilities.contains(&RuntimeCapability::TextCssSingleLineEllipsisV1), expected, "{rule}");
    }
}

#[test]
fn ellipsis_rejects_unqualified_syntax_and_formatting_contexts() {
    for value in ["'more'", "clip ellipsis", "ellipsis clip", "fade"] {
        assert_eq!(compile(&source("<p>text</p>", &format!("{BASE}p{{text-overflow:{value}}}"))).unwrap_err()[0].code, "unsupported-text-overflow");
    }
    for (html, extra) in [("<p>text</p>", "display:flex"), ("<p>text</p>", "white-space:normal"), ("<p>text</p>", "white-space:pre"), ("<p>text</p>", "overflow:visible"), ("<p>one<br>two</p>", "")] {
        assert_eq!(compile(&source(html, &format!("{BASE}p{{text-overflow:ellipsis;{extra}}}"))).unwrap_err()[0].code, "unsupported-text-overflow-combination");
    }
    compile(&source("<p>text</p>", &format!("{BASE}p{{text-overflow:ellipsis;overflow:clip}}"))).unwrap();
}
