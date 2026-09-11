use nuxie_html_to_riv::{CompileInput, compile};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::layout_component::{CssAlignContent, CssJustifyContent, LayoutComponent};
use nuxie_runtime::{File, RuntimeFactoryHandle};

#[test]
fn compiled_distributed_spacing_matches_browser_after_resize() {
    let oracle: serde_json::Value =
        serde_json::from_str(include_str!("assets/distributed-spacing-boxes.json")).unwrap();
    for case in oracle["cases"].as_array().unwrap() {
        let name = case["alignment"].as_str().unwrap();
        let property = case["property"].as_str().unwrap();
        let css = case["css"].as_str().unwrap().to_string();
        let output = compile(&CompileInput {
            html: case["html"].as_str().unwrap().into(),
            css,
            width: 390.,
            height: 320.,
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
        let artboard = file.with_file(File::artboard_default).unwrap();
        let id = output
            .source_map
            .iter()
            .find(|n| n.id == "root")
            .unwrap()
            .object_id;
        let object = artboard
            .with_artboard(|a| a.objects()[id as usize].clone())
            .unwrap();
        use nuxie_html_to_riv::{AlignContent, JustifyDistribution};
        let requirements = &output.runtime_requirements;
        requirements.ensure_supported(&requirements.capabilities.iter().copied().collect::<Vec<_>>()).unwrap();
        requirements.ensure_layout_targets(|target| target == id).unwrap();
        let new_value = name == "space-evenly" || (property == "justify-content" && name == "space-around");
        assert_eq!(requirements.version == 8, new_value);
        for entry in &requirements.layout_justify_content {
            let policy = match entry.alignment {
                JustifyDistribution::SpaceAround => CssJustifyContent::SpaceAround,
                JustifyDistribution::SpaceEvenly => CssJustifyContent::SpaceEvenly,
            };
            assert!(LayoutComponent::set_css_justify_content_occurrence(&object, Some(policy)));
        }
        for entry in &requirements.layout_align_content {
            let policy = match entry.alignment {
                AlignContent::FlexStart => CssAlignContent::FlexStart,
                AlignContent::Center => CssAlignContent::Center,
                AlignContent::FlexEnd => CssAlignContent::FlexEnd,
                AlignContent::Stretch => CssAlignContent::Stretch,
                AlignContent::SpaceBetween => CssAlignContent::SpaceBetween,
                AlignContent::SpaceAround => CssAlignContent::SpaceAround,
                AlignContent::SpaceEvenly => CssAlignContent::SpaceEvenly,
            };
            assert!(LayoutComponent::set_css_align_content_occurrence(&object, Some(policy)));
        }
        for viewport in case["viewports"].as_array().unwrap() {
            let width = viewport["width"].as_f64().unwrap() as f32;
            artboard.set_size(width, 320.);
            artboard.update_pass(true);
            for node in &output.source_map {
                let object = artboard
                    .with_artboard(|a| a.objects()[node.object_id as usize].clone())
                    .unwrap();
                let actual = object
                    .with(|o| {
                        let layout = o.as_layout_component().unwrap();
                        let m = o.as_world_transform_component().unwrap().world_transform();
                        [
                            m[4],
                            m[5],
                            layout.layout().width(),
                            layout.layout().height(),
                        ]
                    })
                    .unwrap();
                for (index, key) in ["x", "y", "width", "height"].iter().enumerate() {
                    let expected = viewport["boxes"][&node.id][key].as_f64().unwrap() as f32;
                    assert!(
                        (actual[index] - expected).abs() < 0.1,
                        "{} {} {} {} at {width}, {} {key}: {} vs {expected}",
                        property,
                        case["direction"],
                        case["alignment"],
                        case["mode"],
                        node.id,
                        actual[index]
                    );
                }
            }
        }
    }
}

#[test]
fn main_alignment_occurrence_survives_clone_and_clear_restores_rive_default() {
    let output = compile(&CompileInput {
        html: "<div id=root><div id=a></div></div>".into(),
        css: "#root{width:100%;height:160px;padding:10px;flex-direction:column;justify-content:center;align-items:center}#a{width:40px;height:20px}".into(),
        width: 390., height: 320., ..Default::default()
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
    let first = file.with_file(File::artboard_default).unwrap();
    let root_id = output
        .source_map
        .iter()
        .find(|n| n.id == "root")
        .unwrap()
        .object_id;
    let child_id = output
        .source_map
        .iter()
        .find(|n| n.id == "a")
        .unwrap()
        .object_id;
    let root = first
        .with_artboard(|a| a.objects()[root_id as usize].clone())
        .unwrap();
    let check = |instance: &nuxie_runtime::source::artboard::RuntimeArtboardInstanceHandle,
                 expected: f32| {
        for width in [240., 390., 768.] {
            instance.set_size(width, 320.);
            instance.update_pass(true);
            let child = instance
                .with_artboard(|a| a.objects()[child_id as usize].clone())
                .unwrap();
            let y = child
                .with(|o| o.as_world_transform_component().unwrap().world_transform()[5])
                .unwrap();
            assert!((y - expected).abs() < 0.1, "{width}: {y} vs {expected}");
        }
    };
    // Authored Rive justification is restored when the occurrence policy is cleared.
    check(&first, 70.);
    for (policy, y) in [
        (CssJustifyContent::FlexStart, 10.),
        (CssJustifyContent::FlexEnd, 130.),
        (CssJustifyContent::Center, 70.),
        (CssJustifyContent::SpaceAround, 70.),
        (CssJustifyContent::SpaceEvenly, 70.),
        (CssJustifyContent::SpaceBetween, 10.),
        (CssJustifyContent::FlexStart, 10.),
    ] {
        assert!(LayoutComponent::set_css_justify_content_occurrence(
            &root,
            Some(policy)
        ));
        check(&first, y);
    }
    let clone = first.instance().unwrap();
    check(&clone, 10.);
    let clone_root = clone
        .with_artboard(|a| a.objects()[root_id as usize].clone())
        .unwrap();
    assert!(LayoutComponent::set_css_justify_content_occurrence(
        &clone_root,
        None
    ));
    check(&clone, 70.);
    check(&first, 10.);
    assert!(LayoutComponent::set_css_justify_content_occurrence(
        &root, None
    ));
    check(&first, 70.);
}
