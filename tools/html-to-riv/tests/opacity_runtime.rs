//! Runtime-only CSS opacity policy; public CSS admission remains gated.
use nuxie_html_to_riv::{compile, CompileInput};
use nuxie_render_api::{PersistentFactory, RecordingFactory, Renderer};
use nuxie_runtime::{File, RuntimeFactoryHandle};
use nuxie_runtime::source::artboard::{Artboard, RuntimeArtboardInstanceHandle};

#[test]
fn opacity_installation_records_balanced_groups_and_preserves_clone_clear_resize() {
    let output = compile(&CompileInput {
        html: "<div id='host'><div id='inner'></div></div>".into(),
        css: "#host { width:60%; height:100px; background:red; } #inner { width:50%; height:40px; background:blue; }".into(),
        width:390., height:320., ..Default::default()
    }).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(&output.riv, RuntimeFactoryHandle::from_factory(&mut factory).unwrap(), None,None,None).unwrap();
    let original = file.with_file(File::artboard_default).unwrap();
    let ids: Vec<_> = ["host", "inner"].iter().map(|name|
        output.source_map.iter().find(|n| n.id == *name).unwrap().object_id).collect();
    let install = |instance: &RuntimeArtboardInstanceHandle, targets: &[(u32, f32)]|
        instance.with_artboard_mut(|a| a.set_css_group_opacity(targets));
    let targets = [(ids[0], 0.5), (ids[1], 0.25)];
    assert!(install(&original, &targets));
    original.update_pass(true);
    let cloned = original.instance().unwrap();
    let boundaries = |instance: &RuntimeArtboardInstanceHandle| {
        instance.update_pass(true);
        let start = factory.borrow().stream().len();
        let mut renderer = factory.borrow().make_renderer();
        assert!(Artboard::try_draw_internal_handle(&instance.core_handle(), &mut renderer));
        factory.borrow().stream()[start..].lines()
            .filter(|s| s.starts_with("beginOpacity") || *s == "endOpacity")
            .map(str::to_owned).collect::<Vec<_>>()
    };
    let expected = ["beginOpacity opacity=0.5", "beginOpacity opacity=0.25", "endOpacity", "endOpacity"];
    assert_eq!(boundaries(&original), expected);
    let style_id = original.with_artboard(|a| a.objects()[ids[0] as usize].clone()).unwrap()
        .with(|o| o.as_layout_component().unwrap().style_id()).unwrap();
    for invalid in [vec![(ids[0],0.),(0,0.)], vec![(ids[0],0.),(u32::MAX,0.)],
        vec![(ids[0],0.),(style_id,0.)], vec![(ids[0],0.),(ids[0],0.5)],
        vec![(ids[0],f32::NAN)], vec![(ids[0],f32::INFINITY)],
        vec![(ids[0],-0.1)], vec![(ids[0],1.1)]] {
        assert!(!install(&original, &invalid));
        assert_eq!(boundaries(&original), expected);
    }
    let mut renderer = factory.borrow().make_renderer();
    for _ in 0..64 { assert!(renderer.begin_opacity_group(1.)); }
    let before = factory.borrow().stream();
    assert!(!original.try_draw(&mut renderer));
    assert_eq!(factory.borrow().stream(), before, "capacity refusal painted part of scene");
    for _ in 0..64 { assert!(renderer.end_opacity_group()); }
    for width in [240.,390.,768.,240.] {
        original.set_size(width,320.); cloned.set_size(width,320.);
        assert!(install(&original,&[]));
        assert!(boundaries(&original).is_empty());
        assert_eq!(boundaries(&cloned),expected);
        assert!(install(&original,&[(ids[0],1.)]));
        assert!(boundaries(&original).is_empty());
        assert!(install(&original,&targets));
        assert_eq!(boundaries(&original),expected);
    }
}

#[test]
fn diagnostic_opacity_initial_chrome_geometry_and_recording() {
    run_opacity_oracle(include_str!("assets/group-opacity-initial-oracle.json"), 12, false);
}

#[test]
fn diagnostic_opacity_stacking_chrome_geometry_and_recording() {
    run_opacity_oracle(include_str!("assets/group-opacity-stacking-oracle.json"), 6, false);
}

#[test]
fn diagnostic_opacity_composition_chrome_geometry_and_recording() {
    run_opacity_oracle(include_str!("assets/group-opacity-composition-oracle.json"), 12, false);
}

#[test]
fn public_opacity_chrome_geometry_and_recording() {
    let mut combined=serde_json::json!({"browser":"153.0.8010.12","cases":[]});
    for source in [include_str!("assets/group-opacity-initial-oracle.json"),
        include_str!("assets/group-opacity-stacking-oracle.json"),
        include_str!("assets/group-opacity-composition-oracle.json")] {
        let oracle:serde_json::Value=serde_json::from_str(source).unwrap();
        assert_eq!(oracle["browser"],combined["browser"]);
        combined["cases"].as_array_mut().unwrap().extend(oracle["cases"].as_array().unwrap().iter().cloned());
    }
    run_opacity_oracle(&combined.to_string(),30,true);
}

#[test]
fn public_opacity_boundary_chrome_geometry_and_recording() {
    run_opacity_oracle(include_str!("assets/group-opacity-boundary-oracle.json"),28,true);
}

#[test]
fn public_opacity_overflow_composition_chrome_geometry_and_recording() {
    run_opacity_oracle(include_str!("assets/group-opacity-overflow-composition-oracle.json"),18,true);
}

#[test]
fn public_opacity_decoration_chrome_geometry_and_recording() {
    run_opacity_oracle(include_str!("assets/group-opacity-decoration-oracle.json"),12,true);
}

fn run_opacity_oracle(source: &str, expected_scenes: usize, public: bool) {
    use nuxie_runtime::source::css_corner_radii::{CssCornerRadii, CssRadiusValue};
    let oracle: serde_json::Value = serde_json::from_str(source).unwrap();
    assert_eq!(oracle["cases"].as_array().unwrap().len(), expected_scenes);
    assert_eq!(oracle["browser"], "153.0.8010.12");
    let recording = std::env::var_os("NUXIE_OPACITY_RECORD_DIR").map(std::path::PathBuf::from);
    if let Some(out) = &recording { std::fs::create_dir(out).unwrap(); }
    let mut records = Vec::new();
    let mut updates = 0;
    for case in oracle["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let authored = case["css"].as_str().unwrap();
        // This fixture-only extractor is intentionally not a second CSS parser.
        // All rules are simple ID selectors and opacity values are literal numbers.
        let mut alphas = std::collections::BTreeMap::new();
        let mut css = String::new();
        if let Some(expected) = case.get("expectedOpacity") {
            assert!(public, "Explicit computed expectations are public-only");
            alphas = serde_json::from_value(expected.clone()).unwrap();
        } else {
        for rule in authored.split('}').filter(|r| !r.is_empty()) {
            let (selector, body) = rule.split_once('{').unwrap();
            css.push_str(selector); css.push('{');
            for declaration in body.split(';').filter(|d| !d.is_empty()) {
                if let Some(alpha) = declaration.strip_prefix("opacity:") {
                    alphas.insert(selector.strip_prefix('#').unwrap().to_owned(), alpha.parse::<f32>().unwrap());
                } else { css.push_str(declaration); css.push(';'); }
            }
            css.push('}');
        }
        }
        if public { css = authored.into(); }
        let mut input = CompileInput { html:case["html"].as_str().unwrap().into(), css:css.clone(),
            width:390.,height:320.,..Default::default() };
        if case["font"].as_bool()==Some(true) {
            input.assets.insert("inter".into(),nuxie_html_to_riv::Asset::Font {family:"Inter".into(),weight:400,
                bytes:include_bytes!("assets/Inter-Regular.ttf").to_vec()});
        }
        if case["image"].as_bool()==Some(true) {
            input.assets.insert("photo".into(),nuxie_html_to_riv::Asset::Image {bytes:include_bytes!("assets/quadrants.png").to_vec()});
        }
        let output = compile(&input).unwrap();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(&output.riv,RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),None,None,None).unwrap();
        if output.runtime_requirements.capabilities.contains(&nuxie_html_to_riv::RuntimeCapability::TextCssShapingPrecisionV1) {
            use nuxie_runtime::source::{assets::font_asset::FontAsset,text::font_hb::ShapingPrecision};
            for asset in file.with_file(|f|f.assets().to_vec()) {
                if asset.with(|a|a.as_any().is::<FontAsset>()).unwrap_or(false) {
                    assert!(FontAsset::set_shaping_precision_occurrence(&asset,ShapingPrecision::CssExperimental));
                }
            }
        }
        let original = file.with_file(File::artboard_default).unwrap();
        let root = original.core_handle();
        let req = &output.runtime_requirements;
        original.with_artboard_mut(|a| a.set_css_paint_order(true));
        assert!(Artboard::set_css_absolute_position_policy_occurrence(&root,&req.layout_positioned,&req.layout_absolute));
        assert!(original.with_artboard_mut(|a| a.set_css_positioned_paint_order(&req.layout_positioned)));
        let levels = req.layout_stacking.iter().map(|e| (e.object_id,e.level)).collect::<Vec<_>>();
        assert!(original.with_artboard_mut(|a| a.set_css_stacking_order(&levels)));
        for id in &req.layout_pixel_bounds {
            let owner=original.with_artboard(|a| a.objects()[*id as usize].clone()).unwrap();
            owner.with_mut(|o| o.as_layout_component_mut().unwrap().set_css_pixel_bounds(true)).unwrap();
        }
        let borders=req.layout_borders.iter().map(|e|(e.object_id,[e.color;4]))
            .chain(req.layout_border_sides.iter().map(|e|(e.object_id,e.colors))).collect::<Vec<_>>();
        assert!(Artboard::set_css_border_sides_occurrence(&root,&borders));
        let radii=req.layout_corner_radii.iter().map(|e| (e.object_id,CssCornerRadii::new(e.radii.map(|pair| pair.map(|v| match v {
            nuxie_html_to_riv::CornerRadiusValue::Pixels(v)=>CssRadiusValue::Pixels(v),
            nuxie_html_to_riv::CornerRadiusValue::Percent(v)=>CssRadiusValue::Percent(v),
        }))).unwrap())).collect::<Vec<_>>();
        assert!(Artboard::set_css_corner_radii_occurrence(&root,&radii));
        use nuxie_runtime::source::layout_component::{LayoutComponent, CssOverflowAxis};
        for id in &req.layout_content_box {
            let owner=original.with_artboard(|a| a.objects()[*id as usize].clone()).unwrap();
            assert!(LayoutComponent::set_css_content_box_occurrence(&owner,true));
        }
        let axes=req.layout_axis_overflow.iter().map(|e| (e.object_id,match e.axis {
            nuxie_html_to_riv::OverflowAxis::X=>CssOverflowAxis::Horizontal,
            nuxie_html_to_riv::OverflowAxis::Y=>CssOverflowAxis::Vertical,
        })).collect::<Vec<_>>();
        assert!(Artboard::set_css_overflow_axes_occurrence(&root,&axes));
        assert!(req.layout_overflow_clip_margins.is_empty());
        for entry in &req.text_underlines {
            use nuxie_html_to_riv::UnderlineSkipInk;
            use nuxie_runtime::source::text::css_decoration::{ResolvedUnderline, SkipInk};
            let lines = entry.lines.iter().map(|line| {
                let skip = match line.skip_ink {
                    UnderlineSkipInk::None => SkipInk::None,
                    UnderlineSkipInk::Auto => SkipInk::Auto,
                    UnderlineSkipInk::All => SkipInk::All,
                };
                ResolvedUnderline::solid(line.color, line.thickness, line.offset, skip).unwrap()
            }).collect();
            let object = original.with_artboard(|a| a.objects()[entry.object_id as usize].clone()).unwrap();
            object.with_mut(|o| o.as_text_mut().unwrap().set_css_underlines(lines)).unwrap();
        }
        for entry in &req.text_strikethroughs {
            use nuxie_runtime::source::text::css_decoration::ResolvedStrikethrough;
            let lines = entry.lines.iter().map(|line| ResolvedStrikethrough::solid(
                line.color, line.thickness, line.offset, line.line_baseline).unwrap()).collect();
            let object = original.with_artboard(|a| a.objects()[entry.object_id as usize].clone()).unwrap();
            object.with_mut(|o| o.as_text_mut().unwrap().set_css_strikethroughs(lines)).unwrap();
        }

        let targets = if public {
            let targets = req.layout_group_opacity.iter().map(|e| (e.object_id,e.opacity)).collect::<Vec<_>>();
            let expected = alphas.iter().filter(|(_,alpha)| **alpha<1.)
                .map(|(id,alpha)| (output.source_map.iter().find(|n| &n.id==id).unwrap().object_id,*alpha))
                .collect::<std::collections::BTreeMap<_,_>>();
            assert_eq!(targets.iter().copied().collect::<std::collections::BTreeMap<_,_>>(),expected);
            req.ensure_supported(&req.capabilities.iter().copied().collect::<Vec<_>>()).unwrap();
            req.ensure_layout_targets(|id| original.with_artboard(|a| a.objects().get(id as usize).cloned())
                .flatten().is_some_and(|o| o.with(|o|o.as_layout_component().is_some()).unwrap_or(false))).unwrap();
            targets
        } else {
            alphas.iter().map(|(id,alpha)| (output.source_map.iter().find(|n| &n.id==id).unwrap().object_id,*alpha)).collect::<Vec<_>>()
        };
        assert!(original.with_artboard_mut(|a| a.set_css_group_opacity(&targets)));
        original.update_pass(true);
        let cloned=original.instance().unwrap();
        let mut views=Vec::new();
        for (instance_index,instance) in [&original,&cloned].into_iter().enumerate() {
            for (step,width) in [240.,390.,768.,240.].into_iter().enumerate() {
                instance.set_size(width,320.); instance.update_pass(true); updates+=1;
                let frame=instance_index*4+step;
                factory.borrow_mut().frame_size(width as u32,320);
                factory.borrow_mut().clear_color(0xffffffff);
                factory.borrow_mut().add_sample(frame as f32);
                let mut renderer=factory.borrow().make_renderer();
                #[cfg(all(target_os="macos",feature="native-glyph-controls"))]
                {
                    let mut cache=nuxie_renderer::glyph_adapter::GlyphCache::new(factory.clone());
                    assert!(instance.try_draw(&mut cache.wrap(&mut renderer)));
                }
                #[cfg(not(all(target_os="macos",feature="native-glyph-controls")))]
                assert!(instance.try_draw(&mut renderer));
                factory.borrow_mut().add_frame();
                let reference=case["viewports"].as_array().unwrap().iter().find(|v| v["width"].as_f64()==Some(width as f64)).unwrap();
                let mut bounds=serde_json::Map::new();
                for node in &output.source_map {
                    let owner=instance.with_artboard(|a| a.objects()[node.object_id as usize].clone()).unwrap();
                    let actual=owner.with(|o| {
                        let layout=o.as_layout_component().unwrap().layout();
                        let transform=o.as_world_transform_component().unwrap().world_transform();
                        [transform[4],transform[5],layout.width(),layout.height()]
                    }).unwrap();
                    for (i,axis) in ["x","y","width","height"].iter().enumerate() {
                        let expected=reference["boxes"][&node.id][axis].as_f64().unwrap();
                        assert!((actual[i] as f64-expected).abs()<0.1,"{name}/{instance_index}/{width}/{}.{axis}: {} != {expected}",node.id,actual[i]);
                    }
                    bounds.insert(node.id.clone(),serde_json::json!({"x":actual[0],"y":actual[1],"width":actual[2],"height":actual[3]}));
                }
                if let Some(out)=&recording {
                    let path=out.join(format!("{name}-{frame}.stream"));
                    std::fs::write(&path,factory.borrow().stream()).unwrap();
                    views.push(serde_json::json!({"instance":instance_index,"frame":frame,"width":width,"height":320,"stream":path,"bounds":bounds}));
                }
            }
        }
        if let Some(out)=&recording {
            std::fs::write(out.join(format!("{name}.riv")),&output.riv).unwrap();
            records.push(serde_json::json!({"name":name,"html":case["html"],"css":authored,"compilerRequestCss":css,
                "runtimeRequirements":serde_json::from_str::<serde_json::Value>(&serde_json::to_string(req).unwrap()).unwrap(),"injectedOpacity":if public {serde_json::Value::Null} else {serde_json::json!(alphas)},"views":views,"font":case["font"].as_bool().unwrap_or(false),"image":case["image"].as_bool().unwrap_or(false)}));
        }
    }
    assert_eq!(updates,expected_scenes*8);
    if let Some(out)=&recording {
        std::fs::write(out.join("lifecycle.json"),serde_json::to_vec_pretty(&serde_json::json!({
            "kind":if public {"opacity-public"} else {"opacity-diagnostic"},"browser":oracle["browser"],"cases":records})).unwrap()).unwrap();
    }
}
