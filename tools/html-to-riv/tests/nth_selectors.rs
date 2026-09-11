use nuxie_html_to_riv::{CompileInput, compile};
fn input(html: &str, css: &str) -> CompileInput { CompileInput {html:html.into(),css:css.into(),width:390.0,height:200.0,..Default::default()} }
fn check(selector: &str, expected: &[usize]) {
    let html=format!("<div id=root>{}</div>",(1..=7).map(|i|format!("<{} id=n{i} class=\"{}\"></{}>",if i==2{"section"}else{"div"},if i%2==1{"pick"}else{"skip"},if i==2{"section"}else{"div"})).collect::<String>());
    let base="#root>div,#root>section{width:20px;height:10px;background:red}";
    let actual=compile(&input(&html,&format!("{base}#root>{selector}{{background:green}}"))).unwrap();
    let explicit=if expected.is_empty(){".missing".into()}else{expected.iter().map(|i|format!("#root>#n{i}")).collect::<Vec<_>>().join(",")};
    let expected=compile(&input(&html,&format!("{base}{explicit}{{background:green}}"))).unwrap();
    assert!(actual.riv == expected.riv,"{selector}");
}
#[test]
fn nth_formulas_count_all_elements_in_both_directions() {
    for (formula, expected) in [("odd",vec![1,3,5,7]),("even",vec![2,4,6]),("2n+1",vec![1,3,5,7]),("2n - 1",vec![1,3,5,7]),("-n+3",vec![1,2,3]),("n+5",vec![5,6,7]),("3n",vec![3,6]),("+n",vec![1,2,3,4,5,6,7]),("0n+2",vec![2]),("0",vec![]),("-2",vec![]),("-n",vec![]),("3",vec![3]),("n-2",vec![1,2,3,4,5,6,7])] {
        check(&format!(":nth-child({formula})"),&expected);
        let reversed:Vec<_>=expected.iter().rev().map(|i|8-i).collect();
        check(&format!(":nth-last-child({formula})"),&reversed);
    }
}
#[test]
fn nth_filtered_lists_use_filtered_indices_and_nested_selectors() {
    check(":nth-child(2 of .pick)",&[3]);
    check(":nth-last-child(2 of .pick)",&[5]);
    check(":nth-child(2n of .pick)",&[3,7]);
    check(":nth-child(2 of .pick, #n2)",&[2]);
    check(":nth-child(1 of #root > section)",&[2]);
    check(":nth-child(2 of :nth-child(odd))",&[3]);
    check(":nth-child(2 of .pick + .skip)",&[4]);
    check(":nth-child(1 of .missing)",&[]);
}
#[test]
fn nth_of_specificity_uses_maximum_even_for_unmatched_filter_branches() {
    let html="<div><div id=target class=pick></div></div>";
    let base="div{width:20px;height:10px}";
    for css in [
        "#target{background:red}.pick:nth-child(1 of .pick,#missing){background:green}",
        ".pick:nth-child(1 of .pick,#missing){background:green}#target{background:red}",
        ".pick:nth-child(1){background:green}.pick{background:red}",
    ] {assert_eq!(compile(&input(html,&format!("{base}{css}"))).unwrap().riv,compile(&input(html,&format!("{base}#target{{background:green}}"))).unwrap().riv,"{css}");}
}
#[test]
fn malformed_formulas_and_unsupported_filters_are_rejected() {
    for selector in [":nth-child()",":nth-child(2n+)",":nth-child(2 n)",":nth-child(1.5)",":nth-child(odd junk)",":nth-child(2 of)",":nth-child(2 of .x,)",":nth-child(2 of :hover)",":nth-child(2",":nth-last-child(1 2)",":nth-of-type(2)",":nth-child(2147483648n-2147483647)",":nth-child(1000001)"] {
        assert!(compile(&input("<div></div>",&format!("{selector}{{width:10px}}"))).is_err(),"{selector}");
    }
    let mut selector="div".to_owned();
    for _ in 0..34 {selector=format!(":nth-child(1 of {selector})");}
    let errors = compile(&input("<div></div>",&format!("{selector}{{width:10px}}"))).unwrap_err();
    assert!(errors[0].message.len() < 1024, "Nested errors must not expand exponentially");
}
