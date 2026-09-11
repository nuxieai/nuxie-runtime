use nuxie_html_to_riv::{compile, CompileInput};
fn input(css: &str) -> CompileInput {
    CompileInput { html: "<div id=root><div id=a class=pick data-state=ready></div><!--gap--><section id=b></section><div id=c class=pick></div><div id=d></div></div>".into(), css: format!("#root>*{{width:20px;height:10px;background:red}}{css}"), width:390.0,height:200.0,..Default::default() }
}
fn matches(selector: &str, ids: &[&str]) {
    let expected=if ids.is_empty(){".missing".into()}else{ids.iter().map(|id|format!("#root>#{id}")).collect::<Vec<_>>().join(",")};
    assert!(compile(&input(&format!("#root>{selector}{{background:green}}"))).unwrap().riv == compile(&input(&format!("{expected}{{background:green}}"))).unwrap().riv,"{selector}");
}
#[test]
fn negation_matches_lists_complex_selectors_and_nested_combinations() {
    for (selector, ids) in [
        (":not(.pick)",vec!["b","d"]),
        (":not(.pick,#d)",vec!["b"]),
        (":not(*)",vec![]),
        (":not(.missing)",vec!["a","b","c","d"]),
        (":not(#root > .pick)",vec!["b","d"]),
        (":not(.pick + section)",vec!["a","c","d"]),
        (":not(:not(.pick))",vec!["a","c"]),
        (":not(:nth-child(odd))",vec!["b","d"]),
        (":nth-child(2 of :not(.pick))",vec!["d"]),
        (":not([data-state='ready'], section)",vec!["c","d"]),
        (r":N\4f T(.pick)",vec!["b","d"]),
    ] {matches(selector,&ids);}
}
#[test]
fn negation_specificity_is_only_maximum_argument_specificity() {
    for css in [
        "#root>:not(#missing,.pick){background:green}#b{background:red}#d{background:red}",
        "#root>:not(.pick){background:green}#root>.pick{background:red}",
    ] {
        assert!(compile(&input(css)).unwrap().riv==compile(&input("#root>#b,#root>#d{background:green}")).unwrap().riv);
    }
    // :not(*) contributes no specificity: the later universal rule wins.
    assert!(compile(&input("#root>:not(:not(*)){background:green}#root>*{background:red}")).unwrap().riv==compile(&input("")).unwrap().riv);
    // Equal argument specificity respects source order, without an extra pseudo unit.
    assert!(compile(&input("#root>:not(.missing){background:green}#root>.pick{background:red}")).unwrap().riv==compile(&input("#root>#b,#root>#d{background:green}")).unwrap().riv);
}
#[test]
fn negation_rejects_invalid_or_unsupported_branches_and_excessive_nesting() {
    for selector in [":not()",":not(.pick,)",":not(,.pick)",":not(>div)",":not(.pick,:hover)",":not(.pick,::before)",":not(.pick",":not(.pick,,div)",":not(:has(.pick))"] {
        assert!(compile(&input(&format!("{selector}{{background:green}}"))).is_err(),"{selector}");
    }
    let mut selector=".pick".to_string();
    for _ in 0..34 {selector=format!(":not({selector})");}
    let errors=compile(&input(&format!("{selector}{{background:green}}"))).unwrap_err();
    assert!(errors[0].message.len()<1024);
}
