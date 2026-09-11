use nuxie_html_to_riv::{compile, CompileInput};
fn input(css: &str) -> CompileInput {
    CompileInput {html:"<div id=root><div id=a class=pick></div><section id=b></section><div id=c class=pick></div><div id=d></div></div>".into(),css:format!("#root>*{{width:20px;height:10px;background:red}}{css}"),width:390.0,height:200.0,..Default::default()}
}
fn equal(actual:&str,expected:&str) {
    assert!(compile(&input(actual)).unwrap().riv==compile(&input(expected)).unwrap().riv,"{actual}");
}
#[test]
fn matches_any_supports_lists_complex_arguments_and_nested_filters() {
    for selector in [":is(.pick,section)",":is(#root>.pick,.pick+section)",":is(:not(#d))",":is(:nth-child(-n+3))",":is(.missing,:where(.pick,section))"] {
        equal(&format!("#root>{selector}{{background:green}}"),"#root>#a,#root>#b,#root>#c{background:green}");
    }
    equal("#root>:nth-child(2 of :is(.pick,#d)){background:green}","#root>#c{background:green}");
    equal("#root>:not(:is(.pick,#d)){background:green}","#root>#b{background:green}");
    equal(r"#root>:I\53 (.pick){background:green}","#root>#a,#root>#c{background:green}");
}
#[test]
fn is_specificity_uses_maximum_including_unmatched_branches() {
    equal("#root>:is(.pick,#missing){background:green}#a,#c{background:red}","#root>#a,#root>#c{background:green}");
    equal("#root>:is(.pick){background:green}#root>.pick{background:red}","");
    equal("#root>:is(*){background:green}#root>*{background:red}","");
}
#[test]
fn where_has_zero_specificity_including_nested_ids() {
    equal("#root>.pick{background:green}#root>:where(#a,#c){background:red}","#root>#a,#root>#c{background:green}");
    equal("#root>.pick{background:green}#root>:where(:is(#a,#c)){background:red}","#root>#a,#root>#c{background:green}");
    equal("#root>:where(.pick,section){background:green}","#root>#a,#root>#b,#root>#c{background:green}");
}
#[test]
fn matches_any_rejects_unsupported_profile_syntax() {
    for selector in [":is(.pick,:hover)",":where(.pick,:has(div))",":is(.pick",":where(.pick",":is([data-x=a s])"] {
        assert!(compile(&input(&format!("{selector}{{width:10px}}"))).is_err(),"{selector}");
    }
}
#[test]
fn forgiving_lists_drop_invalid_branches_before_specificity() {
    for selector in [":is(.pick, > div)",":is(,.pick,)",":where(.pick, > div)",":is(.pick,:nth-child())"] {
        equal(&format!("#root>{selector}{{background:green}}"),"#root>#a,#root>#c{background:green}");
    }
    for selector in [":is()",":where()",":is(> div)",":where(,,)"] {
        equal(&format!("#root>{selector}{{background:green}}"),"");
    }
    equal("#root>:is(.pick,#missing > > div){background:green}#root>.pick{background:red}","");
    equal("#root>:not(:is()){background:green}","#root>*{background:green}");
}
#[test]
fn forgiving_lists_do_not_swallow_resource_limits() {
    let mut selector=".pick".to_string();
    for _ in 0..34 {selector=format!(":is({selector})");}
    assert!(compile(&input(&format!("{selector}{{width:10px}}"))).is_err());
    let selector=format!(":where({})",vec!["> div";1025].join(","));
    assert!(compile(&input(&format!("{selector}{{width:10px}}"))).is_err());
    assert!(compile(&input(":is(.pick,:nth-child(1000001)){width:10px}")).is_err());
}
