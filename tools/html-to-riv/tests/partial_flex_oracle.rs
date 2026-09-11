use nuxie_html_to_riv::{CompileInput, compile};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::layout_component::{
    CssAlignContent, CssJustifyContent, LayoutComponent,
};
use nuxie_runtime::{File, RuntimeFactoryHandle};

#[test]
fn compiled_partial_factors_match_browser_after_resize() {
    let oracle: serde_json::Value =
        serde_json::from_str(include_str!("assets/partial-flex-boxes.json")).unwrap();
    let mut mismatches = Vec::new();
    for case in oracle["cases"].as_array().unwrap() {
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
        requirements
            .ensure_supported(
                &requirements
                    .capabilities
                    .iter()
                    .copied()
                    .collect::<Vec<_>>(),
            )
            .unwrap();
        requirements
            .ensure_layout_targets(|target| output.source_map.iter().any(|n| n.object_id == target))
            .unwrap();
        for entry in &requirements.layout_flex_factors {
            use nuxie_runtime::source::layout_component::CssFlexFactors;
            let target = artboard
                .with_artboard(|a| a.objects()[entry.object_id as usize].clone())
                .unwrap();
            assert!(LayoutComponent::set_css_flex_factors_occurrence(
                &target,
                CssFlexFactors::new(entry.grow, entry.shrink)
            ));
        }
        for entry in &requirements.layout_justify_content {
            let policy = match entry.alignment {
                JustifyDistribution::SpaceAround => CssJustifyContent::SpaceAround,
                JustifyDistribution::SpaceEvenly => CssJustifyContent::SpaceEvenly,
            };
            assert!(LayoutComponent::set_css_justify_content_occurrence(
                &object,
                Some(policy)
            ));
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
            assert!(LayoutComponent::set_css_align_content_occurrence(
                &object,
                Some(policy)
            ));
        }
        for entry in &requirements.layout_align_self {
            use nuxie_html_to_riv::AlignSelf;
            use nuxie_runtime::source::layout_component::CssAlignSelf;
            let target = artboard
                .with_artboard(|a| a.objects()[entry.object_id as usize].clone())
                .unwrap();
            let policy = match entry.alignment {
                AlignSelf::Auto => CssAlignSelf::Auto,
                AlignSelf::FlexStart => CssAlignSelf::FlexStart,
                AlignSelf::Center => CssAlignSelf::Center,
                AlignSelf::FlexEnd => CssAlignSelf::FlexEnd,
                AlignSelf::Stretch => CssAlignSelf::Stretch,
                AlignSelf::Baseline => CssAlignSelf::Baseline,
            };
            assert!(LayoutComponent::set_css_align_self_occurrence(
                &target,
                Some(policy)
            ));
        }
        for artboard in [artboard.clone(), artboard.instance().unwrap()] {
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
                        if (actual[index] - expected).abs() >= 0.1 {
                            mismatches.push(format!(
                                "{} {} {} at {width}, {} {key}: {} vs {expected}",
                                case["direction"],
                                case["grow"],
                                case["mode"],
                                node.id,
                                actual[index]
                            ));
                        }
                    }
                }
            }
        }
    }
    assert!(
        mismatches.is_empty(),
        "{} mismatched coordinates:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}
