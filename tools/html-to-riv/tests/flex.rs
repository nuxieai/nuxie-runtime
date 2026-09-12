//! Only legacy computed flex is publicly admitted until candidate qualification.
use nuxie_html_to_riv::{compile, CompileInput};
fn input(css: &str) -> CompileInput { CompileInput { html:"<div id=p><div id=a></div><div id=b></div></div>".into(),css:format!("#p{{width:160px;height:120px;flex-direction:row}}{css}"),width:240.,height:160. } }
#[test]
fn explicit_legacy_values_preserve_bytes_and_source_maps() {
    let expected=compile(&input("")).unwrap();
    for css in ["div{flex:none}","div{flex:0 0 auto}","div{flex-grow:0;flex-shrink:0;flex-basis:auto}","#a{--f:none;flex:var(--f)}"] {assert_eq!(compile(&input(css)).unwrap(),expected,"{css}");}
}
#[test]
fn candidate_values_remain_diagnostic_in_public_api() {
    for css in ["#a{flex:1 7 0px}","#a{flex:1 1 30px}","#a{flex:1 1 auto}","#a{flex:initial}","#a{flex:1}","#a{flex-grow:1}","#a{flex-shrink:initial}","#a{flex:1 1 30px;flex:none}","#never{flex:1}","#a{--f:1 1 30px;flex:var(--f);flex:none}","#a{flex-grow:1;flex-grow:0}"] {assert_eq!(compile(&input(css)).unwrap_err()[0].code,"unsupported-target-semantics","{css}");}
}
#[test]
fn grammar_is_checked_even_for_losing_and_unmatched_declarations() {
    for css in ["#never{flex:-1}","#a{flex:1 / 2;flex:none}","#a{--f:-1;flex:var(--f);flex:none}"] {assert!(compile(&input(css)).is_err(),"{css}");}
}
