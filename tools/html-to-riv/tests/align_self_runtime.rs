use nuxie_html_to_riv::{CompileInput, compile};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::layout_component::{CssAlignSelf, LayoutComponent};
use nuxie_runtime::{File, RuntimeFactoryHandle};

#[test]
fn compiler_alignment_matches_browser_box_oracle_after_resize() {
    let oracle: serde_json::Value =
        serde_json::from_str(include_str!("assets/align-self-boxes.json")).unwrap();
    for case in oracle["cases"].as_array().unwrap() {
        let name = case["alignment"].as_str().unwrap();
        let policy = match name {
            "auto" => CssAlignSelf::Auto,
            "flex-start" => CssAlignSelf::FlexStart,
            "center" => CssAlignSelf::Center,
            "flex-end" => CssAlignSelf::FlexEnd,
            "stretch" => CssAlignSelf::Stretch,
            "baseline" => CssAlignSelf::Baseline,
            _ => unreachable!(),
        };
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
            .find(|n| n.id == "a")
            .unwrap()
            .object_id;
        let object = artboard
            .with_artboard(|a| a.objects()[id as usize].clone())
            .unwrap();
        if name == "auto" {
            assert!(output.runtime_requirements.layout_align_self.is_empty());
        } else {
            assert_eq!(output.runtime_requirements.version, 6);
            let targets = &output.runtime_requirements.layout_align_self;
            assert_eq!(targets.len(), 1);
            assert_eq!(targets[0].object_id, id);
            assert_eq!(serde_json::to_value(targets[0].alignment).unwrap(), name);
            assert!(LayoutComponent::set_css_align_self_occurrence(
                &object,
                Some(policy)
            ));
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
                        "{} {} {} at {width}, {} {key}: {} vs {expected}",
                        case["direction"],
                        case["alignment"],
                        case["sizing"],
                        node.id,
                        actual[index]
                    );
                }
            }
        }
    }
}

#[test]
fn alignment_occurrence_survives_clone_and_clear_restores_default() {
    let output=compile(&CompileInput {html:"<div id=root><div id=a></div></div>".into(),css:"#root{width:100%;height:120px;padding:10px;flex-direction:row;align-items:flex-end}#a{width:40px;height:30px}".into(),width:390.,height:320.,..Default::default()}).unwrap();
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
    let id = output
        .source_map
        .iter()
        .find(|n| n.id == "a")
        .unwrap()
        .object_id;
    let object = first
        .with_artboard(|a| a.objects()[id as usize].clone())
        .unwrap();
    let y = |instance: &nuxie_runtime::source::artboard::RuntimeArtboardInstanceHandle| {
        instance.update_pass(true);
        instance
            .with_artboard(|a| a.objects()[id as usize].clone())
            .unwrap()
            .with(|o| o.as_world_transform_component().unwrap().world_transform()[5])
            .unwrap()
    };
    assert_eq!(y(&first), 80.);
    assert!(LayoutComponent::set_css_align_self_occurrence(
        &object,
        Some(CssAlignSelf::Center)
    ));
    assert_eq!(y(&first), 45.);
    let second = first.instance().unwrap();
    assert_eq!(y(&second), 45.);
    let other = second
        .with_artboard(|a| a.objects()[id as usize].clone())
        .unwrap();
    assert!(LayoutComponent::set_css_align_self_occurrence(&other, None));
    assert_eq!(y(&second), 80.);
    assert_eq!(y(&first), 45.);
    assert!(LayoutComponent::set_css_align_self_occurrence(
        &object,
        Some(CssAlignSelf::FlexStart)
    ));
    assert_eq!(y(&first), 10.);
    assert!(LayoutComponent::set_css_align_self_occurrence(
        &object,
        Some(CssAlignSelf::Auto)
    ));
    assert_eq!(y(&first), 80.);
}

#[test]
fn text_baselines_reflow_after_policy_changes_and_clone() {
    use nuxie_html_to_riv::Asset;
    let mut input=CompileInput{html:"<div id=root><p id=a>Small</p><p id=b>Large</p><p id=c>Medium</p></div>".into(),css:"#root{width:100%;height:100px;padding:10px;gap:8px;flex-direction:row;align-items:flex-start;font-family:Inter}p{display:block;white-space:nowrap}#a{font-size:16px;line-height:24px}#b{font-size:32px;line-height:40px}#c{font-size:20px;line-height:28px}".into(),width:390.,height:320.,..Default::default()};
    input.assets.insert(
        "inter".into(),
        Asset::Font {
            family: "Inter".into(),
            weight: 400,
            bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
        },
    );
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
    let first = file.with_file(File::artboard_default).unwrap();
    let ids: Vec<_> = ["a", "b", "c"]
        .iter()
        .map(|id| {
            output
                .source_map
                .iter()
                .find(|n| n.id == *id)
                .unwrap()
                .object_id
        })
        .collect();
    let check = |instance: &nuxie_runtime::source::artboard::RuntimeArtboardInstanceHandle,
                 expected: [f32; 3]| {
        for width in [240., 390., 768.] {
            instance.set_size(width, 320.);
            instance.update_pass(true);
            for (id, y) in ids.iter().zip(expected) {
                let object = instance
                    .with_artboard(|a| a.objects()[*id as usize].clone())
                    .unwrap();
                let actual = object
                    .with(|o| o.as_world_transform_component().unwrap().world_transform()[5])
                    .unwrap();
                assert!((actual - y).abs() < 0.1, "{width}: {actual} vs {y}");
            }
        }
    };
    check(&first, [10., 10., 10.]);
    for alignment in [
        Some(CssAlignSelf::Baseline),
        None,
        Some(CssAlignSelf::Baseline),
    ] {
        for id in &ids {
            let object = first
                .with_artboard(|a| a.objects()[*id as usize].clone())
                .unwrap();
            assert!(LayoutComponent::set_css_align_self_occurrence(
                &object, alignment
            ));
        }
        check(
            &first,
            if alignment.is_some() {
                [23., 10., 20.]
            } else {
                [10., 10., 10.]
            },
        );
    }
    check(&first.instance().unwrap(), [23., 10., 20.]);
}

#[test]
fn nested_intrinsic_width_survives_nowrap_alignment_policy_toggle() {
    use nuxie_html_to_riv::{Asset, TextPolicy};
    let mut input = CompileInput {
        html: "<div id=root><div id=wrapper><p id=label>Small</p></div></div>".into(),
        css: "#root{width:100%;flex-direction:row;font-family:Inter}#label{display:block;white-space:nowrap;font-size:16px;line-height:24px}".into(),
        width: 390., height: 320., ..Default::default()
    };
    input.assets.insert(
        "inter".into(),
        Asset::Font {
            family: "Inter".into(),
            weight: 400,
            bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
        },
    );
    let output = compile(&input).unwrap();
    assert!(output.runtime_requirements.layout_align_self.is_empty());
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &output.riv,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let instance = file.with_file(File::artboard_default).unwrap();
    let text_policy = output
        .runtime_requirements
        .text_policies
        .iter()
        .find(|entry| entry.policy == TextPolicy::CssNowrapAlignmentV1)
        .unwrap();
    let text = instance
        .with_artboard(|a| a.objects()[text_policy.object_id as usize].clone())
        .unwrap();
    for enabled in [false, true, false, true] {
        text.with_mut(|o| o.as_text_mut().unwrap().set_css_nowrap_alignment(enabled));
        for viewport in [240., 390., 768.] {
            instance.set_size(viewport, 320.);
            instance.update_pass(true);
            for id in ["wrapper", "label"] {
                let source = output
                    .source_map
                    .iter()
                    .find(|entry| entry.id == id)
                    .unwrap();
                let width = instance
                    .with_artboard(|a| {
                        a.objects()[source.object_id as usize]
                            .as_ref()
                            .unwrap()
                            .with(|o| o.as_layout_component().unwrap().layout().width())
                    })
                    .unwrap();
                // Chromium153 / Inter fixture advance, independently recorded
                // in align-nested-width-fixed's scene.browser.json.
                // Version11 encodes intrinsic width independently of the nowrap
                // alignment policy, so toggling alignment must retain that width.
                let expected = 40.390625;
                assert!(
                    (width - expected).abs() < 0.1,
                    "{enabled} at {viewport}, {id}: {width} vs {expected}"
                );
            }
        }
    }
}

#[test]
fn intrinsic_text_keeps_available_alignment_width_after_resize_and_clone() {
    use nuxie_html_to_riv::Asset;
    for alignment in ["left", "center", "right"] {
        for whitespace in ["normal", "nowrap"] {
            let mut input = CompileInput {
                html: "<div id=parent><p id=child>Inherited font and text alignment wrap together.</p></div>".into(),
                css: format!("#parent{{width:100%;padding:12px;font-family:Inter;font-size:16px;line-height:24px;text-align:{alignment};white-space:{whitespace}}}#child{{text-align:inherit}}"),
                width: 390., height: 320., ..Default::default()
            };
            input.assets.insert("inter".into(), Asset::Font {
                family: "Inter".into(), weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            });
            let output = compile(&input).unwrap();
            let mut factory = PersistentFactory::new(RecordingFactory::new());
            let file = File::import(&output.riv,
                RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
                None, None, None).unwrap();
            let original = file.with_file(File::artboard_default).unwrap();
            for instance in [original.clone(), original.instance().unwrap()] {
                for width in [240., 390., 768., 240.] {
                    instance.set_size(width, 320.);
                    instance.update_pass(true);
                    let widths = instance.with_artboard(|a| a.objects().iter().flatten()
                        .filter_map(|object| object.with(|o| o.as_text().map(|t| t.effective_width())).flatten())
                        .collect::<Vec<_>>());
                    assert_eq!(widths.len(), 1);
                    // Chromium's content box is the viewport minus both 12px
                    // paddings, even when one short line fits at intrinsic width.
                    assert!((widths[0] - (width - 24.)).abs() < 0.1,
                        "{alignment} {whitespace} at {width}: {}", widths[0]);
                }
            }
        }
    }
}
