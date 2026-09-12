use nuxie_html_to_riv::{compile, CompileInput};

fn input(html: &str, css: &str) -> CompileInput {
    CompileInput { html: html.into(), css: css.into(), width: 390., height: 160. }
}

#[test]
fn deterministic_complete_file_and_dom_identity_without_policies() {
    let input = input("<section id='outer'><div id='inner'></div></section>",
        "#outer{width:100%;height:100%;background-color:coral}#inner{width:50%;height:50%;background-color:rebeccapurple}");
    let first = compile(&input).unwrap();
    assert_eq!(first, compile(&input).unwrap());
    assert!(first.riv.starts_with(b"RIVE"));
    assert_eq!(first.source_map.iter().map(|s| (&*s.id,&*s.path)).collect::<Vec<_>>(), vec![("outer","/0"),("inner","/0/0")]);
    assert_ne!(first.source_map[0].object_id, first.source_map[1].object_id);
    let serialized = serde_json::to_value(first).unwrap();
    assert_eq!(serialized.as_object().unwrap().len(), 2);
    assert!(serialized.get("runtime_requirements").is_none());
}

#[test]
fn normalized_colors_and_cascade_generate_identical_artifacts() {
    let html = "<div id='box' class='paint'></div>";
    let a = compile(&input(html,"div{width:100%;height:100%;color:blue;background-color:currentColor}.paint{color:red}#box{color:rgb(102/**/ 51/**/ 153)!important}#box{color:red}")).unwrap();
    let b = compile(&input(html,"#box{width:100%;height:100%;color:rebeccapurple;background-color:rebeccapurple}")).unwrap();
    assert_eq!(a,b);
}

#[test]
fn unsupported_computed_contexts_do_not_publish() {
    for (html,css) in [
        ("<div id='box'></div>","#box{display:grid}"),
        ("<div id='box'></div>","#box{background-image:linear-gradient(red,blue)}"),
        ("<div id='box'></div>",".unmatched{padding:auto}"),
        ("<div style='width:calc(100% - 1px)'></div>",""),
        ("<div><div style='height:50%'></div></div>",""),
        ("<div>Text</div>",""),
        ("<img src='x'>",""),
        ("<div onclick='run()'></div>",""),
        ("<div></div>","body{background-color:red}"),
        ("<div id='x'></div><div id='x'></div>",""),
        ("root text<div></div>",""),
    ] {
        let errors = compile(&input(html,css)).unwrap_err();
        assert!(!errors[0].code.is_empty());
        assert!(!errors[0].message.is_empty(),"{html} {css}");
    }
}

#[test]
fn viewport_source_and_depth_limits_reject_before_emission() {
    for width in [0.,-1.,f32::NAN,f32::INFINITY,16385.] {
        let mut request=input("<div></div>","");request.width=width;
        assert_eq!(compile(&request).unwrap_err()[0].code,"invalid-viewport");
    }
    assert_eq!(compile(&input(&" ".repeat(1_048_577),"")).unwrap_err()[0].code,"input-limit");
    let nested=format!("{}{}","<div>".repeat(130),"</div>".repeat(130));
    assert_eq!(compile(&input(&nested,"")).unwrap_err()[0].code,"depth-limit");
}

#[test]
fn inherited_current_color_resolves_at_each_descendant_not_at_parent() {
    let html = "<section id='parent'><div id='child'><div id='grandchild'></div></div></section>";
    let geometry = "#parent{height:120px}#child{height:80px}#grandchild{height:40px}";
    let colors = "#parent{color:red}#child{color:blue}#grandchild{color:green}";
    let inherited = "#parent{background-color:currentColor}#child,#grandchild{background-color:inherit}";
    let explicit = "#parent{background-color:red}#child{background-color:blue}#grandchild{background-color:green}";
    let wrong_frozen_parent = "#parent,#child,#grandchild{background-color:red}";
    let actual = compile(&input(html,&format!("{geometry}{colors}{inherited}"))).unwrap();
    assert_eq!(actual,compile(&input(html,&format!("{geometry}{colors}{explicit}"))).unwrap());
    assert_ne!(actual.riv,compile(&input(html,&format!("{geometry}{colors}{wrong_frozen_parent}"))).unwrap().riv);
    // Literal backgrounds do inherit their resolved color, not the child's foreground.
    let literal_inherited = "#parent{background-color:red}#child,#grandchild{background-color:inherit}";
    assert_eq!(compile(&input(html,&format!("{geometry}{colors}{literal_inherited}"))).unwrap(),
        compile(&input(html,&format!("{geometry}{colors}{wrong_frozen_parent}"))).unwrap());
    // Explicit unset/initial break the inherited keyword chain on non-inherited properties.
    for reset in ["initial","unset"] {
        let reset_css = format!("{geometry}{colors}{inherited}#child{{background-color:{reset}}}");
        let transparent_css = format!("{geometry}{colors}#parent{{background-color:red}}#child,#grandchild{{background-color:transparent}}");
        assert_eq!(compile(&input(html,&reset_css)).unwrap(),compile(&input(html,&transparent_css)).unwrap());
    }
}

#[test]
fn top_level_background_inherits_the_explicit_host_reset() {
    let input = CompileInput { html: "<div id=box></div>".into(),
        css: "#box{width:100px;height:40px;background-color:inherit}".into(), width:240.,height:160. };
    let inherited = compile(&input).unwrap();
    let literal = compile(&CompileInput { css:input.css.replace("inherit", "white"), ..input }).unwrap();
    assert_eq!(inherited.riv, literal.riv);
    assert_eq!(serde_json::to_value(inherited.source_map).unwrap(), serde_json::to_value(literal.source_map).unwrap());
}
