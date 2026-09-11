//! Validation adapter: imports actual compiler output, resizes the SAME scene,
//! observes its native layout and records native draw calls for renderer-replay.
use nuxie_html_to_riv::{RuntimeCapability, RuntimeRequirements, SourceNode, TextPolicy};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{File, RuntimeFactoryHandle};
use serde_json::json;
use std::{error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(not(all(target_os = "macos", feature = "native-glyph-controls")))]
    if std::env::var("NUXIE_NATIVE_GLYPHS").as_deref() == Ok("1") {
        return Err(
            "Native glyph profile requires macOS and --features native-glyph-controls".into(),
        );
    }
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 5 && args.len() != 6 {
        return Err(
            "probe <scene.riv> <map.json> <width> <height> <output-prefix> [host-state.json]"
                .into(),
        );
    }
    if args.len() == 6 && std::env::var("NUXIE_NATIVE_GLYPHS").as_deref() != Ok("1") {
        return Err("Host-state controls require the explicit native glyph profile".into());
    }
    let map: Vec<SourceNode> = serde_json::from_slice(&fs::read(&args[1])?)?;
    let width: f32 = args[2].parse()?;
    let height: f32 = args[3].parse()?;
    let device_scale: f32 = std::env::var("NUXIE_DEVICE_SCALE")
        .unwrap_or_else(|_| "1".into())
        .parse()?;
    if !device_scale.is_finite() || !(1.0..=4.0).contains(&device_scale) {
        return Err("Device scale must be finite and between 1 and 4".into());
    }
    if device_scale != 1.0 && std::env::var("NUXIE_NATIVE_GLYPHS").as_deref() != Ok("1") {
        return Err("Device scale controls require the explicit native glyph profile".into());
    }
    let requirements: RuntimeRequirements = serde_json::from_slice(&fs::read(
        std::path::Path::new(&args[0]).with_extension("requirements.json"),
    )?)?;
    let supported = if std::env::var("NUXIE_DISABLE_CLUSTER_SPACING").as_deref() == Ok("1") {
        &[RuntimeCapability::TextCssShapingPrecisionV1][..]
    } else {
        &[
            RuntimeCapability::LayoutCssLinearGradientV1,
            RuntimeCapability::LayoutCssGroupOpacityV1,
            RuntimeCapability::LayoutCssCornerRadiiV1,
            RuntimeCapability::LayoutCssSolidBordersV1,
            RuntimeCapability::LayoutCssBorderSidesV1,
            RuntimeCapability::LayoutCssOverflowClipMarginV1,
            RuntimeCapability::LayoutCssAlignSelfV1,
            RuntimeCapability::LayoutCssAlignContentV1,
            RuntimeCapability::LayoutCssDistributedSpacingV1,
            RuntimeCapability::LayoutCssFlexFactorsV1,
            RuntimeCapability::LayoutCssIntrinsicSizingV1,
            RuntimeCapability::LayoutCssIndefiniteBasisV1,
            RuntimeCapability::LayoutCssContentBoxV1,
            RuntimeCapability::LayoutCssAspectRatioV1,
            RuntimeCapability::LayoutCssAspectRatioPairV1,
            RuntimeCapability::LayoutCssPartialFlexFactorsV1,
            RuntimeCapability::LayoutCssPaintOrderV1,
            RuntimeCapability::LayoutCssPixelBoundsV1,
            RuntimeCapability::LayoutCssPercentageSpacingV1,
            RuntimeCapability::LayoutCssPositionedPaintV1,
            RuntimeCapability::LayoutCssAbsolutePositionV1,
            RuntimeCapability::LayoutCssStackingV1,
            RuntimeCapability::LayoutCssAxisOverflowV1,
            RuntimeCapability::TextCssShapingPrecisionV1,
            RuntimeCapability::TextCssSingleLineEllipsisV1,
            RuntimeCapability::TextSolidUnderlinesV1,
            RuntimeCapability::TextSolidStrikethroughsV1,
            RuntimeCapability::TextCssWrappedTabsV1,
            RuntimeCapability::TextCssPreWrapV1,
            RuntimeCapability::TextCssPreLineV1,
            RuntimeCapability::TextCssNormalWrapV1,
            RuntimeCapability::TextCssTabsV1,
            RuntimeCapability::TextCssNowrapAlignmentV1,
            RuntimeCapability::TextClusterSpacingV1,
            RuntimeCapability::TextCssLetterSpacingV1,
            RuntimeCapability::TextPreservedSpaceBreaksV1,
        ][..]
    };
    let supported: Vec<_> = supported
        .iter()
        .copied()
        .filter(|capability| {
            if std::env::var("NUXIE_DISABLE_CSS_LINEAR_GRADIENT").as_deref() == Ok("1")
                && *capability == RuntimeCapability::LayoutCssLinearGradientV1 { return false; }
            if std::env::var("NUXIE_DISABLE_CSS_GROUP_OPACITY").as_deref() == Ok("1")
                && *capability == RuntimeCapability::LayoutCssGroupOpacityV1 { return false; }
            if std::env::var("NUXIE_DISABLE_CSS_CORNER_RADII").as_deref() == Ok("1")
                && *capability == RuntimeCapability::LayoutCssCornerRadiiV1 { return false; }
            if std::env::var("NUXIE_DISABLE_CSS_BORDERS").as_deref() == Ok("1")
                && matches!(*capability, RuntimeCapability::LayoutCssSolidBordersV1 | RuntimeCapability::LayoutCssBorderSidesV1) { return false; }
            if std::env::var("NUXIE_DISABLE_CSS_CLIP_MARGIN").as_deref() == Ok("1")
                && *capability == RuntimeCapability::LayoutCssOverflowClipMarginV1 { return false; }
            if std::env::var("NUXIE_DISABLE_CSS_AXIS_OVERFLOW").as_deref() == Ok("1")
                && *capability == RuntimeCapability::LayoutCssAxisOverflowV1 { return false; }
            if std::env::var("NUXIE_DISABLE_CSS_STACKING").as_deref() == Ok("1")
                && *capability == RuntimeCapability::LayoutCssStackingV1 { return false; }
            if std::env::var("NUXIE_DISABLE_CSS_ABSOLUTE_POSITION").as_deref() == Ok("1")
                && *capability == RuntimeCapability::LayoutCssAbsolutePositionV1 { return false; }
            if std::env::var("NUXIE_DISABLE_CSS_POSITIONED_PAINT").as_deref() == Ok("1")
                && *capability == RuntimeCapability::LayoutCssPositionedPaintV1 { return false; }
            if std::env::var("NUXIE_DISABLE_CSS_ASPECT_RATIO").as_deref() == Ok("1")
                && *capability == RuntimeCapability::LayoutCssAspectRatioV1 { return false; }
            if std::env::var("NUXIE_DISABLE_CSS_PERCENTAGE_SPACING").as_deref() == Ok("1")
                && *capability == RuntimeCapability::LayoutCssPercentageSpacingV1 { return false; }
            if std::env::var("NUXIE_DISABLE_CSS_CONTENT_BOX").as_deref() == Ok("1")
                && *capability == RuntimeCapability::LayoutCssContentBoxV1 { return false; }
            if std::env::var("NUXIE_DISABLE_CSS_INDEFINITE_BASIS").as_deref() == Ok("1")
                && *capability == RuntimeCapability::LayoutCssIndefiniteBasisV1 { return false; }
            if std::env::var("NUXIE_DISABLE_CSS_FLEX_FACTORS").as_deref() == Ok("1")
                && *capability == RuntimeCapability::LayoutCssFlexFactorsV1 { return false; }
            if std::env::var("NUXIE_DISABLE_CSS_INTRINSIC_SIZING").as_deref() == Ok("1")
                && *capability == RuntimeCapability::LayoutCssIntrinsicSizingV1 { return false; }
            if std::env::var("NUXIE_DISABLE_CSS_PARTIAL_FLEX_FACTORS").ok().as_deref() == Some("1")
                && *capability == RuntimeCapability::LayoutCssPartialFlexFactorsV1 { return false; }
            if std::env::var("NUXIE_DISABLE_CSS_DISTRIBUTED_SPACING").as_deref() == Ok("1")
                && *capability == RuntimeCapability::LayoutCssDistributedSpacingV1 { return false; }
            if std::env::var("NUXIE_DISABLE_CSS_ALIGN_CONTENT").as_deref() == Ok("1")
                && *capability == RuntimeCapability::LayoutCssAlignContentV1 { return false; }
            if std::env::var("NUXIE_DISABLE_CSS_ALIGN_SELF").as_deref() == Ok("1")
                && *capability == RuntimeCapability::LayoutCssAlignSelfV1 { return false; }
            if std::env::var("NUXIE_DISABLE_CSS_PAINT_ORDER").as_deref() == Ok("1")
                && *capability == RuntimeCapability::LayoutCssPaintOrderV1 { return false; }
            if std::env::var("NUXIE_DISABLE_CSS_NORMAL_WRAP").as_deref() == Ok("1")
                && *capability == RuntimeCapability::TextCssNormalWrapV1 { return false; }
            (std::env::var("NUXIE_DISABLE_CSS_SHAPING_PRECISION").as_deref() != Ok("1")
                || *capability != RuntimeCapability::TextCssShapingPrecisionV1)
                && (std::env::var("NUXIE_DISABLE_CSS_PIXEL_BOUNDS").as_deref() != Ok("1")
                    || *capability != RuntimeCapability::LayoutCssPixelBoundsV1)
        })
        .collect();
    requirements
        .ensure_supported(&supported)
        .map_err(|d| format!("{}: {}", d.code, d.message))?;
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &fs::read(&args[0])?,
        RuntimeFactoryHandle::from_factory(&mut factory).ok_or("factory")?,
        None,
        None,
        None,
    )
    .ok_or("Rive import failed")?;
    let css_precision = requirements
        .capabilities
        .contains(&RuntimeCapability::TextCssShapingPrecisionV1)
        || std::env::var_os("NUXIE_CSS_SHAPING_PRECISION").is_some();
    if !requirements.capabilities.is_empty() || css_precision {
        use nuxie_runtime::source::{
            assets::font_asset::FontAsset, text::font_hb::LetterSpacingMode,
        };
        let spacing = if requirements
            .capabilities
            .contains(&RuntimeCapability::TextCssLetterSpacingV1)
        {
            Some(LetterSpacingMode::CssLetterSpacingExperimental)
        } else if requirements
            .capabilities
            .contains(&RuntimeCapability::TextClusterSpacingV1)
        {
            Some(LetterSpacingMode::ClusterExperimental)
        } else {
            None
        };
        for asset in file.with_file(|f| f.assets().to_vec()) {
            if !asset
                .with(|a| a.as_any().is::<FontAsset>())
                .unwrap_or(false)
            {
                continue;
            }
            if css_precision
                && !FontAsset::set_shaping_precision_occurrence(
                    &asset,
                    nuxie_runtime::source::text::font_hb::ShapingPrecision::CssExperimental,
                )
            {
                return Err("Font backend cannot honor CSS shaping precision".into());
            }
            if requirements
                .capabilities
                .contains(&RuntimeCapability::TextCssTabsV1)
                && !FontAsset::set_experimental_css_tabs_occurrence(&asset, true)
            {
                return Err("Font backend cannot honor CSS tab stops".into());
            }
            if spacing
                .is_some_and(|mode| !FontAsset::set_letter_spacing_mode_occurrence(&asset, mode))
            {
                return Err("Font backend cannot honor the required text spacing policy".into());
            }
            if requirements
                .capabilities
                .contains(&RuntimeCapability::TextPreservedSpaceBreaksV1)
                && !FontAsset::set_experimental_space_breaks_occurrence(&asset, true)
            {
                return Err("Font backend cannot honor the required preserved-space breaks".into());
            }
        }
    }
    let artboard = file
        .with_file(File::artboard_default)
        .ok_or("no artboard")?;
    requirements
        .ensure_text_targets(|id| {
            artboard
                .with_artboard(|a| a.objects().get(id as usize).cloned().flatten())
                .and_then(|object| object.with_mut(|o| o.as_text_mut().is_some()))
                .unwrap_or(false)
        })
        .map_err(|d| format!("{}: {}", d.code, d.message))?;
    requirements.ensure_layout_targets(|id| {
        artboard.with_artboard(|a| a.objects().get(id as usize).cloned().flatten())
            .and_then(|object| object.with_mut(|o| o.as_layout_component_mut().is_some()))
            .unwrap_or(false)
    }).map_err(|d| format!("{}: {}", d.code, d.message))?;
    for entry in &requirements.text_policies {
        if entry.policy == TextPolicy::CssSingleLineEllipsisV1 {
            let object = artboard.with_artboard(|a| a.objects()[entry.object_id as usize].clone()).unwrap();
            object.with_mut(|o| o.as_text_mut().unwrap().validate_css_single_line_ellipsis()).unwrap()
                .map_err(|message| format!("invalid-text-ellipsis-target: {message}"))?;
        }
    }
    for entry in &requirements.layout_aspect_ratios {
        use nuxie_runtime::source::layout_component::LayoutComponent;
        let target = artboard.with_artboard(|a| a.objects()[entry.object_id as usize].clone()).unwrap();
        if !LayoutComponent::set_css_ratio_content_box_occurrence(&target, entry.content_box)
            || !LayoutComponent::set_css_ratio_pair_occurrence(&target, entry.pair) {
            return Err("invalid-layout-aspect-ratio-target".into());
        }
    }
    for id in &requirements.layout_content_box {
        use nuxie_runtime::source::layout_component::LayoutComponent;
        let target = artboard.with_artboard(|a| a.objects()[*id as usize].clone()).unwrap();
        if !LayoutComponent::set_css_content_box_occurrence(&target, true) {
            return Err("Invalid content-box target".into());
        }
    }
    for entry in &requirements.layout_flex_factors {
        use nuxie_runtime::source::layout_component::{CssFlexFactors, LayoutComponent};
        let factors = CssFlexFactors::new(entry.grow, entry.shrink)
            .ok_or("invalid-layout-flex-factors")?;
        let object = artboard.with_artboard(|a| a.objects()[entry.object_id as usize].clone()).unwrap();
        if !LayoutComponent::set_css_flex_factors_occurrence(&object, Some(factors)) {
            return Err("invalid-layout-flex-factors-target".into());
        }
    }
    for entry in &requirements.layout_justify_content {
        use nuxie_html_to_riv::JustifyDistribution;
        use nuxie_runtime::source::layout_component::{CssJustifyContent, LayoutComponent};
        let alignment = match entry.alignment {
            JustifyDistribution::SpaceAround => CssJustifyContent::SpaceAround,
            JustifyDistribution::SpaceEvenly => CssJustifyContent::SpaceEvenly,
        };
        let object = artboard.with_artboard(|a| a.objects()[entry.object_id as usize].clone()).unwrap();
        if !LayoutComponent::set_css_justify_content_occurrence(&object, Some(alignment)) {
            return Err("invalid-layout-justify-content-target".into());
        }
    }
    for id in &requirements.layout_percentage_spacing {
        use nuxie_runtime::source::layout_component::LayoutComponent;
        let object = artboard.with_artboard(|a| a.objects()[*id as usize].clone()).unwrap();
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
        let object = artboard.with_artboard(|a| a.objects()[entry.object_id as usize].clone()).unwrap();
        if !LayoutComponent::set_css_align_content_occurrence(&object, Some(alignment)) {
            return Err("invalid-layout-align-content-target".into());
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
        let object = artboard.with_artboard(|a| a.objects()[entry.object_id as usize].clone()).unwrap();
        if !LayoutComponent::set_css_align_self_occurrence(&object, Some(alignment)) {
            return Err("invalid-layout-align-self-target".into());
        }
    }
    for id in &requirements.layout_pixel_bounds {
        let object = artboard.with_artboard(|a| a.objects()[*id as usize].clone()).unwrap();
        object.with_mut(|o| o.as_layout_component_mut().unwrap().set_css_pixel_bounds(true));
    }
    if !requirements.layout_absolute.is_empty() {
        let root = artboard.with_artboard(|a| a.objects()[0].clone()).unwrap();
        if !nuxie_runtime::source::artboard::Artboard::set_css_absolute_position_policy_occurrence(
            &root, &requirements.layout_positioned, &requirements.layout_absolute) {
            return Err("invalid-layout-absolute-position-target".into());
        }
    }
    // Apply to this imported instance before its first layout/draw.
    if !requirements.layout_positioned.is_empty()
        && !artboard.with_artboard_mut(|a| a.set_css_positioned_paint_order(&requirements.layout_positioned)) {
        return Err("Invalid CSS positioned paint target".into());
    }
    if !requirements.layout_stacking.is_empty() {
        let targets = requirements.layout_stacking.iter().map(|entry| (entry.object_id, entry.level)).collect::<Vec<_>>();
        if !artboard.with_artboard_mut(|a| a.set_css_stacking_order(&targets)) {
            return Err("invalid-layout-stacking-target".into());
        }
    }
    if !requirements.layout_axis_overflow.is_empty() {
        use nuxie_runtime::source::{artboard::Artboard, layout_component::CssOverflowAxis};
        let targets = requirements.layout_axis_overflow.iter().map(|entry| (entry.object_id, match entry.axis {
            nuxie_html_to_riv::OverflowAxis::X => CssOverflowAxis::Horizontal,
            nuxie_html_to_riv::OverflowAxis::Y => CssOverflowAxis::Vertical,
        })).collect::<Vec<_>>();
        if !Artboard::set_css_overflow_axes_occurrence(&artboard.core_handle(), &targets) {
            return Err("invalid-layout-axis-overflow-target".into());
        }
    }
    if !requirements.layout_borders.is_empty() || !requirements.layout_border_sides.is_empty() {
        use nuxie_runtime::source::artboard::Artboard;
        let targets = requirements.layout_borders.iter().map(|entry|
            (entry.object_id, [entry.color; 4])).chain(requirements.layout_border_sides.iter()
                .map(|entry| (entry.object_id, entry.colors))).collect::<Vec<_>>();
        if !Artboard::set_css_border_sides_occurrence(&artboard.core_handle(), &targets) {
            return Err("invalid-layout-border-target".into());
        }
    }
    if !requirements.layout_linear_gradients.is_empty() {
        use nuxie_html_to_riv::{LinearGradientDirection, LinearGradientPosition};
        use nuxie_runtime::source::{artboard::Artboard,
            css_linear_gradient::{CssLinearGradient, CssGradientDirection, CssGradientPosition}};
        let targets = requirements.layout_linear_gradients.iter().map(|entry| {
            let direction = match entry.direction {
                LinearGradientDirection::Degrees(v) => CssGradientDirection::Degrees(v),
                LinearGradientDirection::Corner { right, bottom } => CssGradientDirection::Corner { right, bottom },
            };
            let colors = entry.stops.iter().map(|stop| stop.color).collect();
            let positions = entry.stops.iter().map(|stop| stop.position.map(|position| match position {
                LinearGradientPosition::Pixels(v) => CssGradientPosition::Pixels(v),
                LinearGradientPosition::Percent(v) => CssGradientPosition::Percent(v),
            })).collect();
            CssLinearGradient::new(direction, colors, positions).map(|paint| (entry.object_id, paint))
        }).collect::<Option<Vec<_>>>().ok_or("invalid-layout-linear-gradient")?;
        if !Artboard::set_css_linear_gradients_occurrence(&artboard.core_handle(), &targets) {
            return Err("invalid-layout-linear-gradient-target".into());
        }
    }
    if !requirements.layout_group_opacity.is_empty() {
        let targets = requirements.layout_group_opacity.iter()
            .map(|entry| (entry.object_id, entry.opacity)).collect::<Vec<_>>();
        if !artboard.with_artboard_mut(|a| a.set_css_group_opacity(&targets)) {
            return Err("invalid-layout-group-opacity-target".into());
        }
    }
    if !requirements.layout_corner_radii.is_empty() {
        use nuxie_runtime::source::{artboard::Artboard, css_corner_radii::{CssCornerRadii, CssRadiusValue}};
        let targets = requirements.layout_corner_radii.iter().map(|entry| {
            let radii = entry.radii.map(|pair| pair.map(|value| match value {
                nuxie_html_to_riv::CornerRadiusValue::Pixels(value) => CssRadiusValue::Pixels(value),
                nuxie_html_to_riv::CornerRadiusValue::Percent(value) => CssRadiusValue::Percent(value),
            }));
            CssCornerRadii::new(radii).map(|radii| (entry.object_id, radii))
        }).collect::<Option<Vec<_>>>().ok_or("invalid-layout-corner-radii")?;
        if !Artboard::set_css_corner_radii_occurrence(&artboard.core_handle(), &targets) {
            return Err("invalid-layout-corner-radii-target".into());
        }
    }
    if !requirements.layout_overflow_clip_margins.is_empty() {
        use nuxie_runtime::source::{artboard::Artboard, layout_component::{CssOverflowClipBox, CssOverflowClipMargin}};
        let targets = requirements.layout_overflow_clip_margins.iter().map(|entry| {
            let origin = match entry.origin {
                nuxie_html_to_riv::OverflowClipBox::ContentBox => CssOverflowClipBox::Content,
                nuxie_html_to_riv::OverflowClipBox::PaddingBox => CssOverflowClipBox::Padding,
                nuxie_html_to_riv::OverflowClipBox::BorderBox => CssOverflowClipBox::Border,
            };
            CssOverflowClipMargin::new(origin, entry.pixels).map(|margin| (entry.object_id, margin))
                .ok_or("invalid-layout-overflow-clip-margin-offset")
        }).collect::<Result<Vec<_>, _>>()?;
        if !Artboard::set_css_overflow_clip_margins_occurrence(&artboard.core_handle(), &targets) {
            return Err("invalid-layout-overflow-clip-margin-target".into());
        }
    }
    // Explicit pre-contract clip-margin experiment. Never enabled by a Rive
    // artifact; the validation receipt must record this diagnostic policy.
    if let Ok(value) = std::env::var("NUXIE_CSS_CLIP_MARGIN_EXPERIMENT") {
        use nuxie_runtime::source::layout_component::{CssOverflowClipBox, CssOverflowClipMargin};
        let (source_id, origin, pixels): (String, String, f32) = serde_json::from_str(&value)?;
        let origin = match origin.as_str() {
            "content-box" => CssOverflowClipBox::Content,
            "padding-box" => CssOverflowClipBox::Padding,
            "border-box" => CssOverflowClipBox::Border,
            _ => return Err("invalid experimental clip-margin origin".into()),
        };
        let margin = CssOverflowClipMargin::new(origin, pixels)
            .ok_or("invalid experimental clip-margin offset")?;
        let id = map.iter().find(|node| node.id == source_id)
            .ok_or("unknown experimental clip-margin source id")?.object_id;
        let target = artboard.with_artboard(|a| a.objects().get(id as usize).cloned())
            .flatten().ok_or("missing experimental clip-margin target")?;
        if target.with_mut(|o| o.as_layout_component_mut().map(|layout|
            layout.set_css_overflow_clip_margin(Some(margin)))).flatten().is_none() {
            return Err("invalid experimental clip-margin layout target".into());
        }
    }
    // Retained diagnostic injection for frozen pre-contract reproductions.
    if let Ok(names) = std::env::var("NUXIE_CSS_POSITIONED_SOURCE_IDS") {
        let ids = names.split(',').filter(|name| !name.is_empty()).map(|name| {
            map.iter().find(|node| node.id == name).map(|node| node.object_id)
                .ok_or_else(|| format!("Unknown positioned source id: {name}"))
        }).collect::<Result<Vec<_>, _>>()?;
        if !artboard.with_artboard_mut(|a| a.set_css_positioned_paint_order(&ids)) {
            return Err("Invalid CSS positioned paint target".into());
        }
    }
    if requirements.capabilities.contains(&RuntimeCapability::LayoutCssPaintOrderV1) {
        artboard.with_artboard_mut(|a| a.set_css_paint_order(true));
    }
    // Diagnostic only: investigate existing Rive ellipsis before exposing CSS.
    if std::env::var("NUXIE_EXPERIMENTAL_TEXT_ELLIPSIS").as_deref() == Ok("1") {
        let objects = artboard.with_artboard(|a| a.objects().to_vec());
        for object in objects.into_iter().flatten() {
            object.with_mut(|o| {
                if o.as_text_mut().is_some() {
                    let mut reader = nuxie_runtime::source::core::binary_reader::BinaryReader::new(&[3]);
                    assert!(o.deserialize(287, &mut reader));
                    o.as_text_mut().unwrap().overflow_value_changed();
                    if std::env::var("NUXIE_EXPERIMENTAL_CSS_ELLIPSIS").as_deref() == Ok("1") {
                        o.as_text_mut().unwrap().set_experimental_css_ellipsis(true);
                    }
                }
            });
        }
    }
    for entry in &requirements.text_policies {
        let object = artboard
            .with_artboard(|a| a.objects()[entry.object_id as usize].clone())
            .unwrap();
        object.with_mut(|o| {
            let text = o.as_text_mut().unwrap();
            match entry.policy {
                TextPolicy::CssSingleLineEllipsisV1 => text.install_css_single_line_ellipsis().expect("validated ellipsis target"),
                TextPolicy::CssPreWrapV1 => text.set_css_pre_wrap(true),
                TextPolicy::CssPreLineV1 => text.set_css_pre_line(true),
                TextPolicy::CssNormalWrapV1 => text.set_css_normal_wrap(true),
                TextPolicy::CssNowrapAlignmentV1 => text.set_css_nowrap_alignment(true),
            }
        });
    }
    for entry in &requirements.text_underlines {
        use nuxie_html_to_riv::UnderlineSkipInk;
        use nuxie_runtime::source::text::css_decoration::{ResolvedUnderline, SkipInk};
        let lines = entry
            .lines
            .iter()
            .map(|line| {
                let skip = match line.skip_ink {
                    UnderlineSkipInk::None => SkipInk::None,
                    UnderlineSkipInk::Auto => SkipInk::Auto,
                    UnderlineSkipInk::All => SkipInk::All,
                };
                ResolvedUnderline::solid(line.color, line.thickness, line.offset, skip)
                    .ok_or("Invalid resolved underline")
            })
            .collect::<Result<Vec<_>, _>>()?;
        let object = artboard
            .with_artboard(|a| a.objects().get(entry.object_id as usize).cloned().flatten())
            .ok_or("Missing underline target")?;
        object.with_mut(|o| o.as_text_mut().unwrap().set_css_underlines(lines));
    }
    for entry in &requirements.text_strikethroughs {
        use nuxie_runtime::source::text::css_decoration::ResolvedStrikethrough;
        let lines = entry
            .lines
            .iter()
            .map(|line| {
                ResolvedStrikethrough::solid(
                    line.color,
                    line.thickness,
                    line.offset,
                    line.line_baseline,
                )
                .ok_or("Invalid resolved strikethrough")
            })
            .collect::<Result<Vec<_>, _>>()?;
        let object = artboard
            .with_artboard(|a| a.objects().get(entry.object_id as usize).cloned().flatten())
            .ok_or("Missing strikethrough target")?;
        object.with_mut(|o| o.as_text_mut().unwrap().set_css_strikethroughs(lines));
    }
    // Version 1 used a scene-wide switch. Keep its published interpretation.
    if requirements.version == 1
        && requirements
            .capabilities
            .contains(&RuntimeCapability::TextCssNowrapAlignmentV1)
    {
        for object in artboard
            .with_artboard(|a| a.objects().to_vec())
            .into_iter()
            .flatten()
        {
            object.with_mut(|o| {
                if let Some(text) = o.as_text_mut() {
                    text.set_css_nowrap_alignment(true);
                }
            });
        }
    }
    artboard.update_pass(true);
    artboard.set_size(width, height);
    artboard.update_pass(true);
    let mut boxes = serde_json::Map::new();
    for node in &map {
        let object = artboard
            .with_artboard(|a| a.objects().get(node.object_id as usize).cloned().flatten())
            .ok_or("missing source object")?;
        let bounds = object.with(|o| {
            let layout = o.as_layout_component()?;
            if std::env::var_os("NUXIE_TRACE_LAYOUT_STYLES").is_some() {
                eprintln!("layout-style {}: {:?}", node.id, layout.taffy_style());
            }
            let b = layout.layout_bounds();
            let t = o.as_world_transform_component()?.world_transform();
            Some(json!({"x":t[4], "y":t[5], "width":b.width(), "height":b.height(), "hidden":layout.is_collapsed()}))
        }).flatten().ok_or("missing layout")?;
        boxes.insert(node.id.clone(), bounds);
    }
    factory.borrow_mut().frame_size(
        (width * device_scale).round() as u32,
        (height * device_scale).round() as u32,
    );
    factory.borrow_mut().clear_color(0xffffffff);
    factory.borrow_mut().add_sample(0.0);
    let mut renderer = factory.borrow().make_renderer();
    #[cfg(all(target_os = "macos", feature = "native-glyph-controls"))]
    if std::env::var("NUXIE_NATIVE_GLYPHS").as_deref() == Ok("1") {
        use nuxie_render_api::{Aabb, Factory, Mat2D, Renderer};
        #[derive(serde::Deserialize)]
        struct HostState {
            matrix: [f32; 6],
            clip: Option<[f32; 4]>,
            opacity: f32,
            restore_copy: bool,
        }
        let state: Option<HostState> = args
            .get(5)
            .map(|path| -> Result<HostState, Box<dyn Error>> {
                Ok(serde_json::from_slice(&fs::read(path)?)?)
            })
            .transpose()?;
        if state.as_ref().is_some_and(|s| {
            !s.matrix.iter().all(|v| v.is_finite())
                || !s.opacity.is_finite()
                || !(0.0..=1.0).contains(&s.opacity)
                || s.clip
                    .is_some_and(|c| !c.iter().all(|v| v.is_finite()) || c[2] < 0.0 || c[3] < 0.0)
        }) {
            return Err("Invalid host-state control".into());
        }
        let clip = state.as_ref().and_then(|s| s.clip).map(|c| {
            factory.borrow_mut().make_render_path_from_aabb(Aabb::new(
                c[0],
                c[1],
                c[0] + c[2],
                c[1] + c[3],
            ))
        });
        let mut cache = nuxie_renderer::glyph_adapter::GlyphCache::new(factory.clone());
        {
            let mut renderer = cache.wrap(&mut renderer);
            renderer.transform(Mat2D([device_scale, 0.0, 0.0, device_scale, 0.0, 0.0]));
            renderer.save();
            if let Some(clip) = clip.as_ref() {
                renderer.clip_path(clip.as_ref());
            }
            if let Some(state) = state.as_ref() {
                renderer.transform(Mat2D(state.matrix));
                renderer.modulate_opacity(state.opacity);
            }
            if !artboard.try_draw(&mut renderer) { return Err("missing-renderer-group-capacity".into()); }
            renderer.restore();
            if state.as_ref().is_some_and(|s| s.restore_copy) {
                renderer.save();
                renderer.translate(0.0, 160.0);
                if !artboard.try_draw(&mut renderer) { return Err("missing-renderer-group-capacity".into()); }
                renderer.restore();
            }
        }
        fs::write(
            format!("{}.glyph-cache.txt", args[4]),
            format!("{:?}", cache.stats()),
        )?;
    } else {
        if !artboard.try_draw(&mut renderer) { return Err("missing-renderer-group-capacity".into()); }
    }
    #[cfg(not(all(target_os = "macos", feature = "native-glyph-controls")))]
    if !artboard.try_draw(&mut renderer) { return Err("missing-renderer-group-capacity".into()); }
    // Diagnostic-only glyph export from the settled runtime. This observes
    // actual shaped IDs/positions, not browser-derived layout coordinates.
    let glyphs = artboard.with_artboard(|a| {
        a.objects()
            .iter()
            .filter_map(|object| {
                object
                    .as_ref()?
                    .with(|object| {
                        use nuxie_runtime::mechanical_port::source::text::{
                            text::Text, text_style_paint::TextStylePaint,
                        };
                        let text = object.as_any().downcast_ref::<Text>()?;
                        let mut glyphs = Vec::new();
                        for line in text.ordered_lines() {
                            let mut x = line.glyph_line().start_x;
                            for (run, index) in line {
                                let i = index as usize;
                                let color =
                                    text.style_from_shaper_id(run.style_id).and_then(|style| {
                                        style.with_downcast::<TextStylePaint, _>(
                                            TextStylePaint::foreground_color,
                                        )
                                    });
                                glyphs.push(json!({"id":run.glyphs[i], "size":run.size,
                            "x":x+run.offsets[i].x, "y":line.y()+run.offsets[i].y,
                            "color":color}));
                                x += run.advances[i];
                            }
                        }
                        Some(
                            json!({"matrix":text.shape_world_transform().values(),"glyphs":glyphs}),
                        )
                    })
                    .flatten()
            })
            .collect::<Vec<_>>()
    });
    fs::write(
        format!("{}.glyphs.json", args[4]),
        serde_json::to_vec_pretty(&glyphs)?,
    )?;
    fs::write(
        format!("{}.bounds.json", args[4]),
        serde_json::to_vec_pretty(&boxes)?,
    )?;
    fs::write(format!("{}.stream", args[4]), factory.borrow().stream())?;
    Ok(())
}
