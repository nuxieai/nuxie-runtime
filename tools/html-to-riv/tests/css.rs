use nuxie_html_to_riv::Diagnostic;
use scraper::{Html, Selector};
#[path = "../src/css.rs"]
mod css;
fn values(stylesheet: &str, html: &str) -> Vec<(String, String)> {
    let rules = css::stylesheet(stylesheet).unwrap();
    let doc = Html::parse_fragment(html);
    let element = doc
        .select(&Selector::parse("#target").unwrap())
        .next()
        .unwrap();
    css::cascade(&rules, element)
        .unwrap()
        .into_iter()
        .map(|d| (d.name, d.value))
        .collect()
}
#[test]
fn specificity_important_inline_and_equal_source_order() {
    let actual = values(
        "div {color:red} .a {color:blue} #target {color:lime} .a {color:gold} div {color:purple!important}",
        "<div id=target class=a style='color:cyan'></div>",
    );
    assert_eq!(
        actual.iter().map(|(_, v)| v.as_str()).collect::<Vec<_>>(),
        ["red", "blue", "gold", "lime", "cyan", "purple"]
    );
    assert_eq!(
        values(
            "#target{color:red!important}",
            "<div id=target style='color:blue!important'></div>"
        )
        .last()
        .unwrap()
        .1,
        "blue"
    );
    // Preserve all shorthand/longhand declarations, rather than collapsing them
    // before caller computation can apply their own reset semantics.
    assert_eq!(
        values(
            "#target{margin:1px;margin-left:2px;margin:3px}",
            "<div id=target></div>"
        )
        .len(),
        3
    );
}
#[test]
fn selector_lists_attributes_combinators_and_function_specificity() {
    let actual = values(
        ".a{color:red}:where(#target){color:black}:is(.a,#absent){color:blue}[data-kind='Abc' i]{color:green} section > div + div{color:gold} div:nth-child(2 of .a){color:purple}",
        "<section><div class=a></div><div class=a id=target data-kind=abc></div></section>",
    );
    assert_eq!(
        actual.iter().map(|(_, v)| v.as_str()).collect::<Vec<_>>(),
        ["black", "gold", "red", "green", "purple", "blue"]
    );
    assert_eq!(
        values(
            "#absent, .a {color:red} div {color:blue}",
            "<div id=target class=a></div>"
        )
        .last()
        .unwrap()
        .1,
        "red"
    );
}
#[test]
fn declarations_keep_values_case_comments_and_precise_sources() {
    let ds = css::declarations(
        "COLOR: hsl(120 100% 50%); --Accent: rebeccapurple; width:var(--Accent)!IMPORTANT",
        "inline",
    )
    .unwrap();
    assert_eq!(ds[0].name, "color");
    assert_eq!(ds[1].name, "--Accent");
    assert_eq!(ds[2].value, "var(--Accent)");
    assert!(ds[2].important);
    assert!(ds.iter().all(|d| d.source.starts_with("inline:1:")));
    assert_eq!(
        css::ordinary_value("RGB(255 /* red */ 0 0 / .5)").unwrap(),
        "rgb(255   0 0 / 0.5)"
    );
    let doc = Html::parse_fragment("<div id=target style='color:red'></div>");
    let el = doc.select(&Selector::parse("div").unwrap()).next().unwrap();
    assert!(
        css::cascade(&[], el).unwrap()[0]
            .source
            .starts_with("html#target@style:1:")
    );
}
#[test]
fn caller_validation_sees_unmatched_unsupported_declarations() {
    let rules = css::stylesheet("#never{display:grid;filter:blur(2px)}").unwrap();
    assert_eq!(css::all_declarations(&rules).count(), 2);
    let err = css::validate_rules(&rules, |d| {
        if d.name == "display" && d.value == "grid" {
            Err(Diagnostic::new(
                "unsupported-css-value",
                &d.source,
                "Grid is unsupported",
            ))
        } else {
            Ok(())
        }
    })
    .unwrap_err();
    assert_eq!(err.code, "unsupported-css-value");
    assert!(err.source.starts_with("css:"));
    assert!(css::ordinary_value("blur(2px)").is_err());
}
#[test]
fn invalid_syntax_selectors_and_limits_are_not_silently_dropped() {
    for text in [
        "@media (width:20px){div{color:red}}",
        "div::before{color:red}",
        "div:hover{color:red}",
        "div{color red}",
        "div{color:red!important junk}",
        "div{color:}",
        "div{color:red",
        "div{color:rgb(255 0 0}",
        "div{--:red}",
        "div[foo='x' s]{color:red}",
    ] {
        assert!(css::stylesheet(text).is_err(), "{text}");
    }
    let too_many = format!("div{{{}}}", "color:red;".repeat(257));
    assert!(css::stylesheet(&too_many).is_err());
    let list = format!("{}{{color:red}}", vec!["div"; 1025].join(","));
    assert!(css::stylesheet(&list).is_err());
}
