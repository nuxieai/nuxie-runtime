use nuxie_html_to_riv::{CompileInput, compile};
fn bytes(html: &str, css: &str) -> Vec<u8> {
    compile(&CompileInput {html:html.into(),css:css.into(),width:390.0,height:200.0,..Default::default()}).unwrap().riv
}
#[test]
fn adjacent_siblings_follow_dom_elements_not_visible_boxes() {
    for (middle, matches) in [("",true),(" \n<!-- comment -->\t",true),("<div class=hidden></div>",false),("<section></section>",false)] {
        let html=format!("<div><div id=a></div>{middle}<div id=b></div><div id=c></div></div>");
        let base="div,section{width:32px;height:20px;background:red}.hidden{display:none}";
        assert_eq!(bytes(&html,&format!("{base}#a+#b{{background:green}}")),bytes(&html,&format!("{base}#b{{background:{}}}",if matches{"green"}else{"red"})),"{middle}");
    }
    let html="<div><div id=a></div><section><div id=b></div></section></div>";
    assert_eq!(bytes(html,"#a+#b{width:100px}"),bytes(html,""));
}
#[test]
fn adjacent_chains_attributes_and_specificity_compose() {
    let html="<div data-zone=main><div class=start></div><div id=b data-kind=card></div><div id=c></div></div>";
    let base="div{width:32px;height:20px;background:red}";
    for (css,target) in [
        (".start + [data-kind=card]{background:green}","b"),
        ("[data-zone] > .start+[data-kind]{background:green}","b"),
        (".start + div + div{background:green}","c"),
        (".start/**/+/**/div{background:green}","b"),
        (".start+div{background:green}div{background:red}","b"),
        ("#b{background:green}.start+div{background:red}","b"),
        (".start+div{background:green!important}#b{background:red}","b"),
    ] { assert_eq!(bytes(html,&format!("{base}{css}")),bytes(html,&format!("{base}#{target}{{background:green}}")),"{css}"); }
}
#[test]
fn invalid_sibling_syntax_is_rejected() {
    for selector in ["+div","div+","div++div","div+>div","~div","div~","div~~div","div~+div","div + :hover"] {
        let result=compile(&CompileInput {html:"<div></div>".into(),css:format!("{selector}{{width:10px}}"),width:390.0,height:200.0,..Default::default()});
        assert!(result.is_err(),"{selector}");
    }
}

#[test]
fn general_siblings_match_only_later_elements_in_the_same_parent() {
    let html="<div><div id=before></div><div id=a class=hidden></div> \n<!-- comment --><section></section><div id=b></div><div id=c></div><section><div id=nested></div></section></div>";
    let base="div,section{width:32px;height:20px;background:red}.hidden{display:none}";
    assert_eq!(bytes(html,&format!("{base}#a~div{{background:green}}")),bytes(html,&format!("{base}#b,#c{{background:green}}")));
    for selector in ["#a~#a", "#a~#before", "#a~#nested"] {
        assert_eq!(bytes(html,&format!("{base}{selector}{{background:green}}")),bytes(html,base),"{selector}");
    }
}
#[test]
fn general_sibling_chains_preserve_specificity_and_source_order() {
    let html="<div data-zone=main><div class=start></div><section></section><div id=b data-kind=card></div><div id=c></div></div>";
    let base="div,section{width:32px;height:20px;background:red}";
    for (css,target) in [
        (".start~[data-kind=card]{background:green}","b"),
        ("[data-zone]>.start~[data-kind]{background:green}","b"),
        (".start~[data-kind]+div{background:green}","c"),
        (".start~div~div{background:green}","c"),
        (".start/**/~/**/[data-kind]{background:green}","b"),
        (".start~[data-kind]{background:green}div{background:red}","b"),
        ("#b{background:green}.start~[data-kind]{background:red}","b"),
        (".start~[data-kind]{background:red}.start~[data-kind]{background:green}","b"),
        (".start~[data-kind]{background:green!important}#b{background:red}","b"),
    ] {assert_eq!(bytes(html,&format!("{base}{css}")),bytes(html,&format!("{base}#{target}{{background:green}}")),"{css}");}
}
