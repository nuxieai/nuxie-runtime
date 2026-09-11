use nuxie_html_to_riv::{compile, CompileInput, RuntimeCapability};

fn input(css: &str) -> CompileInput {
    CompileInput { html: "<div id=root><div id=a></div></div>".into(),
        css: css.into(), width:390., height:320., ..Default::default() }
}

#[test]
fn content_box_cascade_and_css_initial_values() {
    let base = compile(&input("#a{box-sizing:content-box}")).unwrap();
    for css in ["#a{box-sizing:initial}", "#a{box-sizing:unset}",
        "#a{--box:content-box;box-sizing:var(--box)}",
        "#a{box-sizing:content-box!important;box-sizing:border-box}"] {
        let other = compile(&input(css)).unwrap();
        assert_eq!(base.riv, other.riv, "{css}");
        assert_eq!(base.runtime_requirements, other.runtime_requirements, "{css}");
    }
    let inherited = compile(&input("#root{box-sizing:content-box}#a{box-sizing:inherit}")).unwrap();
    assert_eq!(inherited.runtime_requirements.layout_content_box.len(),2);
    let non_inherited = compile(&input("#root{box-sizing:content-box}")).unwrap();
    assert_eq!(non_inherited.runtime_requirements.layout_content_box.len(),1);
    let reset = compile(&input("#a{box-sizing:content-box;box-sizing:border-box}")).unwrap();
    assert!(reset.runtime_requirements.layout_content_box.is_empty());
    assert!(!reset.runtime_requirements.capabilities.contains(&RuntimeCapability::LayoutCssContentBoxV1));
}

#[test]
fn content_box_manifest_requires_capability_version_and_valid_targets() {
    let compiled = compile(&input("#a{box-sizing:content-box;width:50%;padding:10px}")).unwrap();
    let good = compiled.runtime_requirements;
    assert_eq!(good.version,13);
    assert_eq!(good.layout_content_box.len(),1);
    let caps = good.capabilities.iter().copied().collect::<Vec<_>>();
    good.ensure_supported(&caps).unwrap();
    assert_eq!(good.ensure_supported(&[]).unwrap_err().code,"missing-runtime-capability");
    for version in 1..13 {
        let mut bad=good.clone();bad.version=version;
        assert_eq!(bad.ensure_supported(&caps).unwrap_err().code,"invalid-layout-content-box");
    }
    let mut bad=good.clone();bad.layout_content_box.clear();
    assert!(bad.ensure_supported(&caps).is_err());
    let mut bad=good.clone();bad.capabilities.remove(&RuntimeCapability::LayoutCssContentBoxV1);
    assert!(bad.ensure_supported(&caps).is_err());
    let mut bad=good.clone();bad.layout_content_box.push(bad.layout_content_box[0]);
    assert!(bad.ensure_supported(&caps).is_err());
    assert_eq!(good.ensure_layout_targets(|_|false).unwrap_err().code,"invalid-layout-content-box-target");
}
