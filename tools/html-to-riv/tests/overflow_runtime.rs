use nuxie_html_to_riv::{compile, Asset, CompileInput};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{File, RuntimeFactoryHandle};

#[test]
fn public_overflow_preserves_chrome_geometry_across_clone_resize() {
    let oracle: serde_json::Value =
        serde_json::from_str(include_str!("assets/overflow-rounded-oracle.json")).unwrap();
    let nested: serde_json::Value =
        serde_json::from_str(include_str!("assets/overflow-nested-oracle.json")).unwrap();
    let axes: serde_json::Value = serde_json::from_str(include_str!("assets/overflow-axis-oracle.json")).unwrap();
    let composition: serde_json::Value = serde_json::from_str(include_str!("assets/overflow-axis-composition-oracle.json")).unwrap();
    let shortouter: serde_json::Value = serde_json::from_str(include_str!("assets/overflow-axis-shortouter-oracle.json")).unwrap();
    let plain: serde_json::Value = serde_json::from_str(include_str!("assets/overflow-axis-plain-oracle.json")).unwrap();
    let clip_margin: serde_json::Value = serde_json::from_str(include_str!("assets/overflow-clip-margin-oracle.json")).unwrap();
    let expanded: serde_json::Value = serde_json::from_str(include_str!("assets/overflow-clip-margin-expanded-oracle.json")).unwrap();
    let signed: serde_json::Value = serde_json::from_str(include_str!("assets/overflow-clip-margin-signed-oracle.json")).unwrap();
    let margin_composition: serde_json::Value = serde_json::from_str(include_str!("assets/overflow-clip-margin-composition-oracle.json")).unwrap();
    let mut failures = Vec::new();
    let mut updates = 0;
    for case in oracle["cases"]
        .as_array()
        .unwrap()
        .iter()
        .chain(nested["cases"].as_array().unwrap())
        .chain(axes["cases"].as_array().unwrap())
        .chain(composition["cases"].as_array().unwrap())
        .chain(shortouter["cases"].as_array().unwrap())
        .chain(plain["cases"].as_array().unwrap())
        .chain(clip_margin["cases"].as_array().unwrap())
        .chain(expanded["cases"].as_array().unwrap())
        .chain(signed["cases"].as_array().unwrap())
        .chain(margin_composition["cases"].as_array().unwrap())
    {
        let name = case["name"].as_str().unwrap();
        let mut input = CompileInput {
            html: case["html"].as_str().unwrap().into(),
            css: case["css"].as_str().unwrap().into(),
            width: 390.,
            height: 320.,
            ..Default::default()
        };
        if case["font"].as_bool() == Some(true) {
            input.assets.insert(
                "inter".into(),
                Asset::Font {
                    family: "Inter".into(),
                    weight: 400,
                    bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
                },
            );
        }
        if case["image"].as_bool() == Some(true) {
            input.assets.insert(
                "photo".into(),
                Asset::Image {
                    bytes: include_bytes!("assets/quadrants.png").to_vec(),
                },
            );
        }
        let output = compile(&input).unwrap();
        let requirements = &output.runtime_requirements;
        requirements
            .ensure_supported(
                &requirements
                    .capabilities
                    .iter()
                    .copied()
                    .collect::<Vec<_>>(),
            )
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
        let original = file.with_file(File::artboard_default).unwrap();
        if !requirements.layout_absolute.is_empty() {
            let root = original.with_artboard(|a| a.objects()[0].clone()).unwrap();
            assert!(nuxie_runtime::source::artboard::Artboard::set_css_absolute_position_policy_occurrence(
                &root, &requirements.layout_positioned, &requirements.layout_absolute));
        }
        assert!(original.with_artboard_mut(
            |a| a.set_css_positioned_paint_order(&requirements.layout_positioned)
        ));
        let contexts = requirements
            .layout_stacking
            .iter()
            .map(|v| (v.object_id, v.level))
            .collect::<Vec<_>>();
        assert!(original.with_artboard_mut(|a| a.set_css_stacking_order(&contexts)));
        let axis_targets = requirements.layout_axis_overflow.iter().map(|entry| (entry.object_id, match entry.axis {
            nuxie_html_to_riv::OverflowAxis::X => nuxie_runtime::source::layout_component::CssOverflowAxis::Horizontal,
            nuxie_html_to_riv::OverflowAxis::Y => nuxie_runtime::source::layout_component::CssOverflowAxis::Vertical,
        })).collect::<Vec<_>>();
        assert!(nuxie_runtime::source::artboard::Artboard::set_css_overflow_axes_occurrence(&original.core_handle(), &axis_targets));
        use nuxie_runtime::source::layout_component::{CssOverflowClipBox, CssOverflowClipMargin};
        let margins = requirements.layout_overflow_clip_margins.iter().map(|entry| {
            let origin = match entry.origin {
                nuxie_html_to_riv::OverflowClipBox::ContentBox => CssOverflowClipBox::Content,
                nuxie_html_to_riv::OverflowClipBox::PaddingBox => CssOverflowClipBox::Padding,
                nuxie_html_to_riv::OverflowClipBox::BorderBox => CssOverflowClipBox::Border,
            };
            (entry.object_id, CssOverflowClipMargin::new(origin, entry.pixels).unwrap())
        }).collect::<Vec<_>>();
        assert!(nuxie_runtime::source::artboard::Artboard::set_css_overflow_clip_margins_occurrence(&original.core_handle(), &margins));
        let cloned = original.instance().unwrap();
        for (index, instance) in [original.clone(), cloned.clone()].into_iter().enumerate() {
            for width in [240., 390., 768., 240.] {
                instance.set_size(width, 320.);
                instance.update_pass(true);
                updates += 1;
                let reference = case["viewports"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|v| v["width"].as_f64() == Some(width as f64))
                    .unwrap();
                for source in &output.source_map {
                    let owner = instance
                        .with_artboard(|a| a.objects()[source.object_id as usize].clone())
                        .unwrap();
                    let actual = owner
                        .with(|o| {
                            let layout = o.as_layout_component().unwrap().layout();
                            let transform =
                                o.as_world_transform_component().unwrap().world_transform();
                            [transform[4], transform[5], layout.width(), layout.height()]
                        })
                        .unwrap();
                    for (i, axis) in ["x", "y", "width", "height"].into_iter().enumerate() {
                        let expected =
                            reference["boxes"][&source.id][axis].as_f64().unwrap() as f32;
                        if !actual[i].is_finite() || (actual[i] - expected).abs() > 0.1 {
                            failures.push(format!("{name}, instance={index}, width={width}, {}.{axis}: {} vs {expected}", source.id, actual[i]));
                        }
                    }
                }
            }
        }
    }
    assert_eq!(updates, 303 * 2 * 4);
    assert!(
        failures.is_empty(),
        "{} geometry failures: {failures:#?}",
        failures.len()
    );
}
