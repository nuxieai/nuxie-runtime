use nuxie_html_to_riv::{compile, CompileInput};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::layout_component::{CssFlexFactors, LayoutComponent};
use nuxie_runtime::{File, RuntimeFactoryHandle};

// Runtime foundation: inject the existing wire property; public CSS admission
// and rendering qualification are separate gates.
#[test]
fn aspect_ratio_auto_reference_box_matches_chrome_after_clone_and_resize() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!(
        "assets/aspect-ratio-auto-probe-oracle.json")).unwrap();
    let mut failures = Vec::new();
    for case in oracle["cases"].as_array().unwrap() {
        let output = compile(&CompileInput {
            html: case["html"].as_str().unwrap().into(),
            css: case["css"].as_str().unwrap().split(';').filter(|part| !part.trim().starts_with("aspect-ratio:")).collect::<Vec<_>>().join(";"),
            width: 390., height: 320., ..Default::default()
        }).unwrap();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(&output.riv,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None, None, None).unwrap();
        let original = file.with_file(File::artboard_default).unwrap();
        for id in &output.runtime_requirements.layout_content_box {
            let target = original.with_artboard(|a| a.objects()[*id as usize].clone()).unwrap();
            assert!(LayoutComponent::set_css_content_box_occurrence(&target, true));
        }
        let node = output.source_map.iter().find(|node| node.id == "a").unwrap();
        let target = original.with_artboard(|a| a.objects()[node.object_id as usize].clone()).unwrap();
        let style = target.with(|o| o.as_layout_component().unwrap().style_handle().unwrap()).unwrap();
        let name = case["name"].as_str().unwrap();
        let ratio: f32 = if name.ends_with("-height") && !name.ends_with("-min-height") { 1.5 } else if name.ends_with("-percent") { 16. / 9. } else { 2. };
        let bytes = ratio.to_le_bytes();
        let mut reader = nuxie_runtime::source::core::binary_reader::BinaryReader::new(&bytes);
        assert_eq!(style.with_mut(|o| o.deserialize(524, &mut reader)), Some(true));
        assert!(reader.reached_end() && !reader.has_error());
        let auto_ratio = name.ends_with("-auto-first") || name.ends_with("-auto-last");
        assert!(LayoutComponent::set_css_ratio_content_box_occurrence(&target, auto_ratio));
        LayoutComponent::sync_style_occurrence(&target);
        LayoutComponent::mark_layout_node_dirty_occurrence(&target, true);
        LayoutComponent::mark_layout_style_dirty_occurrence(&target);
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
        assert!(!LayoutComponent::set_css_ratio_content_box_occurrence(&style, true));
        if auto_ratio {
            let retained = original.instance().unwrap();
            assert!(LayoutComponent::set_css_ratio_content_box_occurrence(&target, false));
            let bare_name = name.replace("-auto-first", "-bare").replace("-auto-last", "-bare");
            let bare = oracle["cases"].as_array().unwrap().iter().find(|c| c["name"] == bare_name).unwrap();
            for (instance, reference_case) in [(original.clone(), bare), (retained, case)] {
                for width in [240., 390., 768., 240.] {
                    instance.set_size(width, 320.);
                    instance.update_pass(true);
                    let reference = reference_case["viewports"].as_array().unwrap().iter()
                        .find(|v| v["width"].as_f64() == Some(width as f64)).unwrap();
                    let owner = instance.with_artboard(|a| a.objects()[node.object_id as usize].clone()).unwrap();
                    let size = owner.with(|o| {
                        let layout = o.as_layout_component().unwrap().layout();
                        [layout.width(), layout.height()]
                    }).unwrap();
                    for (i, axis) in ["width", "height"].iter().enumerate() {
                        let expected = reference["boxes"]["a"][axis].as_f64().unwrap() as f32;
                        assert!(size[i].is_finite() && (size[i] - expected).abs() <= 0.1,
                            "{name} clear/retained {axis} at {width}: {} vs {expected}", size[i]);
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{} coordinate mismatches:\n{}", failures.len(), failures.join("\n"));
}
