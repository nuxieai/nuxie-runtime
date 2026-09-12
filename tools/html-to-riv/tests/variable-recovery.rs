use nuxie_html_to_riv::{CompileInput, compile};
use serde::Deserialize;
#[path = "support/value_token_history.rs"] mod value_token_history;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case { name: String, html: String, css: String, literal_css: String }

#[test]
fn failed_substitutions_match_explicit_unset_without_cascade_rollback() {
    let cases: Vec<Case> = serde_json::from_str(include_str!("../validation/public-variable-recovery-cases.json")).unwrap();
    assert_eq!(cases.len(), 24);
    for case in cases {
        for (width, height) in [(240.,160.), (390.,200.), (768.,120.)] {
            let request = |css: &str| CompileInput { html:case.html.clone(), css:css.into(), width, height };
            let result = compile(&request(&case.css)).unwrap_or_else(|d| panic!("{}: {d:?}",case.name));
            assert_eq!(result, compile(&request(&case.literal_css)).unwrap(), "{} {width}x{height}",case.name);
            assert_eq!(result, compile(&request(&case.css)).unwrap(), "{} deterministic",case.name);
        }
    }
}

#[test]
fn recovery_preserves_inline_specificity_importance_and_dom_selector_order() {
    let html = "<div id=p><div id=a class=item style='color:var(--missing);order:var(--missing)'></div><div id=b></div></div>";
    let css = "#p{width:120px;height:80px;color:navy}#a,#b{width:20px;height:20px}#a{color:red;order:-4}.item{color:teal}#a:first-child{background:currentColor}#b{background:gold;order:-1}";
    let request = |html: &str, css: &str| CompileInput { html:html.into(), css:css.into(), width:240.,height:160. };
    let result = compile(&request(html,css)).unwrap();
    assert_eq!(result, compile(&request(&html.replace("var(--missing)","unset"),css)).unwrap());
    // A winning important invalid value must remain important when reset.
    let important = format!("{css}#a{{color:var(--missing)!important}} .item{{color:coral!important}}");
    assert_eq!(compile(&request(html,&important)).unwrap(),compile(&request(&html.replace("var(--missing)","unset"),&important.replace("var(--missing)","unset"))).unwrap());
}

#[test]
fn historical_sources_preserve_explicit_recovery_and_retained_admission_errors() {
    #[derive(Deserialize)] struct Rejection {name:String,html:String,css:String}
    let cases: Vec<Rejection> = serde_json::from_str(include_str!("../validation/public-variable-recovery-rejections.json")).unwrap();
    assert_eq!(cases.len(),16);
    for case in cases {
        let request=CompileInput {html:case.html,css:case.css,width:240.,height:160.};
        if value_token_history::assert_recovery("public-variable-recovery-rejections.json",&case.name,&request) { continue; }
        let diagnostics=compile(&request).expect_err(&case.name);
        assert!(!diagnostics.is_empty(),"{}",case.name);
        assert!(diagnostics.iter().all(|d|!d.code.is_empty()&&!d.source.is_empty()&&!d.message.is_empty()),"{}",case.name);
    }
}

#[test]
fn failed_substitution_does_not_swallow_variable_resource_errors() {
    let mut css=String::from("#a{--v0:x;");
    for n in 1..25 {css.push_str(&format!("--v{n}:var(--v{}) var(--v{});",n-1,n-1));}
    css.push_str("width:var(--missing) var(--v24)}");
    let diagnostics=compile(&CompileInput {html:"<div id=a></div>".into(),css,width:240.,height:160.}).unwrap_err();
    assert!(diagnostics.iter().any(|d|d.code=="input-limit"),"{diagnostics:?}");
    // This environment is bounded. Only the ordinary value's later siblings
    // exceed the limit, after its first substitution has already failed.
    let environment=format!("--chunk:{};", "x".repeat(24_000));
    let input=|copies: usize| CompileInput {html:"<div id=a></div>".into(),css:format!("#a{{{environment}width:var(--missing) {}}}","var(--chunk) ".repeat(copies)),width:240.,height:160.};
    assert!(compile(&input(1)).is_ok(),"bounded sibling must retain guaranteed-invalid result");
    let diagnostics=compile(&input(3)).unwrap_err();
    assert!(diagnostics.iter().any(|d|d.code=="input-limit"),"later excessive sibling must not be hidden: {diagnostics:?}");
}

#[test]
fn unsupported_recovered_value_keeps_its_original_source_location() {
    let errors=compile(&CompileInput {html:"<div id=a style='display:var(--missing)'></div>".into(),css:String::new(),width:240.,height:160.}).unwrap_err();
    assert!(errors.iter().any(|d|d.code=="unsupported-target-semantics" && d.source.starts_with("html#a@style:")),"{errors:?}");
}
