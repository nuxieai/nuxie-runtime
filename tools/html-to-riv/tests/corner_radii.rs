use nuxie_html_to_riv::{compile, CompileInput, CompileOutput};

fn scene(css: &str) -> CompileOutput {
    compile(&CompileInput { html: "<div id=\"a\"></div>".into(),
        css: format!("#a{{width:100px;height:80px;background:red;{css}}}"),
        width:390., height:320., ..Default::default() }).unwrap()
}

fn same(a: &str, b: &str) {
    let a = scene(a);
    let b = scene(b);
    assert_eq!(a.riv, b.riv);
    assert_eq!(a.runtime_requirements, b.runtime_requirements);
}

#[test]
fn physical_corner_longhands_and_shorthand_resets_are_equivalent() {
    same("border-radius:4px 8px 12px 16px;",
         "border-top-left-radius:4px;border-top-right-radius:8px;border-bottom-right-radius:12px;border-bottom-left-radius:16px;");
    same("border-radius:4px 8px 12px 16px;border-bottom-left-radius:2px;",
         "border-radius:4px 8px 12px 2px;");
    same("border-top-left-radius:40px;border-radius:8px;", "border-radius:8px 8px 8px 8px;");
    for keyword in ["initial", "unset"] {
        same(&format!("border-radius:4px 8px 12px 16px;border-top-right-radius:{keyword};"),
             "border-radius:4px 0 12px 16px;");
    }
}

#[test]
fn corner_variable_invalidation_resets_only_the_selected_corner() {
    for invalid in ["-1px", "1px 2px 3px", "red"] {
        same(&format!("--r:{invalid};border-radius:4px 8px 12px 16px;border-top-left-radius:var(--r);"),
             "border-radius:0 8px 12px 16px;");
    }
    same("--r:2em;border-top-left-radius:var(--r);font-size:20px;", "border-radius:40px 0 0 0;");
    same("border-top-left-radius:var(--missing, 12px);", "border-radius:12px 0 0 0;");
}

#[test]
fn corner_math_remains_an_explicit_rejection() {
    for value in ["calc(2px + 8px)"] {
        for css in [format!("border-top-left-radius:{value};"),
                    format!("--r:{value};border-top-left-radius:var(--r);")] {
            assert!(compile(&CompileInput { html:"<div></div>".into(),css:format!("div{{{css}}}"),
                width:390., height:320., ..Default::default() }).is_err(), "{css}");
        }
    }
}

#[test]
fn every_physical_corner_resets_and_resolves_independently() {
    let properties = ["border-top-left-radius", "border-top-right-radius",
        "border-bottom-right-radius", "border-bottom-left-radius"];
    for (index, property) in properties.iter().enumerate() {
        for (declaration, expected) in [
            ("initial", 0.), ("unset", 0.),
            ("var(--missing)", 0.), ("var(--bad, 9px)", 0.),
            ("var(--missing, .5em)", 10.), (".25rem", 4.),
            ("0.125px", 0.125),
        ] {
            let mut values = [4., 8., 12., 16.];
            values[index] = expected;
            same(&format!("font-size:20px;--bad:-1px;border-radius:4px 8px 12px 16px;{property}:{declaration};"),
                 &format!("font-size:20px;border-radius:{}px {}px {}px {}px;",values[0],values[1],values[2],values[3]));
        }
    }
}

#[test]
fn inherited_physical_corner_uses_matching_parent_corner() {
    let properties = ["border-top-left-radius", "border-top-right-radius",
        "border-bottom-right-radius", "border-bottom-left-radius"];
    for (index, property) in properties.iter().enumerate() {
        let build = |value: &str| compile(&CompileInput {
            html: "<div id=\"parent\"><div id=\"child\"></div></div>".into(),
            css: format!("#parent{{width:100px;height:100px;border-radius:3px 7px 11px 19px;}}#child{{width:50px;height:50px;background:red;border-radius:1px;{property}:{value};}}"),
            width:390.,height:320.,..Default::default()
        }).unwrap();
        let inherited = build("inherit");
        let explicit = build(["3px", "7px", "11px", "19px"][index]);
        assert_eq!(inherited.riv, explicit.riv, "{property}");
        assert_eq!(inherited.runtime_requirements, explicit.runtime_requirements);
    }
}
