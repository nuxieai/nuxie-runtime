//! Public selector lowering must equal independently enumerated target IDs.
use nuxie_html_to_riv::{CompileInput, compile};
fn equivalent(html: &str, selector: &str, expected: &[&str]) {
    let base = "#root{width:100px;height:100px}#root div{width:80px;height:10px;background:navy}";
    let css = format!("{base}{selector}{{background:coral}}");
    let direct = format!("{base}{}", expected.iter().map(|id| format!("#root #{id}{{background:coral}}")).collect::<String>());
    let input = |css| CompileInput {html: html.into(),css,width:240.,height:160.};
    assert_eq!(compile(&input(css)).unwrap(), compile(&input(direct)).unwrap(), "{selector}");
}
const TREE: &str = "<section id=root><div id=a class='one tag'></div><!-- gap --><div id=b class='two'></div><div id=c class='one tag-more'><div id=e class=one></div></div><div id=d class=ONE></div></section>";
#[test]
fn public_attribute_operator_and_case_matching() {
    for (selector, ids) in [
        ("#root [class]", vec!["a","b","c","d","e"]),
        ("#root [class='two']", vec!["b"]),
        ("#root [class~='one']", vec!["a","c","e"]),
        ("#root [id|='a']", vec!["a"]),
        ("#root [class^='one']", vec!["a","c","e"]),
        ("#root [class$='more']", vec!["c"]),
        ("#root [class*='tag']", vec!["a","c"]),
        ("#root [class='one' i]", vec!["d","e"]),
        ("#root [class^='']", vec![]),
    ] { equivalent(TREE, selector, &ids); }
}
#[test]
fn public_siblings_ignore_comments_but_respect_parentage() {
    equivalent(TREE,"#root > #a + div", &["b"]);
    equivalent(TREE,"#root > #a ~ div", &["b","c","d"]);
    equivalent(TREE,"#root #b + div > div", &["e"]);
}
#[test]
fn public_child_positions_and_filtered_counting() {
    for (selector,ids) in [
        ("#root > div:first-child",vec!["a"]),
        ("#root > div:last-child",vec!["d"]),
        ("#root div:only-child",vec!["e"]),
        ("#root > div:nth-child(2n+1)",vec!["a","c"]),
        ("#root > div:nth-child(-n+2)",vec!["a","b"]),
        ("#root > div:nth-last-child(2)",vec!["c"]),
        ("#root > div:nth-child(2 of .one)",vec!["c"]),
    ] { equivalent(TREE,selector,&ids); }
}
#[test]
fn public_logical_selectors_and_nested_combinations() {
    equivalent(TREE,"#root > div:not(.one)", &["b","d"]);
    equivalent(TREE,"#root > :is(.two,#d)", &["b","d"]);
    // :where contributes no specificity and loses to the base #root div rule.
    equivalent(TREE,"#root > :where(.two,#d)", &[]);
    equivalent(TREE,"#root > div:where(.two,#d)", &["b","d"]);
    equivalent(TREE,"#root > div:not(:is(.one,.two))", &["d"]);
}
