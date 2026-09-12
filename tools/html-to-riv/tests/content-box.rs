//! Public content-box output, cascade isolation, diagnostics and resource bounds.
use nuxie_html_to_riv::{CompileInput, compile};
#[path = "support/value_token_history.rs"] mod value_token_history;
fn input(html:&str,css:&str)->CompileInput {CompileInput{html:html.into(),css:css.into(),width:240.,height:160.}}

#[test]
fn content_box_corpus_matches_independent_border_box_controls_and_repeats() {
    let cases:serde_json::Value=serde_json::from_str(include_str!("../validation/public-content-box-cases.json")).unwrap();
    assert_eq!(cases.as_array().unwrap().len(),34);
    for case in cases.as_array().unwrap() {
        for (width,height) in [(240.,160.),(390.,200.),(768.,120.)] {
            let mut request=input(case["html"].as_str().unwrap(),case["css"].as_str().unwrap());request.width=width;request.height=height;
            let output=compile(&request).unwrap_or_else(|e|panic!("{}: {e:?}",case["name"]));
            assert_eq!(output,compile(&request).unwrap(),"repeat {}",case["name"]);
            // Derived dimensions may exceed the source coefficient limit. Those
            // records are inspected by unit/native tests; a >1m control CSS
            // declaration intentionally cannot enter the public source grammar.
            if case["classification"]!="numeric-boundary" {
                request.css=case["controlCss"].as_str().unwrap().into();
                assert_eq!(output,compile(&request).unwrap(),"control {}",case["name"]);
            }
        }
    }
}
#[test]
fn unsupported_combinations_and_recursive_overflow_produce_diagnostics() {
    let cases:serde_json::Value=serde_json::from_str(include_str!("../validation/public-content-box-rejections.json")).unwrap();
    assert_eq!(cases.as_array().unwrap().len(),14);
    for case in cases.as_array().unwrap() {
        let request=input(case["html"].as_str().unwrap(),case["css"].as_str().unwrap());
        if value_token_history::assert_recovery("public-content-box-rejections.json",case["name"].as_str().unwrap(),&request) { continue; }
        let errors=compile(&request).unwrap_err();
        assert_eq!(errors[0].code,"unsupported-target-semantics","{}: {errors:?}",case["name"]);
        if case["name"]=="lowered-content-overflow-chain" {assert!(errors[0].message.contains("finite binary32"));}
    }
}
#[test]
fn accepted_depth_and_declared_source_limits_remain_available() {
    for depth in [1,64,128] {
        let html=format!("{}{}","<div>".repeat(depth),"</div>".repeat(depth));
        let output=compile(&input(&html,"div{box-sizing:content-box;width:1px;height:1px;padding:1px}")).unwrap();
        assert_eq!(output.source_map.len(),depth);
    }
    for depth in [129,130] {
        let html=format!("{}{}","<div>".repeat(depth),"</div>".repeat(depth));
        assert_eq!(compile(&input(&html,"div{box-sizing:content-box}")).unwrap_err()[0].code,"depth-limit");
    }
    for value in ["width:1000001px","padding:1000001px"] {
        assert!(compile(&input("<div id=p></div>",&format!("#p{{box-sizing:content-box;{value}}}"))).is_err());
    }
    assert!(compile(&input("<div id=p></div>","#p{box-sizing:content-box;width:1000000px;height:1000000px;padding:1000000px}")).is_ok());
}
