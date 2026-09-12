//! Static data attributes affect source selection, never the emitted runtime contract.
use nuxie_html_to_riv::{CompileInput, compile};

fn input(html: &str, css: &str) -> CompileInput {
    CompileInput { html: html.into(), css: css.into(), width: 240., height: 160. }
}

#[test]
fn data_selector_corpus_equals_independent_id_rules_without_attribute_metadata() {
    let fixtures: serde_json::Value = serde_json::from_str(include_str!("../validation/public-data-attributes-cases.json")).unwrap();
    let fixtures = fixtures.as_array().unwrap();
    assert_eq!(fixtures.len(), 16);
    for fixture in fixtures {
        for (width, height) in [(240., 160.), (390., 200.), (768., 120.)] {
            let mut source = input(fixture["html"].as_str().unwrap(), fixture["css"].as_str().unwrap());
            source.width = width; source.height = height;
            let control = CompileInput { html: fixture["controlHtml"].as_str().unwrap().into(), css: fixture["controlCss"].as_str().unwrap().into(), ..source.clone() };
            let actual = compile(&source).unwrap_or_else(|e| panic!("{}: {e:?}", fixture["name"]));
            assert_eq!(actual, compile(&control).unwrap(), "{} at {width}x{height}", fixture["name"]);
            assert_eq!(actual, compile(&source).unwrap(), "determinism: {}", fixture["name"]);
        }
    }
}

#[test]
fn unselected_data_is_inert_and_does_not_generate_identity_or_file_properties() {
    let plain = compile(&input("<div><section></section></div>", "div{width:100px;height:60px}section{width:20px;height:10px;background:teal}")).unwrap();
    for attrs in ["data-id='pretend' data-binding='true' data-action='run()'", "data-src='https://example.invalid/image.png' data-style='display:grid'", "DATA-雪='\"value\"' data-0='zero' data--state='empty-prefix'"] {
        let source = format!("<div {attrs}><section {attrs}></section></div>");
        let actual = compile(&input(&source, "div{width:100px;height:60px}section{width:20px;height:10px;background:teal}")).unwrap();
        assert_eq!(plain, actual, "{attrs}");
    }
}

#[test]
fn data_rules_preserve_inline_important_and_variable_cascade() {
    let html = "<div id='box' data-state='ready' style='background:navy'></div>";
    let base = "#box{width:80px;height:20px}";
    let selected = compile(&input(html, &format!("{base}[data-state=ready]{{--paint:coral;background:var(--paint)!important}}"))).unwrap();
    let control = compile(&input("<div id='box'></div>", &format!("{base}#box{{background:coral}}"))).unwrap();
    assert_eq!(selected, control);
    let inline = compile(&input(html, &format!("{base}[data-state=ready]{{background:coral}}"))).unwrap();
    let control = compile(&input("<div id='box'></div>", &format!("{base}#box{{background:navy}}"))).unwrap();
    assert_eq!(inline, control);
}

#[test]
fn ordinary_behavior_resource_and_empty_data_names_remain_diagnostic() {
    for attribute in ["onclick='run()'", "onload='run()'", "src='image.png'", "href='/route'", "hidden", "contenteditable", "tabindex='0'", "role='button'", "aria-label='label'", "data-='empty-name'"] {
        let errors = compile(&input(&format!("<div data-state='ready' {attribute}></div>"), "")).unwrap_err();
        assert_eq!(errors[0].code, "unsupported-target-semantics", "{attribute}: {errors:?}");
        assert!(errors[0].message.contains("Attribute"), "{attribute}: {errors:?}");
    }
}

#[test]
fn data_attributes_do_not_relax_selector_or_declaration_validation() {
    for css in ["[data-state=x s]{background:red}", "[data-state=x q]{background:red}", "[ns|data-state]{background:red}", "[data-state=x]:hover{background:red}", "[data-state=absent]{display:grid}"] {
        assert!(compile(&input("<div data-state=ready></div>", css)).is_err(), "{css}");
    }
    for html in ["<div data-state='first' DATA-STATE='second'></div>", "<div data-state='unterminated></div>"] {
        assert_eq!(compile(&input(html, "")).unwrap_err()[0].code, "html-syntax");
    }
}

#[test]
fn data_payloads_keep_existing_authoring_resource_limits() {
    let html = format!("<div data-payload='{}'></div>", "x".repeat(1_048_576));
    assert_eq!(compile(&input(&html, "")).unwrap_err()[0].code, "input-limit");
}
