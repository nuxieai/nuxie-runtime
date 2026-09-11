use nuxie_html_to_riv::{CompileInput, compile};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{File, RuntimeFactoryHandle};

#[test]
fn text_leading_does_not_enlarge_authored_height_but_preserves_auto_height() {
    use nuxie_html_to_riv::Asset;
    for (height, expected) in [("0px", 0.0), ("8px", 8.0), ("20px", 20.0), ("auto", 40.0)] {
        let output = compile(&CompileInput {
            html: "<div id=root><p id=text>A long title</p></div>".into(),
            css: format!("#root{{width:100%;padding:12px}}#text{{font:24px/40px Inter;white-space:nowrap;overflow:hidden;height:{height}}}"),
            width: 390.0, height: 200.0,
            assets: [("font".into(), Asset::Font { family: "Inter".into(), weight: 400, bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec() })].into(),
        }).unwrap();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(&output.riv, RuntimeFactoryHandle::from_factory(&mut factory).unwrap(), None, None, None).unwrap();
        let artboard = file.with_file(File::artboard_default).unwrap();
        for width in [240.0, 390.0, 768.0] {
            artboard.set_size(width, 200.0);
            artboard.update_pass(true);
            for (id, wanted) in [("text", expected), ("root", expected + 24.0)] {
                let node = output.source_map.iter().find(|n| n.id == id).unwrap();
                let object = artboard.with_artboard(|a| a.objects()[node.object_id as usize].clone()).unwrap();
                let actual = object.with(|o| o.as_layout_component().unwrap().layout_bounds().height()).unwrap();
                assert!((actual-wanted).abs()<0.1, "height:{height}, {id}, viewport:{width}: {actual} != {wanted}");
            }
        }
    }
}

#[test]
fn clipping_is_serialized_without_inheriting_or_baking_dimensions() {
    let output = compile(&CompileInput {
        html: "<div id=root><div id=a></div><div id=b></div><div id=c></div><div id=d></div></div>".into(),
        css: "#root{width:100%;height:40px;overflow:clip}#a{overflow:inherit}#b{overflow:clip;overflow:initial}#c{overflow:clip;overflow:unset}".into(),
        width:390.0,height:320.0,..Default::default()
    }).unwrap();
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
    for width in [240.0, 390.0, 768.0] {
        artboard.set_size(width, 320.0);
        artboard.update_pass(true);
        for (id, expected) in [
            ("root", true),
            ("a", true),
            ("b", false),
            ("c", false),
            ("d", false),
        ] {
            let node = output.source_map.iter().find(|n| n.id == id).unwrap();
            let object = artboard
                .with_artboard(|a| a.objects()[node.object_id as usize].clone())
                .unwrap();
            object
                .with(|o| {
                    let layout = o.as_layout_component().unwrap();
                    assert_eq!(layout.clip(), expected, "{id}");
                    if id == "root" {
                        assert!((layout.layout_bounds().width() - width).abs() < 0.1);
                    }
                })
                .unwrap();
        }
    }
}

#[test]
fn scrolling_and_scroll_computing_axis_pairs_are_explicitly_rejected() {
    for css in [
        "overflow:auto",
        "overflow:scroll",
        "overflow:hidden visible",
        "overflow-x:hidden",
        "overflow-y:hidden",
    ] {
        assert!(
            compile(&CompileInput {
                html: "<div></div>".into(),
                css: format!("div{{{css}}}"),
                width: 390.0,
                height: 320.0,
                ..Default::default()
            })
            .is_err(),
            "{css}"
        );
    }
}

#[test]
fn hidden_initial_scene_matches_clip_in_static_profile() {
    let input=CompileInput {html:"<div id=root><div id=child></div></div>".into(),css:"#root{overflow:clip;width:80%;height:40px;padding:4px;border-radius:12px}#child{width:900px;height:100px;background:red}".into(),width:390.0,height:320.0,..Default::default()};
    let clipped = compile(&input).unwrap();
    let hidden = compile(&CompileInput {
        css: input.css.replace("overflow:clip", "overflow:hidden"),
        ..input
    })
    .unwrap();
    assert_eq!(clipped.riv, hidden.riv);
    assert_eq!(clipped.runtime_requirements, hidden.runtime_requirements);
}
