use nuxie_html_to_riv::{compile, CompileInput, CompileOutput};
fn input(css: &str) -> CompileInput {
    CompileInput { html: "<div id=p><div id=a></div><div id=b></div></div>".into(), css: format!("#p{{width:160px;height:100px;flex-direction:row}}#a,#b{{width:30px;height:20px;background:red}}{css}"), width: 240., height: 160. }
}
fn scene(css: &str) -> CompileOutput { compile(&input(css)).unwrap() }
#[test]
fn shorthand_expansion_and_later_longhands_follow_css_order() {
    for (shorthand, longhands) in [
        ("auto", "margin-top:auto;margin-right:auto;margin-bottom:auto;margin-left:auto"),
        ("auto 0", "margin-top:auto;margin-bottom:auto"),
        ("0 auto 0", "margin-left:auto;margin-right:auto"),
        ("auto 0 0 auto", "margin-top:auto;margin-left:auto"),
    ] {
        assert_eq!(scene(&format!("#b{{margin:{shorthand}}}")), scene(&format!("#b{{{longhands}}}")));
    }
    assert_eq!(scene("#b{margin:auto;margin-left:0;margin-bottom:0}"), scene("#b{margin-top:auto;margin-right:auto}"));
    assert_eq!(scene("#b{margin-left:0!important;margin:auto}"), scene("#b{margin-top:auto;margin-right:auto;margin-bottom:auto}"));
}
#[test]
fn zero_and_css_wide_resets_preserve_existing_output() {
    for value in ["0", "0px", "0em", "0rem", "initial", "unset"] {
        assert_eq!(scene(""), scene(&format!("#b{{margin:{value}}}")));
        assert_eq!(scene(""), scene(&format!("#b{{margin:auto;margin:{value}}}")));
    }
    assert_eq!(scene("#p{margin:auto}#b{margin:inherit}"), scene("#p,#b{margin:auto}"));
    assert_eq!(scene("#p{margin-left:auto}#b{margin-left:inherit}"), scene("#p,#b{margin-left:auto}"));
}
#[test]
fn variables_selectors_order_and_source_maps_use_the_authored_dom() {
    let a=scene("#p{--m:auto}#a{order:2}#p>div:nth-child(2){margin-left:var(--m)}#a+div{align-self:center}");
    let b=scene("#a{order:2}#b{margin-left:auto;align-self:center}");
    assert_eq!(a,b);
    assert_eq!(a.source_map.iter().map(|n| (n.id.as_str(), n.path.as_str())).collect::<Vec<_>>(), [("p","/0"),("a","/0/0"),("b","/0/1")]);
}
#[test]
fn cross_auto_margins_override_alignment_including_baseline() {
    for direction in ["row", "row-reverse", "column", "column-reverse"] {
        let side=if direction.starts_with("row") {"top"} else {"left"};
        let base=format!("#p{{flex-direction:{direction}}}#b{{margin-{side}:auto}}");
        for alignment in ["center","flex-end","safe center","baseline","last baseline","normal","stretch"] {
            assert_eq!(scene(&base),scene(&format!("{base}#b{{align-self:{alignment}}}")));
        }
    }
}
#[test]
fn unresolved_margin_contexts_diagnose_without_disabling_unrelated_scopes() {
    for css in ["#p{justify-content:space-around}#b{margin-left:auto}", "#b{align-self:baseline;margin-left:auto}"] {
        assert_eq!(compile(&input(css)).unwrap_err()[0].code,"unsupported-target-semantics");
    }
    assert!(compile(&input("#p{justify-content:space-evenly}#b{margin-top:auto}")).is_ok());
    let mut nested=input("#b{align-self:baseline}#leaf{height:10px;margin-top:auto}");
    nested.html="<div id=p><div id=a></div><div id=b><div id=leaf></div></div></div>".into();
    assert_eq!(compile(&nested).unwrap_err()[0].code,"unsupported-target-semantics");
    nested.css.push_str("#b{align-self:normal}");
    assert!(compile(&nested).is_ok());
}
#[test]
fn invalid_or_unqualified_literals_reject_even_when_unmatched_or_overridden() {
    for value in ["1px","-1px","10%","auto auto auto auto auto","inherit auto","none"] {
        for css in [format!("#never{{margin:{value}}}"),format!("#b{{margin:{value};margin:0}}"),format!("#b{{--m:{value};margin:var(--m)}}")] {
            assert!(compile(&input(&css)).is_err(),"{css}");
        }
    }
}
#[test]
fn helper_expansion_preserves_the_authored_element_limit() {
    let mut request=input("div{margin:auto}");
    request.html="<div></div>".repeat(8192);
    let result=compile(&request).unwrap();
    assert_eq!(result.source_map.len(),8192);
    request.html.push_str("<div></div>");
    assert_eq!(compile(&request).unwrap_err()[0].code,"object-limit");
}
