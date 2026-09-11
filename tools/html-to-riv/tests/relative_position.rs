use nuxie_html_to_riv::{compile, CompileInput};
fn input(css:&str)->CompileInput {CompileInput {html:"<div id=root><div id=a></div></div>".into(),css:css.into(),width:390.,height:320.,..Default::default()}}
#[test]
fn relative_insets_compile_with_shorthand_units_and_cascade() {
    for (a,b) in [
        ("inset:5px 10% auto -3px","top:5px;right:10%;bottom:auto;left:-3px"),
        ("inset:5px 10px 15px","top:5px;right:10px;bottom:15px;left:10px"),
        ("font-size:20px;left:-1em;top:1rem","font-size:20px;left:-20px;top:16px"),
        ("--i:-5px 10%;inset:var(--i)","inset:-5px 10%"),
        ("left:5px;inset:initial","inset:auto"),
        ("inset:8px;left:unset","top:8px;right:8px;bottom:8px;left:auto"),
        ("--bad:red;left:var(--bad)","left:auto"),
        ("--bad:banana;position:var(--bad)","position:static"),
    ] {
        let a=compile(&input(&format!("#a{{position:relative;{a}}}"))).unwrap();
        let b=compile(&input(&format!("#a{{position:relative;{b}}}"))).unwrap();
        assert_eq!(a.riv,b.riv);assert_eq!(a.runtime_requirements,b.runtime_requirements);
    }
}
#[test]
fn static_offsets_are_ignored_and_relative_position_is_not_inherited() {
    let base=compile(&input("")).unwrap();
    for css in ["#a{left:10px;top:-8%}","#a{position:relative;left:10px;position:static}","#a{position:relative;inset:4px;position:initial}"] {
        assert_eq!(compile(&input(css)).unwrap().riv,base.riv,"{css}");
    }
    assert_eq!(compile(&input("#root{position:relative;left:10px}#a{left:20px}")).unwrap().riv,compile(&input("#root{position:relative;left:10px}")).unwrap().riv);
    assert_eq!(compile(&input("#root{position:relative;left:10px}#a{position:inherit;left:inherit}")).unwrap().riv,compile(&input("#root,#a{position:relative;left:10px}")).unwrap().riv);
}
#[test]
fn relative_position_rejects_out_of_scope_and_invalid_values() {
    for declaration in ["position:fixed","position:sticky","position:banana","--p:fixed;position:var(--p)","left:1","inset:1px 2px 3px 4px 5px","top:-1000001px","left:10001%","left:calc(1px + 2%)","inset-inline:3px"] {
        assert!(compile(&input(&format!("#a{{{declaration}}}"))).is_err(),"{declaration}");
    }
}

#[test]
fn positioned_paint_contract_is_required_even_for_zero_offsets() {
    use nuxie_html_to_riv::RuntimeCapability;
    for css in ["#a{position:relative}","#a{position:relative;left:0}","#root{position:relative}#a{padding:5%;aspect-ratio:2/1}"] {
        let output=compile(&input(css)).unwrap();let r=&output.runtime_requirements;
        assert_eq!(r.version,17);assert!(!r.layout_positioned.is_empty());
        assert!(r.capabilities.contains(&RuntimeCapability::LayoutCssPositionedPaintV1));
        r.ensure_supported(&r.capabilities.iter().copied().collect::<Vec<_>>()).unwrap();
        let old_host=r.capabilities.iter().copied().filter(|c|*c!=RuntimeCapability::LayoutCssPositionedPaintV1).collect::<Vec<_>>();
        assert_eq!(r.ensure_supported(&old_host).unwrap_err().code,"missing-runtime-capability");
        r.ensure_layout_targets(|id|output.source_map.iter().any(|n|n.object_id==id)).unwrap();
    }
    let output=compile(&input("#a{position:relative;position:static}")).unwrap();
    assert!(output.runtime_requirements.layout_positioned.is_empty());
    assert!(!output.runtime_requirements.capabilities.contains(&RuntimeCapability::LayoutCssPositionedPaintV1));
}

#[test]
fn positioned_contract_rejects_inconsistent_payloads_and_targets() {
    use nuxie_html_to_riv::RuntimeCapability;
    let r=compile(&input("#a{position:relative}")).unwrap().runtime_requirements;
    let supported=r.capabilities.iter().copied().collect::<Vec<_>>();
    for mode in 0..5 {
        let mut invalid=r.clone();
        match mode {
            0=>invalid.version=16,
            1=>invalid.layout_positioned.clear(),
            2=>invalid.layout_positioned.push(invalid.layout_positioned[0]),
            3=>invalid.layout_positioned[0]=0,
            _=>{invalid.capabilities.remove(&RuntimeCapability::LayoutCssPositionedPaintV1);},
        }
        assert_eq!(invalid.ensure_supported(&supported).unwrap_err().code,"invalid-layout-positioned-paint");
    }
    assert_eq!(r.ensure_layout_targets(|_|false).unwrap_err().code,"invalid-layout-positioned-paint-target");
}
