//! Public border lifecycle regressions and retained diagnostic injection controls.
use nuxie_html_to_riv::{compile, CompileInput};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{File, RuntimeFactoryHandle};
use nuxie_runtime::source::layout_component::LayoutComponent;
use nuxie_runtime::source::generated::core_registry::CoreRegistry;
use nuxie_runtime::source::generated::layout::layout_component_style_base::LayoutComponentStyleBase as StyleBase;

#[test]
fn diagnostic_border_layout_matches_chrome_after_clone_and_resize() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!("assets/border-solid-oracle.json")).unwrap();
    assert_eq!(oracle["cases"].as_array().unwrap().len(), 54);
    run_oracle(&oracle, false);
}

#[test]
fn diagnostic_border_transparency_matches_chrome_after_clone_and_resize() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!("assets/border-paint-oracle.json")).unwrap();
    assert_eq!(oracle["cases"].as_array().unwrap().len(), 24);
    run_oracle(&oracle, false);
}

#[test]
fn diagnostic_plain_borders_match_chrome_after_clone_and_resize() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!("assets/border-plain-oracle.json")).unwrap();
    assert_eq!(oracle["cases"].as_array().unwrap().len(), 18);
    run_oracle(&oracle, false);
}

#[test]
fn public_border_emission_matches_all_chrome_geometry_corpora() {
    let mut combined = serde_json::json!({"browser":"153.0.8010.12","cases":[]});
    for source in [include_str!("assets/border-solid-oracle.json"),
        include_str!("assets/border-paint-oracle.json"), include_str!("assets/border-plain-oracle.json")] {
        let oracle: serde_json::Value = serde_json::from_str(source).unwrap();
        assert_eq!(oracle["browser"], combined["browser"]);
        combined["cases"].as_array_mut().unwrap().extend(oracle["cases"].as_array().unwrap().iter().cloned());
    }
    assert_eq!(combined["cases"].as_array().unwrap().len(), 96);
    run_oracle(&combined, true);
}

#[test]
fn public_border_axis_image_lifecycle_matches_chrome() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!("assets/border-axis-margin-oracle.json")).unwrap();
    assert_eq!(oracle["cases"].as_array().unwrap().len(), 24);
    run_oracle(&oracle, true);
}

#[test]
fn public_deferred_border_sizing_lifecycle_matches_chrome() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!("assets/border-deferred-oracle.json")).unwrap();
    assert_eq!(oracle["cases"].as_array().unwrap().len(), 80);
    run_oracle(&oracle, true);
}

#[test]
fn public_border_signed_clip_margin_lifecycle_matches_chrome() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!("assets/border-clip-margin-oracle.json")).unwrap();
    assert_eq!(oracle["cases"].as_array().unwrap().len(), 18);
    run_oracle(&oracle, true);
}

#[test]
fn public_border_text_image_lifecycle_matches_chrome() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!("assets/border-content-oracle.json")).unwrap();
    assert_eq!(oracle["cases"].as_array().unwrap().len(), 24);
    run_oracle(&oracle, true);
}

#[test]
fn diagnostic_individual_border_sides_match_chrome() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!("assets/border-sides-oracle.json")).unwrap();
    assert_eq!(oracle["cases"].as_array().unwrap().len(), 24);
    run_oracle(&oracle, false);
}

#[test]
fn public_individual_border_sides_match_chrome_after_clone_and_resize() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!("assets/border-sides-oracle.json")).unwrap();
    assert_eq!(oracle["cases"].as_array().unwrap().len(), 24);
    run_oracle(&oracle, true);
}

#[test]
fn public_individual_border_edge_cases_match_chrome_after_clone_and_resize() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!("assets/border-sides-edge-oracle.json")).unwrap();
    assert_eq!(oracle["cases"].as_array().unwrap().len(), 40);
    run_oracle(&oracle, true);
}

#[test]
fn public_individual_border_compositions_match_chrome_after_clone_and_resize() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!("assets/border-sides-composition-oracle.json")).unwrap();
    assert_eq!(oracle["cases"].as_array().unwrap().len(), 48);
    run_oracle(&oracle, true);
}

#[test]
fn public_corner_radii_match_chrome_after_clone_and_resize() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!("assets/corner-radii-oracle.json")).unwrap();
    assert_eq!(oracle["cases"].as_array().unwrap().len(), 24);
    run_oracle(&oracle, true);
}

#[test]
fn public_corner_radius_compositions_match_chrome_after_clone_and_resize() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!("assets/corner-radii-composition-oracle.json")).unwrap();
    assert_eq!(oracle["cases"].as_array().unwrap().len(), 36);
    run_oracle(&oracle, true);
}

#[test]
fn public_nested_corner_radii_match_chrome_after_clone_and_resize() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!("assets/corner-radii-nested-oracle.json")).unwrap();
    assert_eq!(oracle["cases"].as_array().unwrap().len(), 16);
    run_oracle(&oracle, true);
}

#[test]
fn experimental_elliptical_radii_match_chrome_after_clone_and_resize() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!("assets/elliptical-radii-diagnostic-oracle.json")).unwrap();
    assert_eq!(oracle["cases"].as_array().unwrap().len(), 16);
    run_oracle(&oracle, true);
}

#[test]
fn public_elliptical_radii_match_chrome_after_clone_and_resize() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!("assets/elliptical-radii-oracle.json")).unwrap();
    assert_eq!(oracle["cases"].as_array().unwrap().len(), 16);
    run_oracle(&oracle, true);
}

#[test]
fn public_elliptical_edge_cases_match_chrome_after_clone_and_resize() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!("assets/elliptical-radii-edge-oracle.json")).unwrap();
    assert_eq!(oracle["cases"].as_array().unwrap().len(), 16);
    run_oracle(&oracle, true);
}

#[test]
fn public_elliptical_compositions_match_chrome_after_clone_and_resize() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!("assets/elliptical-radii-composition-oracle.json")).unwrap();
    assert_eq!(oracle["cases"].as_array().unwrap().len(), 52);
    run_oracle(&oracle, true);
}

fn run_oracle(oracle: &serde_json::Value, public: bool) {
    assert_eq!(oracle["browser"], "153.0.8010.12");
    let recording = std::env::var_os("NUXIE_BORDER_RECORDING").map(std::path::PathBuf::from);
    if let Some(out) = &recording {
        assert!(!out.exists(), "Refusing to overwrite border recording");
        std::fs::create_dir_all(out).unwrap();
    }
    let mut recorded_cases = Vec::new();
    let mut failures = Vec::new();
    let mut updates = 0;
    let ellipse_diagnostic = oracle["cases"].as_array().unwrap().iter().any(|c| c.get("ellipseRadii").is_some());
    assert!(!ellipse_diagnostic || oracle["cases"].as_array().unwrap().iter().all(|c| c.get("ellipseRadii").is_some()));
    for case in oracle["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let border: f32 = case["borderWidth"].as_f64().map(|v|v as f32).unwrap_or_else(||
            name.split("-w").nth(1).unwrap().split('-').next().unwrap().parse().unwrap());
        let color = case["borderColor"].as_u64().map(|v|v as u32).unwrap_or(0xff563cab);
        let source_css = case["css"].as_str().unwrap();
        let compiler_css = if public && !ellipse_diagnostic { source_css }
            else { case["compilerCss"].as_str().unwrap_or(source_css) };
        let side_colors: Option<[u32;4]> = case.get("sideColors").map(|v| serde_json::from_value(v.clone()).unwrap());
        let side_widths: Option<[f32;4]> = case.get("sideWidths").map(|v| serde_json::from_value(v.clone()).unwrap());
        let declaration = case["borderDeclaration"].as_str().map(str::to_owned)
            .unwrap_or_else(|| format!("border:{border}px solid #563cab;"));
        if !public { assert_eq!(compiler_css.matches(&declaration).count(), 1); }
        let mut assets = std::collections::BTreeMap::new();
        if case["font"].as_bool() == Some(true) {
            assets.insert("inter".into(), nuxie_html_to_riv::Asset::Font {
                family:"Inter".into(), weight:400, bytes:include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            });
        }
        if case["image"].as_bool() == Some(true) {
            assets.insert("photo".into(), nuxie_html_to_riv::Asset::Image {
                bytes:include_bytes!("assets/quadrants.png").to_vec(),
            });
        }
        let output = compile(&CompileInput { html:case["html"].as_str().unwrap().into(),
            css:if public { compiler_css.into() } else { compiler_css.replace(&declaration, "") }, width:390., height:320., assets, ..Default::default() }).unwrap();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(&output.riv, RuntimeFactoryHandle::from_factory(&mut factory).unwrap(), None, None, None).unwrap();
        for asset in file.with_file(|f| f.assets().to_vec()) {
            use nuxie_runtime::source::{assets::font_asset::FontAsset, text::font_hb::ShapingPrecision};
            if asset.with(|a| a.as_any().is::<FontAsset>()).unwrap_or(false) {
                assert!(FontAsset::set_shaping_precision_occurrence(&asset, ShapingPrecision::CssExperimental));
            }
        }
        let original = file.with_file(File::artboard_default).unwrap();
        for entry in &output.runtime_requirements.text_policies {
            assert_eq!(entry.policy, nuxie_html_to_riv::TextPolicy::CssNormalWrapV1);
            let object = original.with_artboard(|a| a.objects()[entry.object_id as usize].clone()).unwrap();
            object.with_mut(|o| o.as_text_mut().unwrap().set_css_normal_wrap(true)).unwrap();
        }
        assert!(original.with_artboard_mut(|a| a.set_css_positioned_paint_order(&output.runtime_requirements.layout_positioned)));
        for id in &output.runtime_requirements.layout_content_box {
            let owner = original.with_artboard(|a| a.objects()[*id as usize].clone()).unwrap();
            assert!(LayoutComponent::set_css_content_box_occurrence(&owner, true));
        }
        if public {
            original.with_artboard_mut(|a| a.set_css_paint_order(true));
            for entry in &output.runtime_requirements.layout_flex_factors {
                let owner = original.with_artboard(|a| a.objects()[entry.object_id as usize].clone()).unwrap();
                let factors = nuxie_runtime::source::layout_component::CssFlexFactors::new(entry.grow,entry.shrink).unwrap();
                assert!(LayoutComponent::set_css_flex_factors_occurrence(&owner,Some(factors)));
            }
        }
        let id = output.source_map.iter().find(|n| n.id == case["borderTarget"].as_str().unwrap_or("host")).unwrap().object_id;
        let owner = original.with_artboard(|a| a.objects()[id as usize].clone()).unwrap();
        if public {
            if side_colors.is_some() || border > 0.0 {
                assert_eq!(output.runtime_requirements.version, if !output.runtime_requirements.layout_corner_radii.is_empty() {24} else if side_colors.is_some() {23} else {22});
            }
            output.runtime_requirements.ensure_supported(&[
                nuxie_html_to_riv::RuntimeCapability::TextCssShapingPrecisionV1,
                nuxie_html_to_riv::RuntimeCapability::TextCssNormalWrapV1,
                nuxie_html_to_riv::RuntimeCapability::LayoutCssSolidBordersV1,
                nuxie_html_to_riv::RuntimeCapability::LayoutCssBorderSidesV1,
                nuxie_html_to_riv::RuntimeCapability::LayoutCssCornerRadiiV1,
                nuxie_html_to_riv::RuntimeCapability::LayoutCssContentBoxV1,
                nuxie_html_to_riv::RuntimeCapability::LayoutCssFlexFactorsV1,
                nuxie_html_to_riv::RuntimeCapability::LayoutCssPaintOrderV1,
                nuxie_html_to_riv::RuntimeCapability::LayoutCssPixelBoundsV1,
                nuxie_html_to_riv::RuntimeCapability::LayoutCssPositionedPaintV1,
                nuxie_html_to_riv::RuntimeCapability::LayoutCssAxisOverflowV1,
                nuxie_html_to_riv::RuntimeCapability::LayoutCssOverflowClipMarginV1,
            ]).unwrap();
            if let Some(colors) = side_colors {
                assert!(output.runtime_requirements.layout_borders.is_empty());
                assert_eq!(output.runtime_requirements.layout_border_sides,
                    vec![nuxie_html_to_riv::LayoutBorderSidesRequirement {object_id:id, colors}]);
            } else if border == 0.0 {
                assert!(output.runtime_requirements.layout_borders.is_empty());
                assert!(output.runtime_requirements.layout_border_sides.is_empty());
            } else {
            assert_eq!(output.runtime_requirements.layout_borders,
                vec![nuxie_html_to_riv::LayoutBorderRequirement {object_id:id, color}]);
                assert!(output.runtime_requirements.layout_border_sides.is_empty());
            }
        } else {
        let style = owner.with(|o| o.as_layout_component().unwrap().style_handle().unwrap()).unwrap();
        for (edge_index, (value, units)) in [
            (StyleBase::BORDER_LEFT_PROPERTY_KEY, StyleBase::BORDER_LEFT_UNITS_VALUE_PROPERTY_KEY),
            (StyleBase::BORDER_RIGHT_PROPERTY_KEY, StyleBase::BORDER_RIGHT_UNITS_VALUE_PROPERTY_KEY),
            (StyleBase::BORDER_TOP_PROPERTY_KEY, StyleBase::BORDER_TOP_UNITS_VALUE_PROPERTY_KEY),
            (StyleBase::BORDER_BOTTOM_PROPERTY_KEY, StyleBase::BORDER_BOTTOM_UNITS_VALUE_PROPERTY_KEY),
        ].into_iter().enumerate() {
            let width = side_widths.map(|widths| widths[[3,1,0,2][edge_index]]).unwrap_or(border);
            assert!(CoreRegistry::set_double_handle(&style, value.into(), width));
            assert!(CoreRegistry::set_uint_handle(&style, units.into(), 1));
        }
        }
        if public {
            let requirements = &output.runtime_requirements;
            let targets = requirements.layout_borders.iter().map(|e| (e.object_id,[e.color;4]))
                .chain(requirements.layout_border_sides.iter().map(|e| (e.object_id,e.colors))).collect::<Vec<_>>();
            assert!(nuxie_runtime::source::artboard::Artboard::set_css_border_sides_occurrence(
                &original.core_handle(), &targets));
        } else {
        assert!(nuxie_runtime::source::artboard::Artboard::set_css_borders_occurrence(
            &original.core_handle(), &[(id, color)]));
        if let Some(colors) = side_colors {
            owner.with_mut(|o| o.as_layout_component_mut().unwrap().set_experimental_css_border_side_colors(Some(colors))).unwrap();
            original.with_artboard_mut(|a| a.set_css_paint_order(true));
        }
        }
        if let Some(expected) = case.get("expectedCornerRadii") {
            assert_eq!(output.runtime_requirements.layout_corner_radii.len(),1);
            assert_eq!(output.runtime_requirements.layout_corner_radii[0].object_id,id);
            let expected: [[nuxie_html_to_riv::CornerRadiusValue;2];4] = serde_json::from_value(expected.clone()).unwrap();
            assert_eq!(output.runtime_requirements.layout_corner_radii[0].radii,expected);
        }
        if !output.runtime_requirements.layout_corner_radii.is_empty() {
            use nuxie_runtime::source::css_corner_radii::{CssCornerRadii, CssRadiusValue};
            let targets = output.runtime_requirements.layout_corner_radii.iter().map(|entry| {
                let radii = entry.radii.map(|pair| pair.map(|value| match value {
                    nuxie_html_to_riv::CornerRadiusValue::Pixels(v) => CssRadiusValue::Pixels(v),
                    nuxie_html_to_riv::CornerRadiusValue::Percent(v) => CssRadiusValue::Percent(v),
                }));
                (entry.object_id,CssCornerRadii::new(radii).unwrap())
            }).collect::<Vec<_>>();
            assert!(nuxie_runtime::source::artboard::Artboard::set_css_corner_radii_occurrence(&original.core_handle(),&targets));
        }
        if let Some(values) = case.get("ellipseRadii") {
            use nuxie_runtime::source::css_corner_radii::{CssCornerRadii, CssRadiusValue};
            let mut radii = [[CssRadiusValue::Pixels(0.);2];4];
            for corner in 0..4 { for axis in 0..2 {
                let value = &values[corner][axis];
                radii[corner][axis] = if let Some(pixels) = value["pixels"].as_f64() {
                    CssRadiusValue::Pixels(pixels as f32)
                } else { CssRadiusValue::Percent(value["percent"].as_f64().unwrap() as f32) };
            } }
            assert!(nuxie_runtime::source::artboard::Artboard::set_css_corner_radii_occurrence(
                &original.core_handle(), &[(id, CssCornerRadii::new(radii).unwrap())]));
        }
        if public || side_colors.is_some() {
            for target in &output.runtime_requirements.layout_pixel_bounds {
                let target = original.with_artboard(|a| a.objects()[*target as usize].clone()).unwrap();
                target.with_mut(|o|o.as_layout_component_mut().unwrap().set_css_pixel_bounds(true)).unwrap();
            }
        }
        if public {
            use nuxie_runtime::source::{artboard::Artboard, layout_component::{CssOverflowAxis, CssOverflowClipBox, CssOverflowClipMargin}};
            let axes = output.runtime_requirements.layout_axis_overflow.iter().map(|e|
                (e.object_id,match e.axis { nuxie_html_to_riv::OverflowAxis::X=>CssOverflowAxis::Horizontal, nuxie_html_to_riv::OverflowAxis::Y=>CssOverflowAxis::Vertical })).collect::<Vec<_>>();
            assert!(Artboard::set_css_overflow_axes_occurrence(&original.core_handle(), &axes));
            let margins = output.runtime_requirements.layout_overflow_clip_margins.iter().map(|e| {
                let origin = match e.origin { nuxie_html_to_riv::OverflowClipBox::ContentBox=>CssOverflowClipBox::Content,
                    nuxie_html_to_riv::OverflowClipBox::PaddingBox=>CssOverflowClipBox::Padding,
                    nuxie_html_to_riv::OverflowClipBox::BorderBox=>CssOverflowClipBox::Border };
                (e.object_id,CssOverflowClipMargin::new(origin,e.pixels).unwrap())
            }).collect::<Vec<_>>();
            assert!(Artboard::set_css_overflow_clip_margins_occurrence(&original.core_handle(), &margins));
        }
        original.update_pass(true);
        let cloned = original.instance().unwrap();
        let mut views = Vec::new();
        for (instance_index, instance) in [&original, &cloned].into_iter().enumerate() {
            for (step, width) in [240., 390., 768., 240.].into_iter().enumerate() {
                instance.set_size(width, 320.);
                instance.update_pass(true);
                let frame = instance_index * 4 + step;
                factory.borrow_mut().frame_size(width as u32, 320);
                factory.borrow_mut().clear_color(0xffffffff);
                factory.borrow_mut().add_sample(frame as f32);
                let before = factory.borrow().stream().len();
                let mut renderer = factory.borrow().make_renderer();
                #[cfg(all(target_os = "macos", feature = "native-glyph-controls"))]
                {
                    let mut cache = nuxie_renderer::glyph_adapter::GlyphCache::new(factory.clone());
                    instance.draw(&mut cache.wrap(&mut renderer));
                }
                #[cfg(not(all(target_os = "macos", feature = "native-glyph-controls")))]
                instance.draw(&mut renderer);
                factory.borrow_mut().add_frame();
                let stream = factory.borrow().stream();
                let painted = stream[before..].lines().any(|line| line.starts_with("drawPath ")
                    && line.contains(&format!("{:06x}", color & 0xffffff)));
                if side_colors.is_none() { assert_eq!(painted, border > 0.0 && color >> 24 != 0, "border paint presence for {name}"); }
                updates += 1;
                let reference = case["viewports"].as_array().unwrap().iter()
                    .find(|v| v["width"].as_f64() == Some(width as f64)).unwrap();
                let mut bounds = serde_json::Map::new();
                for node in &output.source_map {
                    let target = instance.with_artboard(|a| a.objects()[node.object_id as usize].clone()).unwrap();
                    let actual = target.with(|o| {
                        let layout = o.as_layout_component().unwrap().layout();
                        let transform = o.as_world_transform_component().unwrap().world_transform();
                        [transform[4], transform[5], layout.width(), layout.height()]
                    }).unwrap();
                    bounds.insert(node.id.clone(), serde_json::json!({"x":actual[0],"y":actual[1],"width":actual[2],"height":actual[3]}));
                    for (index, axis) in ["x", "y", "width", "height"].into_iter().enumerate() {
                        let expected = reference["boxes"][&node.id][axis].as_f64().unwrap() as f32;
                        if !actual[index].is_finite() || (actual[index] - expected).abs() > 0.1 {
                            failures.push(format!("{name} instance{instance_index} width{width} {}.{axis}: {} != {expected}", node.id, actual[index]));
                        }
                    }
                }
                if let Some(out) = &recording {
                    let path = out.join(format!("{name}-{frame}.stream"));
                    std::fs::write(&path, &stream).unwrap();
                    views.push(serde_json::json!({"instance":instance_index,"width":width,"height":320,
                        "frame":frame,"stream":path,"bounds":bounds}));
                }
            }
        }
        if recording.is_some() {
            if public {
                let out = recording.as_ref().unwrap();
                std::fs::write(out.join(format!("{name}.riv")), &output.riv).unwrap();
                std::fs::write(out.join(format!("{name}.requirements.json")), serde_json::to_vec_pretty(&output.runtime_requirements).unwrap()).unwrap();
                recorded_cases.push(serde_json::json!({"name":name,"html":case["html"],"css":source_css,
                    "runtimeRequirements":output.runtime_requirements,"compilerRequestCss":compiler_css,"injectedEllipseRadii":case["ellipseRadii"],"font":case["font"].as_bool().unwrap_or(false),"image":case["image"].as_bool().unwrap_or(false),"views":views}));
            } else {
                recorded_cases.push(serde_json::json!({"name":name,"html":case["html"],"css":source_css,
                    "compilerRequestCss":compiler_css.replace(&declaration, ""),"sideWidths":side_widths,"sideColors":side_colors,"injectedBorderWidth":border,
                    "injectedBorderColor":color,"views":views}));
            }
        }
    }
    if let Some(out) = &recording {
        std::fs::write(out.join("lifecycle.json"), serde_json::to_vec_pretty(&serde_json::json!({
            "kind":if ellipse_diagnostic {"ellipse-diagnostic"} else if public {"border-public"} else {"border-diagnostic"}, "cases":recorded_cases})).unwrap()).unwrap();
    }
    assert_eq!(updates, oracle["cases"].as_array().unwrap().len() * 2 * 4);
    assert!(failures.is_empty(), "{} geometry failures: {failures:#?}", failures.len());
}

#[test]
fn plain_border_installation_is_atomic_and_survives_clone_clear_and_resize() {
    checked_border_installation_lifecycle(false, false);
}

#[test]
fn individual_border_installation_is_atomic_and_survives_clone_clear_and_resize() {
    checked_border_installation_lifecycle(true, false);
}

#[test]
fn equal_color_border_sides_share_one_paint_and_preserve_clone_clear() {
    checked_border_installation_lifecycle(true, true);
}

fn checked_border_installation_lifecycle(individual: bool, grouped: bool) {
    use nuxie_runtime::source::artboard::Artboard;
    let output = compile(&CompileInput {
        html: "<div id='host'><div id='inner'></div></div>".into(),
        css: "#host { width:60%; height:100px; } #inner { width:50%; height:40px; }".into(),
        width:390., height:320., ..Default::default()
    }).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(&output.riv, RuntimeFactoryHandle::from_factory(&mut factory).unwrap(), None,None,None).unwrap();
    let original = file.with_file(File::artboard_default).unwrap();
    let root = original.core_handle();
    let ids: Vec<_> = ["host", "inner"].iter().map(|name|
        output.source_map.iter().find(|n| n.id == *name).unwrap().object_id).collect();
    for id in &ids {
        let owner = original.with_artboard(|a| a.objects()[*id as usize].clone()).unwrap();
        let style = owner.with(|o| o.as_layout_component().unwrap().style_handle().unwrap()).unwrap();
        for (value, units) in [
            (StyleBase::BORDER_LEFT_PROPERTY_KEY, StyleBase::BORDER_LEFT_UNITS_VALUE_PROPERTY_KEY),
            (StyleBase::BORDER_RIGHT_PROPERTY_KEY, StyleBase::BORDER_RIGHT_UNITS_VALUE_PROPERTY_KEY),
            (StyleBase::BORDER_TOP_PROPERTY_KEY, StyleBase::BORDER_TOP_UNITS_VALUE_PROPERTY_KEY),
            (StyleBase::BORDER_BOTTOM_PROPERTY_KEY, StyleBase::BORDER_BOTTOM_UNITS_VALUE_PROPERTY_KEY),
        ] {
            assert!(CoreRegistry::set_double_handle(&style, value.into(), 8.));
            assert!(CoreRegistry::set_uint_handle(&style, units.into(), 1));
        }
    }
    let side_colors = if grouped { [0xff563cab, 0xff123456, 0xff563cab, 0xff563cab] }
        else { [0xff563cab, 0xff123456, 0xffabcdef, 0xff654321] };
    let install = |root: &nuxie_runtime::CoreHandle, targets: &[(u32, u32)]| {
        if individual {
            Artboard::set_css_border_sides_occurrence(root,
                &targets.iter().map(|&(id, _)| (id, side_colors)).collect::<Vec<_>>())
        } else {
            Artboard::set_css_borders_occurrence(root, targets)
        }
    };
    let expected_draws = if grouped { 4 } else if individual { 8 } else { 2 };
    let targets = [(ids[0], 0xff563cab), (ids[1], 0xff563cab)];
    let owner = original.with_artboard(|a| a.objects()[ids[0] as usize].clone()).unwrap();
    let style_id = owner.with(|o| o.as_layout_component().unwrap().style_id()).unwrap();
    assert!(!install(&owner, &targets));
    assert!(install(&root, &targets));
    original.update_pass(true);
    let clone = original.instance().unwrap();
    let draw_count = |instance: &nuxie_runtime::source::artboard::RuntimeArtboardInstanceHandle| {
        instance.update_pass(true);
        let start = factory.borrow().stream().len();
        let mut renderer = factory.borrow().make_renderer();
        instance.draw(&mut renderer);
        factory.borrow().stream()[start..].lines().filter(|l|
            l.starts_with("drawPath ") && ["563cab", "123456", "abcdef", "654321"].iter().any(|color| l.contains(color))).count()
    };
    for invalid in [vec![(ids[0], 0), (0, 0)], vec![(ids[0], 0), (u32::MAX, 0)],
        vec![(ids[0], 0), (style_id, 0)], vec![(ids[0], 0), (ids[0], 0)]] {
        assert!(!install(&root, &invalid));
        assert_eq!(draw_count(&original), expected_draws, "invalid installation mutated existing borders");
    }
    for width in [240., 390., 768., 240.] {
        original.set_size(width, 320.);
        clone.set_size(width, 320.);
        assert!(install(&root, &[]));
        assert_eq!(draw_count(&original), 0);
        assert_eq!(draw_count(&clone), expected_draws);
        for _ in 0..2 {
            assert!(install(&root, &targets));
            assert_eq!(draw_count(&original), expected_draws, "missing or duplicated proxy");
        }
    }
    if individual {
        assert!(Artboard::set_css_borders_occurrence(&root, &targets));
        assert_eq!(draw_count(&original), 2, "uniform replacement retained individual paints");
        assert_eq!(draw_count(&clone), expected_draws, "replacement changed the independent clone");
        assert!(install(&root, &targets));
        assert_eq!(draw_count(&original), expected_draws);
        assert!(Artboard::set_css_borders_occurrence(&root, &[]));
        assert_eq!(draw_count(&original), 0, "uniform clear retained side policy");
    }
}

#[test]
fn corner_installation_is_atomic_and_preserves_clone_and_imported_radii() {
    use nuxie_runtime::source::artboard::{Artboard, RuntimeArtboardInstanceHandle};
    use nuxie_runtime::source::css_corner_radii::{CssCornerRadii, CssRadiusValue::Percent};
    let output = compile(&CompileInput {
        html: "<div id='host'><div id='inner'></div></div>".into(),
        css: "#host { width:60%; height:100px; overflow:clip; border-radius:7px; } #inner { width:50%; height:40px; overflow:clip; border-radius:3px; }".into(),
        width:390., height:320., ..Default::default()
    }).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(&output.riv, RuntimeFactoryHandle::from_factory(&mut factory).unwrap(), None,None,None).unwrap();
    let original = file.with_file(File::artboard_default).unwrap();
    let root = original.core_handle();
    let ids: Vec<_> = ["host", "inner"].iter().map(|name|
        output.source_map.iter().find(|n| n.id == *name).unwrap().object_id).collect();
    let owner = original.with_artboard(|a| a.objects()[ids[0] as usize].clone()).unwrap();
    let style_id = owner.with(|o| o.as_layout_component().unwrap().style_id()).unwrap();
    let value = CssCornerRadii::new([[Percent(25.);2];4]).unwrap();
    let replacement = CssCornerRadii::new([[Percent(10.);2];4]).unwrap();
    let targets = [(ids[0],value), (ids[1],value)];
    let endpoints = |instance: &RuntimeArtboardInstanceHandle| {
        instance.update_pass(true);
        ids.iter().map(|id| {
            let object = instance.with_artboard(|a| a.objects()[*id as usize].clone()).unwrap();
            object.with_mut(|o| o.as_layout_component_mut().unwrap()
                .local_path().unwrap().raw_path().points()[0]).unwrap()
        }).collect::<Vec<_>>()
    };
    let imported = endpoints(&original);
    assert!(!Artboard::set_css_corner_radii_occurrence(&owner, &targets));
    assert!(Artboard::set_css_corner_radii_occurrence(&root, &targets));
    let installed = endpoints(&original);
    assert_ne!(installed, imported);
    let clone = original.instance().unwrap();
    for invalid in [0, u32::MAX, style_id, ids[0]] {
        assert!(!Artboard::set_css_corner_radii_occurrence(&root,
            &[(ids[0],replacement), (invalid,replacement)]));
        assert_eq!(endpoints(&original), installed, "rejection partially mutated radii");
    }
    for width in [240.,390.,768.,240.] {
        original.set_size(width,320.);
        clone.set_size(width,320.);
        assert!(Artboard::set_css_corner_radii_occurrence(&root, &[(ids[0],value)]));
        let partial = endpoints(&original);
        let retained = endpoints(&clone);
        assert_eq!(partial[0],retained[0]);
        assert_eq!(partial[1],imported[1], "omitted layout did not revert");
        assert_ne!(partial[1],retained[1]);
        assert!(Artboard::set_css_corner_radii_occurrence(&root, &[]));
        assert_eq!(endpoints(&original),imported);
        assert_eq!(endpoints(&clone),retained);
        assert!(Artboard::set_css_corner_radii_occurrence(&root, &targets));
        assert_eq!(endpoints(&original),retained);
    }
}
