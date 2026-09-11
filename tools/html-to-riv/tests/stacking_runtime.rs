use nuxie_html_to_riv::{compile, Asset, CompileInput};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{File, RuntimeFactoryHandle};
use nuxie_runtime::source::artboard::Artboard;

#[test]
fn public_stacking_preserves_chrome_geometry_across_clone_resize_and_clear() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!("assets/stacking-context-oracle.json")).unwrap();
    let composition: serde_json::Value = serde_json::from_str(include_str!("assets/stacking-composition-oracle.json")).unwrap();
    let overlap: serde_json::Value = serde_json::from_str(include_str!("assets/stacking-overlap-oracle.json")).unwrap();
    let mut failures = Vec::new();
    let mut updates = 0;
    for case in oracle["cases"].as_array().unwrap().iter().chain(composition["cases"].as_array().unwrap()).chain(overlap["cases"].as_array().unwrap()) {
        let name = case["name"].as_str().unwrap();
        let mut input = CompileInput {
            html: case["html"].as_str().unwrap().into(), css: case["css"].as_str().unwrap().into(),
            width: 390., height: 320., ..Default::default()
        };
        if case["font"].as_bool() == Some(true) {
            input.assets.insert("inter".into(), Asset::Font {family:"Inter".into(), weight:400,
                bytes:include_bytes!("assets/Inter-Regular.ttf").to_vec()});
        }
        if case["image"].as_bool() == Some(true) {
            input.assets.insert("photo".into(), Asset::Image {bytes:include_bytes!("assets/quadrants.png").to_vec()});
        }
        let output = compile(&input).unwrap();
        let requirements = &output.runtime_requirements;
        requirements.ensure_supported(&requirements.capabilities.iter().copied().collect::<Vec<_>>()).unwrap();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(&output.riv, RuntimeFactoryHandle::from_factory(&mut factory).unwrap(), None, None, None).unwrap();
        let original = file.with_file(File::artboard_default).unwrap();
        let root = original.with_artboard(|a| a.objects()[0].clone()).unwrap();
        assert!(Artboard::set_css_absolute_position_policy_occurrence(&root, &requirements.layout_positioned, &requirements.layout_absolute));
        let contexts = requirements.layout_stacking.iter().map(|entry| (entry.object_id, entry.level)).collect::<Vec<_>>();
        assert!(original.with_artboard_mut(|a| a.set_css_stacking_order(&contexts)));
        let cloned = original.instance().unwrap();
        // Clear/reinstall only stacking, leaving containing-block policies intact.
        // A static flex context must not become an absolute containing block.
        for enabled in [true, false, true] {
            assert!(original.with_artboard_mut(|a| a.set_css_stacking_order(if enabled { &contexts } else { &[] })));
            for (index, instance) in [original.clone(), cloned.clone()].into_iter().enumerate() {
                for width in [240., 390., 768., 240.] {
                    instance.set_size(width, 320.); instance.update_pass(true); updates += 1;
                    let reference = case["viewports"].as_array().unwrap().iter().find(|v| v["width"].as_f64() == Some(width as f64)).unwrap();
                    for source in &output.source_map {
                        let owner = instance.with_artboard(|a| a.objects()[source.object_id as usize].clone()).unwrap();
                        let actual = owner.with(|o| {
                            let layout = o.as_layout_component().unwrap().layout();
                            let transform = o.as_world_transform_component().unwrap().world_transform();
                            [transform[4], transform[5], layout.width(), layout.height()]
                        }).unwrap();
                        for (i, axis) in ["x", "y", "width", "height"].into_iter().enumerate() {
                            let expected = reference["boxes"][&source.id][axis].as_f64().unwrap() as f32;
                            if !actual[i].is_finite() || (actual[i] - expected).abs() > 0.1 {
                                failures.push(format!("{name}, enabled={enabled}, instance={index}, width={width}, {}.{axis}: {} vs {expected}", source.id, actual[i]));
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(updates, 96 * 3 * 2 * 4);
    assert!(failures.is_empty(), "{} geometry failures: {failures:#?}", failures.len());
}
