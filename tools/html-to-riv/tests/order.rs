use nuxie_html_to_riv::{CompileInput, compile};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{File, RuntimeFactoryHandle};

#[test]
fn order_changes_runtime_layout_but_keeps_authored_ids_paths_and_selectors() {
    let source = CompileInput {
        html: "<div id='root'><div id='a'></div><div id='b'></div><div id='c'></div></div>".into(),
        css: "#root{width:100%;flex-direction:row;gap:5px}#root>div{width:20px;height:20px}#a{order:2}#b{order:-1}#root>:nth-child(3){order:-1;width:30px}".into(),
        width:390.,height:200.,..Default::default()
    };
    let output = compile(&source).unwrap();
    assert_eq!(
        output
            .source_map
            .iter()
            .map(|n| (n.id.as_str(), n.path.as_str()))
            .collect::<Vec<_>>(),
        vec![("root", "/0"), ("a", "/0/0"), ("b", "/0/1"), ("c", "/0/2")]
    );
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &output.riv,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(File::artboard_default).unwrap();
    for width in [240., 390., 768.] {
        artboard.set_size(width, 200.);
        artboard.update_pass(true);
        for (id, x) in [("b", 0.), ("c", 25.), ("a", 60.)] {
            let node = output.source_map.iter().find(|n| n.id == id).unwrap();
            let object = artboard
                .with_artboard(|a| a.objects()[node.object_id as usize].clone())
                .unwrap();
            let actual = object
                .with(|o| o.as_world_transform_component().unwrap().world_transform()[4])
                .unwrap();
            assert!(
                (actual - x).abs() < 0.01,
                "{id} at {width}: {actual} vs {x}"
            );
        }
    }
}

#[test]
fn order_cascade_substitution_and_integer_profile() {
    let base = CompileInput {
        html: "<div id='root'><div id='a'></div><div id='b'></div></div>".into(),
        css: "#root{order:2}#a,#b{width:20px;height:20px}".into(),
        width: 390.,
        height: 200.,
        ..Default::default()
    };
    let bytes = |extra: &str| {
        let mut input = base.clone();
        input.css.push_str(extra);
        compile(&input).unwrap().riv
    };
    assert!(bytes("#a{order:-1}") == bytes("#a{--o:-1;order:var(--o)}"));
    assert!(bytes("#a{order:inherit}") == bytes("#a{order:2}"));
    for reset in ["initial", "unset", "var(--missing)", "var(--bad)"] {
        assert!(
            bytes(&format!("#a{{--bad:1.5;order:{reset}}}")) == bytes("#a{order:0}"),
            "{reset}"
        );
    }
    assert!(
        bytes("") == bytes("#a,#b{order:0}"),
        "not implicitly inherited"
    );
    assert!(bytes("#a{order:+0002}") == bytes("#a{order:2}"));
    assert!(bytes("#a{order:2147483648}") == bytes("#a{order:2147483647}"));
    assert!(bytes("#a{order:-2147483649}") == bytes("#a{order:-2147483648}"));
    for invalid in [
        "1.0",
        "1e0",
        "1px",
        "1%",
        "1 2",
        "auto",
        "calc(1 + 1)",
        "var(--o)",
    ] {
        let mut input = base.clone();
        input
            .css
            .push_str(&format!("#a{{--o:calc(1 + 1);order:{invalid}}}"));
        assert!(
            compile(&input).is_err(),
            "{invalid} must be diagnosed, not coerced"
        );
    }
}

#[test]
fn explicit_css_paint_policy_reorders_whole_groups_and_can_restore_raw_order() {
    let input = CompileInput {
        html:"<div id='root'><div id='a'><div id='ap'></div></div><div id='b'><div id='bp'></div></div><div id='c'><div id='cp'></div></div></div>".into(),
        css:"#root{width:120px;flex-direction:row;overflow:clip}#root>div{width:40px;height:40px}#a{order:2}#b{order:-1}#ap,#bp,#cp{width:100px;height:40px}#ap{background:#f44}#bp{background:#48f}#cp{background:#4b6}".into(),
        width:390.,height:200.,..Default::default()
    };
    let output = compile(&input).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &output.riv,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(File::artboard_default).unwrap();
    for (enabled, expected) in [
        (false, ["0xffff4444", "0xff44bb66", "0xff4488ff"]),
        (true, ["0xff4488ff", "0xff44bb66", "0xffff4444"]),
        (true, ["0xff4488ff", "0xff44bb66", "0xffff4444"]),
        (false, ["0xffff4444", "0xff44bb66", "0xff4488ff"]),
    ] {
        artboard.with_artboard_mut(|a| a.set_css_paint_order(enabled));
        for width in [240., 390., 768.] {
            artboard.set_size(width, 200.);
            artboard.update_pass(true);
            let before = factory.borrow().stream().len();
            let mut renderer = factory.borrow().make_renderer();
            artboard.draw(&mut renderer);
            let stream = factory.borrow().stream();
            let colors = stream[before..]
                .lines()
                .filter(|line| line.starts_with("drawPath "))
                .map(|line| {
                    line.split("color=")
                        .nth(1)
                        .unwrap()
                        .split(',')
                        .next()
                        .unwrap()
                })
                .collect::<Vec<_>>();
            assert_eq!(colors, expected, "enabled={enabled}, width={width}");
            for (id, x) in [("b", 0.), ("c", 40.), ("a", 80.)] {
                let node = output.source_map.iter().find(|n| n.id == id).unwrap();
                let object = artboard
                    .with_artboard(|a| a.objects()[node.object_id as usize].clone())
                    .unwrap();
                let actual = object
                    .with(|o| o.as_world_transform_component().unwrap().world_transform()[4])
                    .unwrap();
                assert!(
                    (actual - x).abs() < 0.01,
                    "paint policy must not change {id} layout"
                );
            }
        }
    }
}

#[test]
fn paint_order_capability_is_required_for_siblings_even_without_order() {
    use nuxie_html_to_riv::RuntimeCapability;
    for (html, expected) in [
        ("<div></div>", false),
        ("<div><div></div></div>", false),
        ("<div></div><div></div>", true),
        ("<div><div></div><div></div></div>", true),
    ] {
        let output = compile(&CompileInput {
            html: html.into(),
            width: 240.,
            height: 200.,
            ..Default::default()
        })
        .unwrap();
        let requirements = output.runtime_requirements;
        assert_eq!(
            requirements
                .capabilities
                .contains(&RuntimeCapability::LayoutCssPaintOrderV1),
            expected
        );
        if expected {
            assert_eq!(
                requirements.ensure_supported(&[]).unwrap_err().code,
                "missing-runtime-capability"
            );
            requirements
                .ensure_supported(&[RuntimeCapability::LayoutCssPaintOrderV1])
                .unwrap();
        }
    }
}

#[test]
fn css_paint_order_is_an_explicit_instance_policy() {
    let output = compile(&CompileInput {
        html: "<div id=a></div><div id=b></div>".into(),
        css: "div{width:30px;height:20px}#a{background:#f44}#b{background:#48f}".into(),
        width: 240.,
        height: 200.,
        ..Default::default()
    })
    .unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &output.riv,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let first = file.with_file(File::artboard_default).unwrap();
    first.with_artboard_mut(|a| a.set_css_paint_order(true));
    let second = first.instance().unwrap();
    let colors = |instance: &nuxie_runtime::source::artboard::RuntimeArtboardInstanceHandle| {
        instance.update_pass(true);
        let before = factory.borrow().stream().len();
        let mut renderer = factory.borrow().make_renderer();
        instance.draw(&mut renderer);
        factory.borrow().stream()[before..]
            .lines()
            .filter(|line| line.starts_with("drawPath "))
            .map(|line| {
                line.split("color=")
                    .nth(1)
                    .unwrap()
                    .split(',')
                    .next()
                    .unwrap()
                    .to_string()
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(colors(&first), ["0xffff4444", "0xff4488ff"]);
    assert_eq!(colors(&second), ["0xffff4444", "0xff4488ff"]);
    second.with_artboard_mut(|a| a.set_css_paint_order(false));
    assert_eq!(colors(&second), ["0xff4488ff", "0xffff4444"]);
    assert_eq!(colors(&first), ["0xffff4444", "0xff4488ff"]);
    second.with_artboard_mut(|a| a.set_css_paint_order(true));
    assert_eq!(colors(&second), ["0xffff4444", "0xff4488ff"]);
    first.with_artboard_mut(|a| a.set_css_paint_order(false));
    assert_eq!(colors(&second), ["0xffff4444", "0xff4488ff"]);
}

#[test]
fn css_paint_order_tracks_reverse_flex_directions() {
    for direction in ["row","column","row-reverse","column-reverse"] {
        let output=compile(&CompileInput {html:"<div id=root><div id=a></div><div id=b></div><div id=c></div></div>".into(),css:format!("#root{{width:120px;height:120px;flex-direction:{direction}}}#a,#b,#c{{width:40px;height:40px}}#a{{order:2;background:#f44}}#b{{order:-1;background:#48f}}#c{{background:#4b6}}"),width:240.,height:200.,..Default::default()}).unwrap();
        let mut factory=PersistentFactory::new(RecordingFactory::new());
        let file=File::import(&output.riv,RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),None,None,None).unwrap();
        let artboard=file.with_file(File::artboard_default).unwrap();
        artboard.with_artboard_mut(|a|a.set_css_paint_order(true));
        artboard.update_pass(true);
        let before=factory.borrow().stream().len();
        let mut renderer=factory.borrow().make_renderer();artboard.draw(&mut renderer);
        let stream=factory.borrow().stream();
        let colors=stream[before..].lines().filter(|line|line.starts_with("drawPath ")).map(|line|line.split("color=").nth(1).unwrap().split(',').next().unwrap()).collect::<Vec<_>>();
        let expected=if direction.ends_with("reverse") {["0xffff4444","0xff44bb66","0xff4488ff"]}else{["0xff4488ff","0xff44bb66","0xffff4444"]};
        assert_eq!(colors,expected,"{direction}");
    }
}
