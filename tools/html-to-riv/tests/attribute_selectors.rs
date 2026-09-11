use nuxie_html_to_riv::{CompileInput, compile};

fn bytes(html: &str, css: &str) -> Vec<u8> {
    compile(&CompileInput { html: html.into(), css: css.into(), width:390.0,height:200.0,..Default::default() }).unwrap().riv
}

#[test]
fn attribute_operators_flags_and_escaping_match_through_public_compiler() {
    let html = r#"<div id=root class="tag other" lang="en-US" data-label="Alpha,beta words" data-empty="" data-symbol="a]b" data-unicode="É"></div>"#;
    for (selector, matches) in [
        ("[data-label]", true), ("[data-missing]", false),
        ("[data-empty='']", true), ("[data-empty^='']", false),
        ("[data-label='Alpha,beta words']", true),
        ("[data-label='alpha,beta words']", false),
        ("[data-label='alpha,beta words' i]", true),
        ("[class~=other]", true), ("[class~=oth]", false),
        ("[lang|=en]", true), ("[lang|=e]", false),
        ("[data-label^=Alpha]", true), ("[data-label$=words]", true),
        ("[data-label*=beta]", true), ("[data-label*=BETA i]", true),
        ("[data-label*=missing]", false), ("[data-symbol='a]b']", true),
        (r#"[data-\6c abel="\41 lpha,beta words"]"#, true),
        ("[DATA-LABEL]", true), ("[data-unicode='é' i]", false),
        ("[data-missing], [data-label='Alpha,beta words']", true),
        ("div/**/[data-label]", true),
    ] {
        let base = "div{width:32px;height:20px;background:red}";
        let actual = bytes(html, &format!("{base}{selector}{{background:green}}"));
        let expected = bytes(html, &format!("{base}#root{{background:{}}}", if matches { "green" } else { "red" }));
        assert_eq!(actual, expected, "{selector}");
    }
}

#[test]
fn attribute_specificity_and_combinators_obey_cascade() {
    let html = "<div data-zone=main><div id=root class=tag data-role=card></div></div>";
    for css in [
        "[data-role]{background:green}.tag{background:red}[data-role]{background:green}",
        "[data-zone] > [data-role]{background:green}.tag{background:red}",
        "[data-zone] [data-role]{background:green}div{background:red}",
        "#root{background:green}[data-zone] [data-role]{background:red}",
        "[data-role]{background:green!important}#root{background:red}",
    ] {
        let base = "div{width:40px;height:30px}div[data-zone]{background:red}";
        assert_eq!(bytes(html,&format!("{base}{css}")), bytes(html,&format!("{base}#root{{background:green}}")), "{css}");
    }
}

#[test]
fn malformed_and_out_of_scope_selectors_are_rejected_even_unmatched() {
    for selector in ["[data-x", "[data-x=]", "[data-x=one two]", "[data-x=one z]", "[data-x=one s]", "[data-x i]", "[data-x!=one]", "[ns|data-x]", "[*|data-x]", "[data-x=12]", "[data-x='abc]", "[data-x],", ", [data-x]", "div:hover", ":has(div)"] {
        let result = compile(&CompileInput {html:"<div></div>".into(),css:format!("{selector}{{color:red}}"),width:390.0,height:200.0,..Default::default()});
        assert!(result.is_err(), "{selector} was silently accepted");
    }
    for attribute in ["onclick=go", "data-=bad", "unknown=value"] {
        assert!(compile(&CompileInput {html:format!("<div {attribute}></div>"),width:390.0,height:200.0,..Default::default()}).is_err());
    }
}
