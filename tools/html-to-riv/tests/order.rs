//! Compiler ordering semantics; browser/native qualification is separate.
use nuxie_html_to_riv::{compile, CompileInput, CompileOutput};
fn input(css:&str)->CompileInput { CompileInput{html:"<div id=p><div id=a></div><div id=b></div><div id=c></div><div id=d></div></div>".into(),css:css.into(),width:240.,height:160.} }
fn scene(css:&str)->CompileOutput {compile(&input(css)).unwrap()}
fn file_order(scene:&CompileOutput)->Vec<&str> {
    let mut nodes:Vec<_>=scene.source_map.iter().filter(|n|n.id!="p").collect();nodes.sort_by_key(|n|n.object_id);nodes.into_iter().map(|n|n.id.as_str()).collect()
}
#[test]
fn stable_order_then_direction_compensation_preserves_dom_identity() {
    for direction in ["row","column","row-reverse","column-reverse"] {
        let css=format!("#p{{flex-direction:{direction}}}#a,#c{{order:-1}}#b{{order:2}}");
        let actual=scene(&css);
        let expected=if direction.ends_with("reverse") {vec!["a","c","d","b"]} else {vec!["b","d","c","a"]};
        assert_eq!(file_order(&actual),expected);
        assert_eq!(actual.source_map.iter().map(|n|n.id.as_str()).collect::<Vec<_>>(),["p","a","b","c","d"]);
        assert_eq!(actual.source_map[3].path,"/0/2");
        assert_eq!(actual,scene(&css));
    }
}
#[test]
fn keywords_inherit_final_parent_value_but_unset_and_default_are_zero() {
    assert_eq!(scene("#p{order:4;order:-2}#a{order:inherit}"),scene("#a{order:-2}"));
    for keyword in ["initial","unset","0","-0","+0"] { assert_eq!(scene(&format!("#p{{order:5}}#a{{order:{keyword}}}")),scene("")); }
    assert_eq!(scene("#p{order:inherit}"),scene(""));
}
#[test]
fn exact_signed_bounds_comments_and_variables_keep_integer_precision() {
    let actual=scene("#p{flex-direction:row-reverse;--big:2147483647}#a{order:var(--big)}#b{order:2147483646}#c{order:/**/-2147483648/**/}#d{order:+0002}");
    assert_eq!(file_order(&actual),["c","d","b","a"]);
    let precise=scene("#p{flex-direction:row-reverse;--n:16777217}#a{order:var(--n)}#b{order:16777216}#c{order:16777218}#d{order:-2147483648}");
    assert_eq!(file_order(&precise),["d","b","a","c"]);
    assert_eq!(scene("#a{order:1!important;order:-1}#b{order:var(--missing,+2)}"),scene("#a{order:1}#b{order:2}"));
}
#[test]
fn selectors_still_use_original_dom_positions() {
    assert_eq!(scene("#a{order:4}#b{order:-1}#p>div:nth-child(1){background:red}#a+div{height:20px}"),
        scene("#a{order:4;background:red}#b{order:-1;height:20px}"));
}
#[test]
fn invalid_lexemes_and_overflow_reject_even_when_overridden_or_unmatched() {
    for value in ["2147483648","-2147483649","1.0","1e2",".5","1px","1%","+ 1","1 2","--1","auto","inherit 1"] {
        for css in [format!("#never{{order:{value}}}"),format!("#a{{order:{value};order:0}}"),format!("#a{{--o:{value};order:var(--o)}}")] {
            let errors=compile(&input(&css)).expect_err(&format!("unexpected success: {css}"));
            assert!(!errors[0].source.is_empty(),"{css}");
        }
    }
}
#[test]
fn sibling_precomputation_keeps_global_element_limit() {
    for html in ["<div></div>".repeat(8193), format!("<div>{}</div><div></div>", "<div></div>".repeat(8191))] {
        let errors=compile(&CompileInput{html,css:String::new(),width:240.,height:160.}).unwrap_err();
        assert_eq!(errors[0].code,"object-limit");
    }
}
#[test]
fn lightweight_sorting_does_not_bypass_losing_declarations_or_custom_environments() {
    for css in [
        "#a{--bad:1.5;order:var(--bad);order:0}",
        "#a{order:var(--missing);order:1}",
        "#a{--bad:revert;order:0}",
        "#a{--bad:revert}",
        "#a{width:var(--missing);order:0}",
    ] {
        assert!(compile(&input(css)).is_err(),"{css}");
    }
}
