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
fn public_realistic_gradients_record_original_and_clone_resize_frames() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!(
        "assets/linear-gradient-realistic-oracle.json"
    ))
    .unwrap();
    assert_eq!(oracle["browser"], "153.0.8010.12");
    let recording =
        std::env::var_os("NUXIE_GRADIENT_REALISTIC_RECORD_DIR").map(std::path::PathBuf::from);
    if let Some(out) = &recording {
        assert!(out.is_absolute(), "Recording directory must be absolute");
        std::fs::create_dir(out).expect("Use a fresh recording directory");
    }
    let native_glyphs = std::env::var("NUXIE_NATIVE_GLYPHS").as_deref() == Ok("1");
    #[cfg(not(all(target_os="macos",feature="native-glyph-controls")))]
    assert!(!native_glyphs,"Native glyph profile requires native-glyph-controls on macOS");
    let mut records = Vec::new();
    let mut frame_count = 0;
    for case in oracle["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        assert_eq!(case["font"], true);
        assert_eq!(case["image"], true);
        let mut input = CompileInput {
            html: case["html"].as_str().unwrap().into(),
            css: case["css"].as_str().unwrap().into(),
            width: 390.,
            height: 320.,
            ..Default::default()
        };
        input.assets.insert("inter".into(), nuxie_html_to_riv::Asset::Font { family:"Inter".into(),weight:400,bytes:include_bytes!("assets/Inter-Regular.ttf").to_vec() });
        input.assets.insert("photo".into(), nuxie_html_to_riv::Asset::Image { bytes:include_bytes!("assets/quadrants.png").to_vec() });
        // Compile exactly once, with the original CSS, before creating either instance.
        let output = compile(&input).unwrap();
        let req = &output.runtime_requirements;
        req.ensure_supported(&[
            RuntimeCapability::LayoutCssAbsolutePositionV1,
            RuntimeCapability::LayoutCssPositionedPaintV1,
            RuntimeCapability::LayoutCssAlignContentV1,
            RuntimeCapability::LayoutCssAlignSelfV1,
            RuntimeCapability::LayoutCssFlexFactorsV1,
            RuntimeCapability::LayoutCssIntrinsicSizingV1,
            RuntimeCapability::TextCssNormalWrapV1,
            RuntimeCapability::TextCssShapingPrecisionV1,
            RuntimeCapability::LayoutCssPixelBoundsV1,
            RuntimeCapability::LayoutCssPaintOrderV1,
            RuntimeCapability::LayoutCssLinearGradientV1,
            RuntimeCapability::LayoutCssSolidBordersV1,
            RuntimeCapability::LayoutCssBorderSidesV1,
            RuntimeCapability::LayoutCssCornerRadiiV1,
            RuntimeCapability::LayoutCssGroupOpacityV1,
            RuntimeCapability::LayoutCssContentBoxV1,
            RuntimeCapability::LayoutCssOverflowClipMarginV1,
        ])
        .unwrap();
        assert_eq!(req.version, 26);
        assert!(!req.layout_linear_gradients.is_empty());
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &output.riv,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .unwrap();
        use nuxie_runtime::source::{assets::font_asset::FontAsset,text::font_hb::ShapingPrecision};
        for asset in file.with_file(|f| f.assets().to_vec()) {
            if asset.with(|a| a.as_any().is::<FontAsset>()).unwrap_or(false) {
                assert!(FontAsset::set_shaping_precision_occurrence(&asset,ShapingPrecision::CssExperimental));
            }
        }
        let original = file.with_file(File::artboard_default).unwrap();
        assert!(Artboard::set_css_absolute_position_policy_occurrence(&original.core_handle(), &req.layout_positioned, &req.layout_absolute));
        assert!(original.with_artboard_mut(|a| a.set_css_positioned_paint_order(&req.layout_positioned)));
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
        use nuxie_runtime::source::css_corner_radii::{CssCornerRadii, CssRadiusValue};
        use nuxie_runtime::source::layout_component::{
            CssOverflowClipBox, CssOverflowClipMargin, LayoutComponent,
        };
        let root = original.core_handle();
        let borders = req
            .layout_borders
            .iter()
            .map(|e| (e.object_id, [e.color; 4]))
            .chain(
                req.layout_border_sides
                    .iter()
                    .map(|e| (e.object_id, e.colors)),
            )
            .collect::<Vec<_>>();
        assert!(Artboard::set_css_border_sides_occurrence(&root, &borders));
        let radii = req
            .layout_corner_radii
            .iter()
            .map(|e| {
                (
                    e.object_id,
                    CssCornerRadii::new(e.radii.map(|pair| {
                        pair.map(|v| match v {
                            nuxie_html_to_riv::CornerRadiusValue::Pixels(v) => {
                                CssRadiusValue::Pixels(v)
                            }
                            nuxie_html_to_riv::CornerRadiusValue::Percent(v) => {
                                CssRadiusValue::Percent(v)
                            }
                        })
                    }))
                    .unwrap(),
                )
            })
            .collect::<Vec<_>>();
        assert!(Artboard::set_css_corner_radii_occurrence(&root, &radii));
        for id in &req.layout_content_box {
            let owner = original
                .with_artboard(|a| a.objects()[*id as usize].clone())
                .unwrap();
            assert!(LayoutComponent::set_css_content_box_occurrence(
                &owner, true
            ));
        }
        let margins = req
            .layout_overflow_clip_margins
            .iter()
            .map(|e| {
                let origin = match e.origin {
                    nuxie_html_to_riv::OverflowClipBox::ContentBox => CssOverflowClipBox::Content,
                    nuxie_html_to_riv::OverflowClipBox::PaddingBox => CssOverflowClipBox::Padding,
                    nuxie_html_to_riv::OverflowClipBox::BorderBox => CssOverflowClipBox::Border,
                };
                (
                    e.object_id,
                    CssOverflowClipMargin::new(origin, e.pixels).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        assert!(Artboard::set_css_overflow_clip_margins_occurrence(
            &root, &margins
        ));
        let alphas = req
            .layout_group_opacity
            .iter()
            .map(|e| (e.object_id, e.opacity))
            .collect::<Vec<_>>();
        assert!(original.with_artboard_mut(|a| a.set_css_group_opacity(&alphas)));
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
    for entry in &req.layout_flex_factors {
        use nuxie_runtime::source::layout_component::{CssFlexFactors, LayoutComponent};
        let factors = CssFlexFactors::new(entry.grow, entry.shrink)
            .expect("valid flex factors");
        let object = original.with_artboard(|a| a.objects()[entry.object_id as usize].clone()).unwrap();
        assert!(LayoutComponent::set_css_flex_factors_occurrence(&object, Some(factors)));
    }

    for entry in &req.layout_align_content {
        use nuxie_html_to_riv::AlignContent;
        use nuxie_runtime::source::layout_component::{CssAlignContent, LayoutComponent};
        let alignment = match entry.alignment {
            AlignContent::FlexStart => CssAlignContent::FlexStart,
            AlignContent::Center => CssAlignContent::Center,
            AlignContent::FlexEnd => CssAlignContent::FlexEnd,
            AlignContent::Stretch => CssAlignContent::Stretch,
            AlignContent::SpaceBetween => CssAlignContent::SpaceBetween,
            AlignContent::SpaceAround => CssAlignContent::SpaceAround,
            AlignContent::SpaceEvenly => CssAlignContent::SpaceEvenly,
        };
        let object = original.with_artboard(|a| a.objects()[entry.object_id as usize].clone()).unwrap();
        assert!(LayoutComponent::set_css_align_content_occurrence(&object, Some(alignment)));
    }
    for entry in &req.layout_align_self {
        use nuxie_html_to_riv::AlignSelf;
        use nuxie_runtime::source::layout_component::{CssAlignSelf, LayoutComponent};
        let alignment = match entry.alignment {
            AlignSelf::Auto => CssAlignSelf::Auto,
            AlignSelf::FlexStart => CssAlignSelf::FlexStart,
            AlignSelf::Center => CssAlignSelf::Center,
            AlignSelf::FlexEnd => CssAlignSelf::FlexEnd,
            AlignSelf::Stretch => CssAlignSelf::Stretch,
            AlignSelf::Baseline => CssAlignSelf::Baseline,
        };
        let object = original.with_artboard(|a| a.objects()[entry.object_id as usize].clone()).unwrap();
        assert!(LayoutComponent::set_css_align_self_occurrence(&object, Some(alignment)));
    }

    for entry in &req.text_policies {
        let object = original
            .with_artboard(|a| a.objects()[entry.object_id as usize].clone())
            .unwrap();
        object.with_mut(|o| {
            let text = o.as_text_mut().unwrap();
            match entry.policy {
                nuxie_html_to_riv::TextPolicy::CssSingleLineEllipsisV1 => text.install_css_single_line_ellipsis().expect("validated ellipsis target"),
                nuxie_html_to_riv::TextPolicy::CssPreWrapV1 => text.set_css_pre_wrap(true),
                nuxie_html_to_riv::TextPolicy::CssPreLineV1 => text.set_css_pre_line(true),
                nuxie_html_to_riv::TextPolicy::CssNormalWrapV1 => text.set_css_normal_wrap(true),
                nuxie_html_to_riv::TextPolicy::CssNowrapAlignmentV1 => text.set_css_nowrap_alignment(true),
            }
        });
    }

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
                #[cfg(all(target_os="macos",feature="native-glyph-controls"))]
                if native_glyphs {
                    let mut cache = nuxie_renderer::glyph_adapter::GlyphCache::new(factory.clone());
                    let mut wrapped = cache.wrap(&mut renderer);
                    assert!(instance.try_draw(&mut wrapped));
                } else { assert!(instance.try_draw(&mut renderer)); }
                #[cfg(not(all(target_os="macos",feature="native-glyph-controls")))]
                assert!(instance.try_draw(&mut renderer));
                assert!(
                    factory.borrow().stream()[start..].contains("makePremultipliedLinearGradient")
                        || factory.borrow().stream()[start..]
                            .contains("makeTiledPremultipliedLinearGradient"),
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
                "sourceMap":output.source_map,"runtimeRequirements":req,"views":views,"font":true,"image":true,"nativeGlyphs":native_glyphs}));
        }
    }
    assert_eq!(frame_count, 48);
    if let Some(out) = &recording {
        std::fs::write(out.join("lifecycle.json"), serde_json::to_vec_pretty(&serde_json::json!({"kind":"gradient-public","nativeGlyphs":native_glyphs,"corpus":"linear-gradient-realistic","browser":oracle["browser"],"cases":records})).unwrap()).unwrap();
    }
}
