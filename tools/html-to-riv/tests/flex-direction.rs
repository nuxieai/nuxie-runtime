//! Public direction/cascade tests. Native geometry and pixels are a separate gate.
use nuxie_html_to_riv::{compile, CompileInput, CompileOutput};
fn compiled(css: &str) -> CompileOutput {
    compile(&CompileInput { html: "<section id=p><div id=c><div id=g></div></div><div id=s></div></section>".into(), css: css.into(), width: 240., height: 160. }).unwrap()
}
fn equivalent(actual: &str, expected: &str) { assert_eq!(compiled(actual), compiled(expected)); }
#[test]
fn four_directions_are_distinct_without_reordering_source_identity() {
    let scenes: Vec<_> = ["column", "column-reverse", "row", "row-reverse"].iter()
        .map(|d| compiled(&format!("#p{{flex-direction:{d}}}#c,#s{{width:20px;height:30px}}"))).collect();
    for (i,a) in scenes.iter().enumerate() { for b in &scenes[i+1..] {
        assert_ne!(a.riv,b.riv);
        assert_eq!(a.source_map.iter().map(|n| (&n.id, &n.path)).collect::<Vec<_>>(),
            b.source_map.iter().map(|n| (&n.id, &n.path)).collect::<Vec<_>>());
    }}
}
#[test]
fn inheritance_copies_final_parent_direction_and_default_does_not_inherit() {
    equivalent("#p{flex-direction:row;flex-direction:column-reverse}#c,#g{flex-direction:inherit}",
        "#p,#c,#g{flex-direction:column-reverse}");
    equivalent("#p{flex-direction:row-reverse}#g{flex-direction:inherit}",
        "#p{flex-direction:row-reverse}#c,#g{flex-direction:column}");
    equivalent("#p{flex-direction:inherit}", "#p{flex-direction:column}");
    for keyword in ["initial", "unset", "InItIaL"] {
        equivalent(&format!("#p{{flex-direction:column-reverse}}#c{{flex-direction:{keyword}}}#g{{flex-direction:inherit}}"),
            "#p{flex-direction:column-reverse}#c,#g{flex-direction:row}");
    }
}
#[test]
fn cascade_variables_and_font_relative_bounds_compose() {
    equivalent("#p{--axis:row-reverse;flex-direction:var(--axis);font-size:20px}#c{flex-direction:inherit!important;flex-direction:column;width:2em;min-width:1em;max-height:3rem}",
        "#p{flex-direction:row-reverse}#c{flex-direction:row-reverse;width:40px;min-width:20px;max-height:48px}");
    equivalent("#p{flex-direction:var(--missing, row)}#c{flex-direction:var(--missing, unset)}", "#p,#c{flex-direction:row}");
}
#[test]
fn auto_is_preserved_for_native_intrinsic_measurement() {
    equivalent("#p{flex-direction:row}#c{width:initial;height:unset}#g{width:60px;height:40px}",
        "#p{flex-direction:row}#c{width:auto;height:auto}#g{width:60px;height:40px}");
    assert_ne!(compiled("#p{flex-direction:row}#c{width:auto}").riv,
        compiled("#p{flex-direction:row}#c{width:100%}").riv);
    assert_ne!(compiled("#p{flex-direction:column}#c{width:auto}").riv,
        compiled("#p{flex-direction:column}#c{width:100%}").riv);
}
#[test]
fn invalid_directions_are_diagnosed_even_when_unmatched_or_overridden() {
    for css in ["#never{flex-direction:diagonal}", "#p{flex-direction:row column}",
        "#p{flex-direction:revert}", "#p{flex-direction:row-reverse;flex-direction:inherit row}"] {
        let errors=compile(&CompileInput{html:"<div id=p></div>".into(),css:css.into(),width:240.,height:160.}).unwrap_err();
        assert_eq!(errors[0].code,"unsupported-target-semantics", "{css}");
        assert!(!errors[0].source.is_empty());
    }
    equivalent("#p{--axis:diagonal;flex-direction:var(--axis)}",
        "#p{--axis:diagonal;flex-direction:unset}");
}
#[test]
fn paint_order_lowering_preserves_numeric_dom_preorder_and_selectors() {
    let html = (0..12).map(|i| format!("<div id=n{i}><div id=c{i}></div></div>")).collect::<String>();
    let input = |css: &str| CompileInput { html:html.clone(),css:css.into(),width:240.,height:160. };
    let actual=compile(&input("body>div:nth-child(3){background:red}body>div:last-child{height:10px}")).unwrap();
    let expected=compile(&input("#n2{background:red}#n11{height:10px}")).unwrap();
    assert_eq!(actual, expected);
    let identities:Vec<_>=actual.source_map.iter().map(|n|n.id.as_str()).collect();
    let expected_ids=(0..12).flat_map(|i|[format!("n{i}"),format!("c{i}")]).collect::<Vec<_>>();
    assert_eq!(identities,expected_ids);
    assert_eq!(actual.source_map[20].path,"/10");
    assert_eq!(actual.source_map[21].path,"/10/0");
    assert!(actual.source_map[0].object_id > actual.source_map[22].object_id,
        "public DOM order is independent of native paint object order");
}
#[test]
fn reverse_parents_keep_file_order_while_public_order_remains_dom_order() {
    let normal=compiled("#p{flex-direction:row}");
    let reversed=compiled("#p{flex-direction:row-reverse}");
    let object_id=|scene:&CompileOutput,id:&str| scene.source_map.iter().find(|n|n.id==id).unwrap().object_id;
    assert!(object_id(&normal,"c") > object_id(&normal,"s"));
    assert!(object_id(&reversed,"c") < object_id(&reversed,"s"));
    assert_eq!(normal.source_map.iter().map(|n|&n.id).collect::<Vec<_>>(),
        reversed.source_map.iter().map(|n|&n.id).collect::<Vec<_>>());
    equivalent("#p{flex-direction:row-reverse}#c{flex-direction:inherit}",
        "#p,#c{flex-direction:row-reverse}");
}
