use nuxie_html_to_riv::{compile, Asset, CompileInput, RuntimeCapability, TextPolicy};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::layout_component::{CssFlexFactors, LayoutComponent};
use nuxie_runtime::{File, RuntimeFactoryHandle};

// Public compiler output, including validated occurrence policies.
#[test]
fn aspect_ratio_public_compiler_matches_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/aspect-ratio-initial-oracle.json"),
        include_str!("assets/aspect-ratio-auto-probe-oracle.json"),
        include_str!("assets/aspect-ratio-expanded-oracle.json")]);
}

#[test]
fn aspect_ratio_text_matches_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/aspect-ratio-text-oracle.json")]);
}

#[test]
fn cloned_intrinsic_footer_matches_chrome_after_resize() {
    check_oracles(&[include_str!("assets/aspect-ratio-clone-minimal-oracle.json")]);
}

#[test]
fn cloned_intrinsic_single_label_matches_chrome_after_resize() {
    check_oracles(&[include_str!("assets/aspect-ratio-clone-single-label-oracle.json")]);
}

#[test]
fn aspect_ratio_images_match_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/aspect-ratio-image-oracle.json")]);
}

#[test]
fn aspect_ratio_image_flex_stress_matches_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/aspect-ratio-image-stress-oracle.json")]);
}

#[test]
fn aspect_ratio_finite_math_matches_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/aspect-ratio-math-oracle.json")]);
}

#[test]
fn aspect_ratio_component_boundaries_match_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/aspect-ratio-clamp-oracle.json")]);
}

#[test]
fn aspect_ratio_approximations_match_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/aspect-ratio-approximation-oracle.json")]);
}

#[test]
fn aspect_ratio_nonfinite_values_match_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/aspect-ratio-nonfinite-oracle.json")]);
}

#[test]
fn aspect_ratio_saturation_matches_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/aspect-ratio-saturation-oracle.json")]);
}

#[test]
fn aspect_ratio_saturation_stress_matches_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/aspect-ratio-saturation-stress-oracle.json")]);
}

#[test]
fn aspect_ratio_cascade_matches_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/aspect-ratio-cascade-oracle.json")]);
}

#[test]
fn aspect_ratio_typed_math_matches_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/aspect-ratio-typed-oracle.json")]);
}

#[test]
fn aspect_ratio_unit_conversion_order_matches_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/aspect-ratio-unit-order-oracle.json")]);
}

#[test]
fn aspect_ratio_fractional_source_sizes_match_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/aspect-ratio-rounding-oracle.json")]);
}

#[test]
fn aspect_ratio_fractional_padding_and_flex_match_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/aspect-ratio-rounding-stress-oracle.json")]);
}

#[test]
fn aspect_ratio_nested_gap_allocation_matches_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/aspect-ratio-nested-gap-oracle.json")]);
}

#[test]
fn aspect_ratio_font_math_matches_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/aspect-ratio-font-math-oracle.json")]);
}

#[test]
fn aspect_ratio_font_boundaries_match_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/aspect-ratio-font-boundary-oracle.json")]);
}

fn check_oracles(oracles: &[&str]) {
    check_oracles_with_tolerance(oracles, 0.1);
}

fn check_oracles_with_tolerance(oracles: &[&str], tolerance: f32) {
    assert!(tolerance > 0.0 && tolerance <= 0.1);
    let mut cases = Vec::new();
    for text in oracles {
        let oracle: serde_json::Value = serde_json::from_str(text).unwrap();
        cases.extend(oracle["cases"].as_array().unwrap().iter().cloned());
    }
    let mut failures = Vec::new();
    for case in &cases {
        let output = compile(&CompileInput {
            html: case["html"].as_str().unwrap().into(),
            css: case["css"].as_str().unwrap().into(),
            width: 390., height: 320.,
            assets: {
                let mut assets = std::collections::BTreeMap::new();
                if case["font"].as_bool() == Some(true) {
                    assets.insert("inter".into(), Asset::Font { family: "Inter".into(), weight: 400,
                        bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec() });
                }
                if case["image"].as_bool() == Some(true) {
                    assets.insert("photo".into(), Asset::Image {
                        bytes: match case["imageAsset"].as_str() {
                            Some("intrinsic-wide.png") => include_bytes!("assets/intrinsic-wide.png").as_slice(),
                            Some("intrinsic-tall.png") => include_bytes!("assets/intrinsic-tall.png").as_slice(),
                            None => include_bytes!("assets/quadrants.png").as_slice(),
                            Some(name) => panic!("Unknown image fixture: {name}"),
                        }.to_vec() });
                }
                assets
            },
            ..Default::default()
        }).unwrap();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(&output.riv,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None, None, None).unwrap();
        if output.runtime_requirements.capabilities.contains(&RuntimeCapability::TextCssShapingPrecisionV1) {
            use nuxie_runtime::source::{assets::font_asset::FontAsset, text::font_hb::ShapingPrecision};
            for asset in file.with_file(|f| f.assets().to_vec()) {
                if asset.with(|a| a.as_any().is::<FontAsset>()).unwrap_or(false) {
                    assert!(FontAsset::set_shaping_precision_occurrence(&asset, ShapingPrecision::CssExperimental));
                }
            }
        }
        let original = file.with_file(File::artboard_default).unwrap();
        for id in &output.runtime_requirements.layout_content_box {
            let target = original.with_artboard(|a| a.objects()[*id as usize].clone()).unwrap();
            assert!(LayoutComponent::set_css_content_box_occurrence(&target, true));
        }
        for entry in &output.runtime_requirements.layout_aspect_ratios {
            let target = original.with_artboard(|a| a.objects()[entry.object_id as usize].clone()).unwrap();
            assert!(LayoutComponent::set_css_ratio_content_box_occurrence(&target, entry.content_box));
            assert!(LayoutComponent::set_css_ratio_pair_occurrence(&target, entry.pair));
        }
        let requirements = &output.runtime_requirements;
        requirements.ensure_supported(&requirements.capabilities.iter().copied().collect::<Vec<_>>()).unwrap();
        requirements.ensure_layout_targets(|id| output.source_map.iter().any(|n| n.object_id == id)).unwrap();
        requirements.ensure_text_targets(|id| {
            original.with_artboard(|a| a.objects().get(id as usize).cloned().flatten())
                .and_then(|object| object.with_mut(|o| o.as_text_mut().is_some())).unwrap_or(false)
        }).unwrap();
        for id in &requirements.layout_pixel_bounds {
            let target = original.with_artboard(|a| a.objects()[*id as usize].clone()).unwrap();
            target.with_mut(|o| o.as_layout_component_mut().unwrap().set_css_pixel_bounds(true)).unwrap();
        }
        if !requirements.layout_absolute.is_empty() {
            let root = original.with_artboard(|a| a.objects()[0].clone()).unwrap();
            assert!(nuxie_runtime::source::artboard::Artboard::set_css_absolute_position_policy_occurrence(
                &root, &requirements.layout_positioned, &requirements.layout_absolute));
        }
        if !requirements.layout_positioned.is_empty() {
            assert!(original.with_artboard_mut(|a| a.set_css_positioned_paint_order(&requirements.layout_positioned)));
        }
        if requirements.capabilities.contains(&RuntimeCapability::LayoutCssPaintOrderV1) {
            original.with_artboard_mut(|a| a.set_css_paint_order(true));
        }
        for entry in &requirements.text_policies {
            let target = original.with_artboard(|a| a.objects()[entry.object_id as usize].clone()).unwrap();
            target.with_mut(|o| {
                let text = o.as_text_mut().unwrap();
                match entry.policy {
                    TextPolicy::CssNormalWrapV1 => text.set_css_normal_wrap(true),
                    TextPolicy::CssSingleLineEllipsisV1 => {
                        text.validate_css_single_line_ellipsis().unwrap();
                        text.install_css_single_line_ellipsis().unwrap();
                    }
                    _ => panic!("oracle needs an installer for {:?}", entry.policy),
                }
            }).unwrap();
        }
        for entry in &requirements.layout_flex_factors {
            let target = original.with_artboard(|a| a.objects()[entry.object_id as usize].clone()).unwrap();
            assert!(LayoutComponent::set_css_flex_factors_occurrence(
                &target, CssFlexFactors::new(entry.grow, entry.shrink)));
        }
    for entry in &requirements.layout_justify_content {
        use nuxie_html_to_riv::JustifyDistribution;
        use nuxie_runtime::source::layout_component::CssJustifyContent;
        let alignment = match entry.alignment {
            JustifyDistribution::SpaceAround => CssJustifyContent::SpaceAround,
            JustifyDistribution::SpaceEvenly => CssJustifyContent::SpaceEvenly,
        };
        let object = original.with_artboard(|a| a.objects()[entry.object_id as usize].clone()).unwrap();
        assert!(LayoutComponent::set_css_justify_content_occurrence(&object, Some(alignment)));
    }
    for id in &requirements.layout_percentage_spacing {
        let object = original.with_artboard(|a| a.objects()[*id as usize].clone()).unwrap();
        assert!(LayoutComponent::set_css_percentage_spacing_occurrence(&object, true));
    }
    for entry in &requirements.layout_align_content {
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
        if !LayoutComponent::set_css_align_content_occurrence(&object, Some(alignment)) {
            panic!("invalid-layout-align-content-target");
        }
    }
    for entry in &requirements.layout_align_self {
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
        if !LayoutComponent::set_css_align_self_occurrence(&object, Some(alignment)) {
            panic!("invalid-layout-align-self-target");
        }
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
                        // Match the native probe and Chrome's bounding-rect
                        // measurement, including endpoint precision for huge
                        // extents offset from the origin. Raw layout size is
                        // a different quantity at those f32 boundaries.
                        let bounds = layout.layout_bounds();
                        [transform[4], transform[5], bounds.width(), bounds.height()]
                    }).unwrap();
                    for (index, axis) in ["x", "y", "width", "height"].iter().enumerate() {
                        let expected = reference["boxes"][&node.id][axis].as_f64().unwrap() as f32;
                        if !actual[index].is_finite() || (actual[index] - expected).abs() > tolerance {
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


#[test]
fn aspect_ratio_exact_pair_stress_matches_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/aspect-ratio-pair-stress-oracle.json")]);
}


#[test]
fn intrinsic_images_match_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/intrinsic-image-initial-oracle.json"),
        include_str!("assets/intrinsic-image-nonsquare-oracle.json")]);
}


#[test]
fn intrinsic_image_cards_match_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/intrinsic-image-card-oracle.json")]);
}

#[test]
fn intrinsic_image_flex_matches_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/intrinsic-image-flex-oracle.json")]);
}

#[test]
fn intrinsic_image_stretch_edges_match_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/intrinsic-image-stretch-edge-oracle.json")]);
}

#[test]
fn percentage_spacing_matches_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/percentage-spacing-initial-oracle.json")]);
}

#[test]
fn percentage_spacing_roots_match_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/percentage-spacing-root-oracle.json")]);
}

#[test]
fn percentage_spacing_compositions_match_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/percentage-spacing-composition-oracle.json")]);
}

#[test]
fn percentage_spacing_reverse_compositions_match_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/percentage-spacing-reverse-composition-oracle.json")]);
}

#[test]
fn percentage_spacing_minimal_matches_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/percentage-spacing-minimal-oracle.json")]);
}

#[test]
fn percentage_spacing_rounding_matches_chrome_before_pixel_snapping() {
    check_oracles_with_tolerance(&[include_str!("assets/percentage-spacing-rounding-oracle.json")], 0.001);
}

#[test]
fn percentage_spacing_interactions_match_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/percentage-spacing-interaction-oracle.json")]);
}

#[test]
fn percentage_spacing_basis_reduced_matches_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/percentage-spacing-basis-reduced-oracle.json")]);
}

#[test]
fn negative_margins_match_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/negative-margin-oracle.json")]);
}

#[test]
fn negative_margin_compositions_match_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/negative-margin-composition-oracle.json")]);
}

#[test]
fn negative_margin_interactions_match_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/negative-margin-interaction-oracle.json")]);
}

#[test]
fn relative_position_public_compiler_matches_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/relative-position-oracle.json")]);
}

#[test]
fn relative_position_compositions_match_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/relative-position-composition-oracle.json")]);
}

#[test]
fn relative_position_interactions_match_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/relative-position-interaction-oracle.json")]);
}

#[test]
fn absolute_position_public_compiler_matches_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/absolute-position-oracle.json"),
        include_str!("assets/absolute-position-containing-block-oracle.json")]);
}

#[test]
fn absolute_position_compositions_match_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/absolute-position-composition-oracle.json")]);
}

#[test]
fn absolute_position_interactions_match_chrome_after_clone_and_resize() {
    check_oracles(&[include_str!("assets/absolute-position-interaction-oracle.json")]);
}
