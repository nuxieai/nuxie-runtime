use nuxie_html_to_riv::{compile, CompileInput};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::layout_component::{CssFlexFactors, LayoutComponent};
use nuxie_runtime::{File, RuntimeFactoryHandle};

#[test]
fn percentage_basis_preserves_browser_geometry_after_clone_and_repeated_resize() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!(
        "assets/indefinite-factor-boxes.json")).unwrap();
    let mut failures = Vec::new();
    for case in oracle["cases"].as_array().unwrap() {
        let output = compile(&CompileInput {
            html: case["html"].as_str().unwrap().into(),
            css: case["css"].as_str().unwrap().into(),
            width: 390., height: 320., ..Default::default()
        }).unwrap();
        assert_eq!(output.runtime_requirements.version, 12);
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(&output.riv,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None, None, None).unwrap();
        let original = file.with_file(File::artboard_default).unwrap();
        let requirements = &output.runtime_requirements;
        requirements.ensure_supported(&requirements.capabilities.iter().copied().collect::<Vec<_>>()).unwrap();
        requirements.ensure_layout_targets(|id| output.source_map.iter().any(|n| n.object_id == id)).unwrap();
        assert!(requirements.layout_intrinsic_sizing.is_empty(), "box-only oracle");
        for entry in &requirements.layout_flex_factors {
            let target = original.with_artboard(|a| a.objects()[entry.object_id as usize].clone()).unwrap();
            assert!(LayoutComponent::set_css_flex_factors_occurrence(
                &target, CssFlexFactors::new(entry.grow, entry.shrink)));
        }
        for (instance_index, instance) in [original.clone(), original.instance().unwrap()].into_iter().enumerate() {
            for width in [240., 390., 768., 240.] {
                instance.set_size(width, 320.);
                instance.update_pass(true);
                let reference = case["viewports"].as_array().unwrap().iter()
                    .find(|v| v["width"].as_f64() == Some(width as f64)).unwrap();
                for node in &output.source_map {
                    let target = instance.with_artboard(|a| a.objects()[node.object_id as usize].clone()).unwrap();
                    let actual = target.with(|o| {
                        let layout = o.as_layout_component().unwrap();
                        let transform = o.as_world_transform_component().unwrap().world_transform();
                        [transform[4], transform[5], layout.layout().width(), layout.layout().height()]
                    }).unwrap();
                    for (index, axis) in ["x", "y", "width", "height"].iter().enumerate() {
                        let expected = reference["boxes"][&node.id][axis].as_f64().unwrap() as f32;
                        if !actual[index].is_finite() || (actual[index] - expected).abs() > 0.1 {
                            failures.push(format!("{} instance{instance_index} at{width} {}.{axis}: {} vs{expected}",
                                case["name"], node.id, actual[index]));
                        }
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{} coordinate mismatches:\n{}", failures.len(), failures.join("\n"));
}
