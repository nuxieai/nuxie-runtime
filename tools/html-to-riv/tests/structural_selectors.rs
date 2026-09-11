use nuxie_html_to_riv::{CompileInput, compile};
fn bytes(html: &str, css: &str) -> Vec<u8> {
    compile(&CompileInput {html:html.into(),css:css.into(),width:390.0,height:200.0,..Default::default()}).unwrap().riv
}
#[test]
fn child_pseudo_classes_count_source_elements_including_hidden_ones() {
    let html="<div id=root> <!--before--> <section id=a></section> \n<div id=b></div><!--after--><div id=c></div></div>";
    let base="div,section{width:32px;height:20px;background:red}";
    for (selector, targets) in [("#root > :first-child","#a"),("#root > :last-child","#c"),("#root > :only-child",".missing"),("#root > div:first-child",".missing"),("#root > :FIRST-CHILD","#a"),(r"#root > :first-\63 hild","#a")] {
        assert_eq!(bytes(html,&format!("{base}{selector}{{background:green}}")),bytes(html,&format!("{base}{targets}{{background:green}}")),"{selector}");
    }
    for hidden in ["#a{display:none}","#c{display:none}"] {
        assert_eq!(bytes(html,&format!("{base}{hidden}#b:first-child,#b:last-child,#b:only-child{{background:green}}")),bytes(html,&format!("{base}{hidden}")));
    }
    let single="<div id=root><!-- before --> <div id=only></div> <!-- after --></div>";
    for selector in ["#root > :only-child", "#root > :first-child:last-child", "#only:only-child"] {
        assert_eq!(bytes(single,&format!("{base}{selector}{{background:green}}")),bytes(single,&format!("{base}#only{{background:green}}")));
    }
}
#[test]
fn structural_pseudo_specificity_and_sibling_composition_are_correct() {
    let html="<div id=root><div id=a class=tile data-kind=card></div><div id=b class=tile></div><div id=c class=tile></div></div>";
    let base="div{width:32px;height:20px;background:red}";
    for (css,target) in [
        (".tile:first-child{background:green}.tile{background:red}","a"),
        (".tile:first-child{background:red}.tile:first-child{background:green}","a"),
        ("#a{background:green}.tile:first-child{background:red}","a"),
        (".tile:last-child{background:green!important}#c{background:red}","c"),
        ("[data-kind]:first-child+div{background:green}","b"),
        ("[data-kind]:first-child~div:last-child{background:green}","c"),
    ] {assert_eq!(bytes(html,&format!("{base}{css}")),bytes(html,&format!("{base}#{target}{{background:green}}")),"{css}");}
}
#[test]
fn malformed_and_other_pseudo_classes_stay_rejected() {
    for selector in ["div:","div::first-child","div: first-child","div:first-child()","div:first-child(1)","div:first-of-type","div:hover","div:only-child(1)"] {
        assert!(compile(&CompileInput {html:"<div></div>".into(),css:format!("{selector}{{width:10px}}"),width:390.0,height:200.0,..Default::default()}).is_err(),"{selector}");
    }
}
