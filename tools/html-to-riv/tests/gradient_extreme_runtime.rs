//! Public compilation is the only source of runtime paint and policy in this test.
use nuxie_html_to_riv::{
    compile, CompileInput, LinearGradientDirection, LinearGradientPosition, RuntimeCapability,
};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    artboard::Artboard,
    css_linear_gradient::{CssGradientDirection, CssGradientPosition, CssLinearGradient},
};
use nuxie_runtime::{File, RuntimeFactoryHandle};

#[test]
fn extreme_public_gradients_record_original_and_clone_resize_frames() {
    let cases: Vec<_> = [
        ("positive", "red 0%, blue 3e38%"),
        ("negative", "red -3e38%, blue 100%"),
        ("symmetric-alpha", "rgba(255,0,0,.25) -3e38%, rgba(0,0,255,.75) 3e38%"),
        ("hard-interior", "red -3e38%, red 50%, blue 50%, blue 3e38%"),
    ].into_iter().map(|(name, stops)| serde_json::json!({
        "name": name, "html": "<div id=\"gradient\"></div>",
        "css": format!("#gradient{{width:100%;height:140px;background:linear-gradient(to right,{stops});}}"),
        "viewports": ([240,390,768].map(|width| serde_json::json!({"width":width,
            "boxes":{"gradient":{"x":0,"y":0,"width":width,"height":140}}})))
    })).collect();
    // Analytic rectangle expectations; independent browser captures are produced
    // by validation/validate-extreme-gradient-native.mjs.
    let fixtures = serde_json::json!({"cases":cases});
    let recording =
        std::env::var_os("NUXIE_GRADIENT_EXTREME_RECORD_DIR").map(std::path::PathBuf::from);
    if let Some(out) = &recording {
        assert!(out.is_absolute(), "Recording directory must be absolute");
        std::fs::create_dir(out).expect("Use a fresh recording directory");
    }
    let mut records = Vec::new();
    let mut frame_count = 0;
    for case in fixtures["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let input = CompileInput {
            html: case["html"].as_str().unwrap().into(),
            css: case["css"].as_str().unwrap().into(),
            width: 390.,
            height: 320.,
            ..Default::default()
        };
        // Compile exactly once, with the original CSS, before creating either instance.
        let output = compile(&input).unwrap();
        let req = &output.runtime_requirements;
        req.ensure_supported(&[
            RuntimeCapability::LayoutCssPixelBoundsV1,
            RuntimeCapability::LayoutCssPaintOrderV1,
            RuntimeCapability::LayoutCssLinearGradientV1,
        ])
        .unwrap();
        assert_eq!(req.version, 26);
        assert_eq!(req.layout_linear_gradients.len(), 1);
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
        req.ensure_layout_targets(|id| {
            original
                .with_artboard(|a| a.objects().get(id as usize).cloned())
                .flatten()
                .is_some_and(|o| {
                    o.with(|o| o.as_layout_component().is_some())
                        .unwrap_or(false)
                })
        })
        .unwrap();
        original.with_artboard_mut(|a| {
            a.set_css_paint_order(
                req.capabilities
                    .contains(&RuntimeCapability::LayoutCssPaintOrderV1),
            )
        });
        for id in &req.layout_pixel_bounds {
            original
                .with_artboard(|a| a.objects()[*id as usize].clone())
                .unwrap()
                .with_mut(|o| {
                    o.as_layout_component_mut()
                        .unwrap()
                        .set_css_pixel_bounds(true)
                })
                .unwrap();
        }
        let gradients = req
            .layout_linear_gradients
            .iter()
            .map(|gradient| {
                let direction = match gradient.direction {
                    LinearGradientDirection::Degrees(v) => CssGradientDirection::Degrees(v),
                    LinearGradientDirection::Corner { right, bottom } => {
                        CssGradientDirection::Corner { right, bottom }
                    }
                };
                let colors = gradient.stops.iter().map(|stop| stop.color).collect();
                let positions = gradient
                    .stops
                    .iter()
                    .map(|stop| {
                        stop.position.map(|position| match position {
                            LinearGradientPosition::Pixels(v) => CssGradientPosition::Pixels(v),
                            LinearGradientPosition::Percent(v) => CssGradientPosition::Percent(v),
                        })
                    })
                    .collect();
                (
                    gradient.object_id,
                    CssLinearGradient::new(direction, colors, positions).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        assert!(Artboard::set_css_linear_gradients_occurrence(
            &original.core_handle(),
            &gradients
        ));
        original.update_pass(true);
        let cloned = original.instance().unwrap();
        let mut views = Vec::new();
        for (instance_index, instance) in [&original, &cloned].into_iter().enumerate() {
            for (step, width) in [240., 390., 768., 240.].into_iter().enumerate() {
                let frame = instance_index * 4 + step;
                instance.set_size(width, 320.);
                instance.update_pass(true);
                factory.borrow_mut().frame_size(width as u32, 320);
                factory.borrow_mut().clear_color(0xffffffff);
                factory.borrow_mut().add_sample(frame as f32);
                let start = factory.borrow().stream().len();
                let mut renderer = factory.borrow().make_renderer();
                assert!(instance.try_draw(&mut renderer));
                assert!(
                    factory.borrow().stream()[start..].contains("makePremultipliedLinearGradient"),
                    "{name}/{frame}: gradient missing"
                );
                factory.borrow_mut().add_frame();
                frame_count += 1;
                let reference = case["viewports"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|v| v["width"].as_f64() == Some(width as f64))
                    .unwrap();
                let mut bounds = serde_json::Map::new();
                for node in &output.source_map {
                    let owner = instance
                        .with_artboard(|a| a.objects()[node.object_id as usize].clone())
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
                        let expected = reference["boxes"][&node.id][axis].as_f64().unwrap();
                        assert!(
                            (actual[i] as f64 - expected).abs() < 0.1,
                            "{name}/{instance_index}/{step}/{}.{axis}: {} != {expected}",
                            node.id,
                            actual[i]
                        );
                    }
                    bounds.insert(node.id.clone(), serde_json::json!({"x":actual[0],"y":actual[1],"width":actual[2],"height":actual[3]}));
                }
                assert_eq!(bounds.len(), reference["boxes"].as_object().unwrap().len());
                if let Some(out) = &recording {
                    let stream = out.join(format!("{name}-{frame}.stream"));
                    std::fs::write(&stream, factory.borrow().stream()).unwrap();
                    views.push(serde_json::json!({"instance":instance_index,"frame":frame,"width":width,"height":320,"stream":stream,"bounds":bounds}));
                }
            }
        }
        if let Some(out) = &recording {
            std::fs::write(out.join(format!("{name}.riv")), &output.riv).unwrap();
            std::fs::write(
                out.join(format!("{name}.request.json")),
                serde_json::to_vec_pretty(&input).unwrap(),
            )
            .unwrap();
            records.push(serde_json::json!({"name":name,"html":input.html,"css":input.css,"compileViewport":{"width":390,"height":320},
                "sourceMap":output.source_map,"runtimeRequirements":req,"views":views,"font":false,"image":false}));
        }
    }
    assert_eq!(frame_count, 32);
    if let Some(out) = &recording {
        std::fs::write(out.join("lifecycle.json"), serde_json::to_vec_pretty(&serde_json::json!({"kind":"gradient-public","geometryReference":"analytic","cases":records})).unwrap()).unwrap();
    }
}
