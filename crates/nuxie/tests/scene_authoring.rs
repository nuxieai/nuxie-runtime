use anyhow::Result;
use nuxie::{
    Aabb, AnimationId, AnimationStateOptions, AnimationStateSpec, ArtboardComponentListAxis,
    ArtboardComponentListFlow, ArtboardComponentListSpec, ArtboardId, ArtboardListMapRuleSpec,
    ArtboardSpec, BlendAnimation1DSpec, BlendAnimationDirectSpec, BlendState1DSpec,
    BlendValueSource, BooleanInputSpec, ChildIndex, ClippingShapeSpec, ColorInt,
    CubicDetachedVertexSpec, DashPathSpec, DashSpec, DataBindId, DrawError, EditAbort,
    EditErrorKind, EditId, EditReason, EventId, EventSpec, EventStringPropertySpec,
    ExportedAnimatableProperty, ExportedObjectKind, ExportedProperty, ExportedRecord, Factory,
    FeatherSpec, FillRule, FillSpec, FireEventOccurs, FontAssetId, FontAssetSpec, GradientStopSpec,
    ImageAssetId, ImageAssetSpec, ImageDecodeError, ImageSpec, KeyInterpolation,
    LinearAnimationSpec, LinearGradientSpec, MachineId, MachineInputId, MachineLayerSpec,
    MachineListenerSpec, MachineListenerType, MachineSpec, MachineStateFlags, MachineTransitionId,
    MachineTransitionSpec, NestedArtboardSpec, NodeKind, NodeSpec, NumberComparator,
    NumberInputSpec, ObjectId, Parent, PointsPathSpec, PropValueKind, RawPath, RecordingFactory,
    RectangleCornerRadii, RectangleSpec, RenderBuffer, RenderBufferFlags, RenderBufferType,
    RenderImage, RenderPaint, RenderPath, RenderShader, ResolveError, Scene, SceneClippingFillRule,
    SceneEvent, SceneEventStringProperty, SceneFeatherSpace, SceneStrokeCap, SceneStrokeJoin,
    SceneTextAlign, SceneTextOverflow, SceneTextSizing, SceneTextWrap, SceneTx, ScriptAssetSpec,
    ScriptedDrawableSpec, ShaderAssetSpec, ShapeSpec, SolidColorSpec, StaleCursor, StrokeSpec,
    StructureEpoch, TextSpec, TextStylePaintSpec, TextValueRunSpec, TriggerInputSpec, Vec2D,
    ViewModelBooleanSpec, ViewModelChildSpec, ViewModelColorSpec, ViewModelEnumSpec, ViewModelId,
    ViewModelImageSpec, ViewModelInstanceId, ViewModelInstanceSpec, ViewModelListIndexSpec,
    ViewModelListSource, ViewModelListSpec, ViewModelNumberId, ViewModelNumberSpec, ViewModelScope,
    ViewModelSpec, ViewModelStringId, ViewModelStringSpec, ViewModelTriggerSpec,
    VisibilityCondition, props,
};

#[allow(clippy::arithmetic_side_effects)]
fn fixture_font_bytes() -> Vec<u8> {
    let mut accumulator = 0u32;
    let mut bit_count = 0u8;
    let mut decoded = Vec::new();
    for byte in include_bytes!("fixtures/roboto-a.ttf.base64")
        .iter()
        .copied()
        .filter(|byte| !byte.is_ascii_whitespace())
    {
        if byte == b'=' {
            break;
        }
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => panic!("invalid base64 font fixture"),
        };
        accumulator = (accumulator << 6) | u32::from(value);
        bit_count += 6;
        if bit_count >= 8 {
            bit_count -= 8;
            decoded.push((accumulator >> bit_count) as u8);
            accumulator &= (1u32 << bit_count) - 1;
        }
    }
    decoded
}

fn draw_stream(scene: &mut Scene, instance: nuxie::InstanceId) -> Result<String> {
    scene.reset_renderer(instance)?;
    let mut factory = RecordingFactory::new();
    let mut cache = scene.new_draw_token(instance)?;
    let mut renderer = factory.make_renderer();
    scene
        .frame()
        .draw(instance, &mut factory, &mut renderer, &mut cache)?;
    Ok(factory.stream())
}

fn canonical_draw_stream(scene: &mut Scene, instance: nuxie::InstanceId) -> Result<String> {
    scene.reset_renderer(instance)?;
    let mut factory = RecordingFactory::new();
    let mut cache = scene.new_draw_token(instance)?;
    let mut renderer = factory.make_renderer();
    scene
        .frame()
        .draw(instance, &mut factory, &mut renderer, &mut cache)?;
    Ok(factory.canonical_recording().stream().to_owned())
}

fn drawn_move_count(stream: &str) -> usize {
    stream
        .lines()
        .filter(|line| line.starts_with("drawPath "))
        .map(|line| line.matches("move").count())
        .sum()
}

fn drawn_path_points(stream: &str) -> Vec<(f32, f32)> {
    stream
        .lines()
        .filter(|line| line.starts_with("drawPath "))
        .flat_map(|line| {
            let points = line
                .split_once("points=[")
                .and_then(|(_, rest)| rest.split_once("]}"))
                .map(|(points, _)| points)
                .unwrap_or_default();
            points
                .split("),(")
                .filter_map(|point| {
                    let point = point.trim_matches(['(', ')']);
                    let (x, y) = point.split_once(',')?;
                    Some((x.parse().ok()?, y.parse().ok()?))
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn smallest_positive_transform_scale(stream: &str) -> Option<f32> {
    stream
        .lines()
        .filter_map(|line| {
            line.strip_prefix("transform matrix=[")?
                .split(',')
                .next()?
                .parse::<f32>()
                .ok()
                .filter(|scale| *scale > 0.0)
        })
        .min_by(f32::total_cmp)
}

fn overflow_text_scene(
    overflow: SceneTextOverflow,
    width: f32,
    height: f32,
    content: &str,
) -> Result<(Scene, ArtboardId, ObjectId)> {
    overflow_text_scene_with_line_height(overflow, width, height, content, 20.0)
}

fn overflow_text_scene_with_line_height(
    overflow: SceneTextOverflow,
    width: f32,
    height: f32,
    content: &str,
    line_height: f32,
) -> Result<(Scene, ArtboardId, ObjectId)> {
    text_geometry_scene(
        overflow,
        width,
        height,
        content,
        line_height,
        SceneTextAlign::Left,
    )
}

fn text_geometry_scene(
    overflow: SceneTextOverflow,
    width: f32,
    height: f32,
    content: &str,
    line_height: f32,
    align: SceneTextAlign,
) -> Result<(Scene, ArtboardId, ObjectId)> {
    text_geometry_scene_with_letter_spacing(
        overflow,
        width,
        height,
        content,
        line_height,
        align,
        0.0,
    )
}

#[allow(clippy::too_many_arguments)]
fn text_geometry_scene_with_letter_spacing(
    overflow: SceneTextOverflow,
    width: f32,
    height: f32,
    content: &str,
    line_height: f32,
    align: SceneTextAlign,
    letter_spacing: f32,
) -> Result<(Scene, ArtboardId, ObjectId)> {
    let mut scene = Scene::new();
    let ((artboard, text), _) = scene.edit(|tx| {
        let font = tx.create_font_asset(FontAssetSpec {
            name: "Roboto A".into(),
            bytes: fixture_font_bytes(),
        })?;
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Overflow".into(),
            width: 200.0,
            height: 100.0,
        })?;
        let text = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Text(TextSpec {
                name: "Overflow Text".into(),
                x: 10.0,
                y: 10.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                sizing: SceneTextSizing::Fixed,
                width,
                height,
                align,
                wrap: SceneTextWrap::NoWrap,
                overflow,
            }),
        )?;
        let style = tx.create(
            Parent::Object(text),
            NodeSpec::TextStylePaint(TextStylePaintSpec {
                name: "Overflow Style".into(),
                font_size: 20.0,
                line_height,
                letter_spacing,
                font,
            }),
        )?;
        let fill = tx.create(
            Parent::Object(style),
            NodeSpec::Fill(FillSpec {
                name: "Overflow Fill".into(),
            }),
        )?;
        tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Overflow Color".into(),
                color: 0xff11_2233,
            }),
        )?;
        tx.create(
            Parent::Object(text),
            NodeSpec::TextValueRun(TextValueRunSpec {
                name: "Overflow Run".into(),
                text: content.into(),
                style,
            }),
        )?;
        Ok((artboard, text))
    })?;
    Ok((scene, artboard, text))
}

fn wrapped_text_geometry_scene(content: &str, width: f32) -> Result<(Scene, ArtboardId, ObjectId)> {
    let mut scene = Scene::new();
    let ((artboard, text), _) = scene.edit(|tx| {
        let font = tx.create_font_asset(FontAssetSpec {
            name: "Roboto A".into(),
            bytes: fixture_font_bytes(),
        })?;
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Wrap".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let text = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Text(TextSpec {
                name: "Wrapped Text".into(),
                x: 10.0,
                y: 10.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                sizing: SceneTextSizing::Fixed,
                width,
                height: 80.0,
                align: SceneTextAlign::Left,
                wrap: SceneTextWrap::Wrap,
                overflow: SceneTextOverflow::Visible,
            }),
        )?;
        let style = tx.create(
            Parent::Object(text),
            NodeSpec::TextStylePaint(TextStylePaintSpec {
                name: "Wrap Style".into(),
                font_size: 20.0,
                line_height: 24.0,
                letter_spacing: 0.0,
                font,
            }),
        )?;
        tx.create(
            Parent::Object(text),
            NodeSpec::TextValueRun(TextValueRunSpec {
                name: "Wrap Run".into(),
                text: content.into(),
                style,
            }),
        )?;
        Ok((artboard, text))
    })?;
    Ok((scene, artboard, text))
}

fn backtracking_multi_run_text_geometry_scene() -> Result<(Scene, ArtboardId, ObjectId)> {
    let mut scene = Scene::new();
    let ((artboard, text), _) = scene.edit(|tx| {
        let font = tx.create_font_asset(FontAssetSpec {
            name: "Roboto A".into(),
            bytes: fixture_font_bytes(),
        })?;
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Backtracking advances".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let text = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Text(TextSpec {
                name: "Backtracking Text".into(),
                x: 10.0,
                y: 10.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                sizing: SceneTextSizing::Fixed,
                width: 80.0,
                height: 40.0,
                align: SceneTextAlign::Left,
                wrap: SceneTextWrap::NoWrap,
                overflow: SceneTextOverflow::Visible,
            }),
        )?;
        let forward = tx.create(
            Parent::Object(text),
            NodeSpec::TextStylePaint(TextStylePaintSpec {
                name: "Forward".into(),
                font_size: 20.0,
                line_height: 20.0,
                letter_spacing: 0.0,
                font,
            }),
        )?;
        let backward = tx.create(
            Parent::Object(text),
            NodeSpec::TextStylePaint(TextStylePaintSpec {
                name: "Backward".into(),
                font_size: 20.0,
                line_height: 20.0,
                letter_spacing: -25.0,
                font,
            }),
        )?;
        for (name, content, style) in [
            ("Forward A", "a", forward),
            ("Backward B", "b", backward),
            ("Forward C", "c", forward),
        ] {
            tx.create(
                Parent::Object(text),
                NodeSpec::TextValueRun(TextValueRunSpec {
                    name: name.into(),
                    text: content.into(),
                    style,
                }),
            )?;
        }
        Ok((artboard, text))
    })?;
    Ok((scene, artboard, text))
}

#[test]
fn shaped_horizontal_alignment_moves_caret_hit_and_selection_geometry_with_the_glyphs() -> Result<()>
{
    let mut layouts = Vec::new();
    for align in [
        SceneTextAlign::Left,
        SceneTextAlign::Center,
        SceneTextAlign::Right,
    ] {
        let (mut scene, artboard, text) =
            text_geometry_scene(SceneTextOverflow::Visible, 80.0, 40.0, "aa", 20.0, align)?;
        let instance = scene.instantiate(artboard)?;
        let mut frame = scene.frame();
        let start = frame
            .text_caret(instance, text, 0)
            .expect("aligned shaped text has a start caret");
        let end = frame
            .text_caret(instance, text, 2)
            .expect("aligned shaped text has an end caret");
        let selection = frame.text_selection_rects(instance, text, 0..2);
        assert_eq!(selection.len(), 1);
        let selection = selection[0];
        let midpoint = Vec2D::new(
            (start.top.x + end.top.x) / 2.0,
            (start.top.y + start.bottom.y) / 2.0,
        );
        assert_eq!(frame.text_hit(instance, text, midpoint), Some(1));
        layouts.push((start, end, selection));
    }

    let [
        (left_start, left_end, left_selection),
        (center_start, center_end, center_selection),
        (right_start, right_end, right_selection),
    ] = layouts.as_slice()
    else {
        panic!("all three horizontal alignments were measured");
    };
    let glyph_width = left_end.top.x - left_start.top.x;
    let center_shift = center_start.top.x - left_start.top.x;
    let right_shift = right_start.top.x - left_start.top.x;
    assert!((center_shift - (80.0 - glyph_width) / 2.0).abs() <= 0.001);
    assert!((right_shift - (80.0 - glyph_width)).abs() <= 0.001);
    assert!((center_end.top.x - left_end.top.x - center_shift).abs() <= 0.001);
    assert!((right_end.top.x - left_end.top.x - right_shift).abs() <= 0.001);
    assert!((center_selection.min_x - left_selection.min_x - center_shift).abs() <= 0.001);
    assert!((right_selection.max_x - left_selection.max_x - right_shift).abs() <= 0.001);
    Ok(())
}

#[test]
fn shaped_fit_uniformly_scales_caret_hit_and_selection_geometry_into_the_text_box() -> Result<()> {
    let (mut visible, visible_artboard, visible_text) =
        overflow_text_scene(SceneTextOverflow::Visible, 30.0, 40.0, "aaaaaaaa")?;
    let visible_instance = visible.instantiate(visible_artboard)?;
    let mut visible_frame = visible.frame();
    let visible_start = visible_frame
        .text_caret(visible_instance, visible_text, 0)
        .expect("visible text has a start caret");
    let visible_end = visible_frame
        .text_caret(visible_instance, visible_text, 8)
        .expect("visible text has an end caret");
    let visible_selection = visible_frame
        .text_selection_rects(visible_instance, visible_text, 0..8)
        .into_iter()
        .next()
        .expect("visible text has one selected line segment");

    let (mut fit, fit_artboard, fit_text) =
        overflow_text_scene(SceneTextOverflow::Fit, 30.0, 40.0, "aaaaaaaa")?;
    let fit_instance = fit.instantiate(fit_artboard)?;
    let mut fit_frame = fit.frame();
    let fit_start = fit_frame
        .text_caret(fit_instance, fit_text, 0)
        .expect("fit text has a start caret");
    let fit_middle = fit_frame
        .text_caret(fit_instance, fit_text, 4)
        .expect("fit text has a middle caret");
    let fit_end = fit_frame
        .text_caret(fit_instance, fit_text, 8)
        .expect("fit text has an end caret");
    let fit_selection = fit_frame
        .text_selection_rects(fit_instance, fit_text, 0..8)
        .into_iter()
        .next()
        .expect("fit text has one selected line segment");

    let visible_width = visible_end.top.x - visible_start.top.x;
    let fit_width = fit_end.top.x - fit_start.top.x;
    let scale = fit_width / visible_width;
    assert!(scale > 0.0 && scale < 1.0, "fit scale was {scale}");
    assert!((fit_start.top.x - 10.0).abs() <= 0.001);
    assert!((fit_end.top.x - 40.0).abs() <= 0.001);
    let visible_caret_height = visible_start.bottom.y - visible_start.top.y;
    let fit_caret_height = fit_start.bottom.y - fit_start.top.y;
    assert!((fit_caret_height / visible_caret_height - scale).abs() <= 0.001);
    assert!((fit_selection.width() / visible_selection.width() - scale).abs() <= 0.001);
    assert!((fit_selection.height() / visible_selection.height() - scale).abs() <= 0.001);
    assert!((fit_selection.min_x - fit_start.top.x).abs() <= 0.001);
    assert!((fit_selection.max_x - fit_end.bottom.x).abs() <= 0.001);

    for (expected, caret) in [(0, fit_start), (4, fit_middle), (8, fit_end)] {
        let midpoint = Vec2D::new(
            (caret.top.x + caret.bottom.x) / 2.0,
            (caret.top.y + caret.bottom.y) / 2.0,
        );
        assert_eq!(
            fit_frame.text_hit(fit_instance, fit_text, midpoint),
            Some(expected)
        );
    }
    assert_eq!(
        fit_frame.text_hit(fit_instance, fit_text, Vec2D::new(-1_000.0, 20.0)),
        Some(0)
    );
    assert_eq!(
        fit_frame.text_hit(fit_instance, fit_text, Vec2D::new(1_000.0, 20.0)),
        Some(8)
    );
    Ok(())
}

#[test]
fn visibility_filtering_overflow_modes_fail_closed_for_all_public_text_geometry() -> Result<()> {
    for overflow in [
        SceneTextOverflow::Hidden,
        SceneTextOverflow::Clipped,
        SceneTextOverflow::Ellipsis,
    ] {
        let (mut scene, artboard, text) = overflow_text_scene(overflow, 80.0, 40.0, "aa")?;
        let instance = scene.instantiate(artboard)?;
        let mut frame = scene.frame();
        assert_eq!(
            frame.text_caret(instance, text, 0),
            None,
            "{overflow:?} caret geometry is unsupported in v1"
        );
        assert_eq!(
            frame.text_hit(instance, text, Vec2D::new(10.0, 20.0)),
            None,
            "{overflow:?} hit geometry is unsupported in v1"
        );
        assert!(
            frame.text_selection_rects(instance, text, 0..2).is_empty(),
            "{overflow:?} selection geometry is unsupported in v1"
        );
    }

    for overflow in [SceneTextOverflow::Visible, SceneTextOverflow::Fit] {
        let (mut scene, artboard, text) = overflow_text_scene(overflow, 80.0, 40.0, "aa")?;
        let instance = scene.instantiate(artboard)?;
        let mut frame = scene.frame();
        let caret = frame
            .text_caret(instance, text, 0)
            .unwrap_or_else(|| panic!("{overflow:?} remains supported in v1"));
        let midpoint = Vec2D::new(
            (caret.top.x + caret.bottom.x) / 2.0,
            (caret.top.y + caret.bottom.y) / 2.0,
        );
        assert_eq!(frame.text_hit(instance, text, midpoint), Some(0));
        assert_eq!(frame.text_selection_rects(instance, text, 0..2).len(), 1);
    }
    Ok(())
}

#[test]
fn shaped_text_carets_and_hits_round_trip_utf8_byte_offsets_in_world_space() -> Result<()> {
    let (mut scene, artboard, text) =
        overflow_text_scene(SceneTextOverflow::Visible, 80.0, 40.0, "aé")?;
    let instance = scene.instantiate(artboard)?;

    let mut frame = scene.frame();
    let start = frame
        .text_caret(instance, text, 0)
        .expect("the start of shaped text has a caret");
    let between = frame
        .text_caret(instance, text, 1)
        .expect("the UTF-8 boundary between glyphs has a caret");
    let end = frame
        .text_caret(instance, text, 3)
        .expect("the end of shaped text has a caret");

    assert_eq!(start.top.y, between.top.y);
    assert_eq!(start.bottom.y, between.bottom.y);
    assert!(start.top.x < between.top.x && between.top.x < end.top.x);
    assert!(
        start.top.x >= 10.0,
        "the authored Text transform is applied"
    );
    assert_eq!(frame.text_caret(instance, text, 2), None);
    assert_eq!(frame.text_caret(instance, text, 4), None);
    assert_eq!(
        frame.text_hit(instance, text, Vec2D::new(f32::NAN, 0.0)),
        None
    );
    for (expected, caret) in [(0, start), (1, between), (3, end)] {
        let midpoint = Vec2D::new(
            (caret.top.x + caret.bottom.x) / 2.0,
            (caret.top.y + caret.bottom.y) / 2.0,
        );
        assert_eq!(frame.text_hit(instance, text, midpoint), Some(expected));
    }
    Ok(())
}

#[test]
fn shaped_empty_text_run_exposes_only_caret_zero_without_changing_logical_bounds() -> Result<()> {
    let (mut scene, artboard, text) =
        overflow_text_scene(SceneTextOverflow::Visible, 80.0, 40.0, "")?;
    let instance = scene.instantiate(artboard)?;
    let mut frame = scene.frame();
    let caret = frame
        .text_caret(instance, text, 0)
        .expect("an empty styled TextValueRun has its insertion caret");
    assert_eq!(frame.text_caret(instance, text, 1), None);
    assert!(frame.text_selection_rects(instance, text, 0..0).is_empty());
    assert_eq!(
        frame.world_bounds(instance, text),
        Some(Aabb::new(10.0, 10.0, 90.0, 50.0)),
        "empty fixed Text keeps its authored logical bounds"
    );
    let midpoint = Vec2D::new(
        (caret.top.x + caret.bottom.x) / 2.0,
        (caret.top.y + caret.bottom.y) / 2.0,
    );
    assert_eq!(frame.text_hit(instance, text, midpoint), Some(0));
    Ok(())
}

#[test]
fn shaped_text_geometry_rejects_non_text_unknown_and_unshapeable_targets() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, shape, unshapeable_text), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Invalid geometry targets".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Not Text".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let unshapeable_text = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Text(TextSpec {
                name: "No style or run".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                sizing: SceneTextSizing::Fixed,
                width: 80.0,
                height: 40.0,
                align: SceneTextAlign::Left,
                wrap: SceneTextWrap::NoWrap,
                overflow: SceneTextOverflow::Visible,
            }),
        )?;
        Ok((artboard, shape, unshapeable_text))
    })?;
    let instance = scene.instantiate(artboard)?;

    let (mut other_scene, other_artboard, other_text) =
        overflow_text_scene(SceneTextOverflow::Visible, 80.0, 40.0, "a")?;
    let other_instance = other_scene.instantiate(other_artboard)?;

    let mut frame = scene.frame();
    for object in [shape, unshapeable_text, other_text] {
        assert_eq!(frame.text_caret(instance, object, 0), None);
        assert_eq!(frame.text_hit(instance, object, Vec2D::new(0.0, 0.0)), None);
        assert!(
            frame
                .text_selection_rects(instance, object, 0..1)
                .is_empty()
        );
    }
    assert_eq!(frame.text_caret(other_instance, unshapeable_text, 0), None);
    assert_eq!(
        frame.text_hit(other_instance, unshapeable_text, Vec2D::new(0.0, 0.0)),
        None
    );
    assert!(
        frame
            .text_selection_rects(other_instance, unshapeable_text, 0..1)
            .is_empty()
    );
    Ok(())
}

#[test]
fn shaped_text_selection_returns_one_world_rect_for_one_contiguous_line_segment() -> Result<()> {
    let (mut scene, artboard, text) =
        overflow_text_scene(SceneTextOverflow::Visible, 80.0, 40.0, "aé")?;
    let instance = scene.instantiate(artboard)?;

    let mut frame = scene.frame();
    assert!(
        frame.text_selection_rects(instance, text, 1..1).is_empty(),
        "a collapsed selection has no highlight"
    );
    assert!(
        frame.text_selection_rects(instance, text, 1..2).is_empty(),
        "an invalid UTF-8 endpoint has no selection geometry"
    );
    assert!(
        frame.text_selection_rects(instance, text, 3..1).is_empty(),
        "a reversed source range has no selection geometry"
    );
    assert!(
        frame.text_selection_rects(instance, text, 0..4).is_empty(),
        "a range ending past the source has no selection geometry"
    );
    let start = frame
        .text_caret(instance, text, 1)
        .expect("the selection start is a valid UTF-8 boundary");
    let end = frame
        .text_caret(instance, text, 3)
        .expect("the selection end is a valid UTF-8 boundary");
    assert_eq!(
        frame.text_selection_rects(instance, text, 1..3),
        vec![Aabb::new(
            start.top.x,
            start.top.y,
            end.bottom.x,
            end.bottom.y,
        )]
    );
    Ok(())
}

#[test]
fn shaped_multiline_selection_returns_one_world_rect_per_selected_line_segment() -> Result<()> {
    let (mut scene, artboard, text) =
        overflow_text_scene(SceneTextOverflow::Visible, 80.0, 80.0, "a\na")?;
    let instance = scene.instantiate(artboard)?;
    let mut frame = scene.frame();

    let first_start = frame
        .text_caret(instance, text, 0)
        .expect("first line has a start caret");
    let first_end = frame
        .text_caret(instance, text, 1)
        .expect("first line has an end caret");
    let second_start = frame
        .text_caret(instance, text, 2)
        .expect("second line has a start caret");
    let second_end = frame
        .text_caret(instance, text, 3)
        .expect("second line has an end caret");

    assert_eq!(
        frame.text_selection_rects(instance, text, 0..3),
        vec![
            Aabb::new(
                first_start.top.x,
                first_start.top.y,
                first_end.bottom.x,
                first_end.bottom.y,
            ),
            Aabb::new(
                second_start.top.x,
                second_start.top.y,
                second_end.bottom.x,
                second_end.bottom.y,
            ),
        ]
    );
    Ok(())
}

#[test]
fn shaped_text_runs_share_one_public_utf8_byte_offset_space() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, text), _) = scene.edit(|tx| {
        let font = tx.create_font_asset(FontAssetSpec {
            name: "Roboto A".into(),
            bytes: fixture_font_bytes(),
        })?;
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Runs".into(),
            width: 200.0,
            height: 100.0,
        })?;
        let text = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Text(TextSpec {
                name: "Runs Text".into(),
                x: 10.0,
                y: 10.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                sizing: SceneTextSizing::Fixed,
                width: 100.0,
                height: 40.0,
                align: SceneTextAlign::Left,
                wrap: SceneTextWrap::NoWrap,
                overflow: SceneTextOverflow::Visible,
            }),
        )?;
        let style = tx.create(
            Parent::Object(text),
            NodeSpec::TextStylePaint(TextStylePaintSpec {
                name: "Runs Style".into(),
                font_size: 20.0,
                line_height: 24.0,
                letter_spacing: 0.0,
                font,
            }),
        )?;
        for (name, value) in [("First", "a"), ("Second", "éb")] {
            tx.create(
                Parent::Object(text),
                NodeSpec::TextValueRun(TextValueRunSpec {
                    name: name.into(),
                    text: value.into(),
                    style,
                }),
            )?;
        }
        Ok((artboard, text))
    })?;
    let instance = scene.instantiate(artboard)?;

    let mut frame = scene.frame();
    let carets: Vec<nuxie::CaretGeometry> = [0, 1, 3, 4]
        .into_iter()
        .map(|byte_offset| {
            frame
                .text_caret(instance, text, byte_offset)
                .expect("every concatenated UTF-8 boundary has a public caret")
        })
        .collect();
    assert_eq!(frame.text_caret(instance, text, 2), None);
    for (byte_offset, caret) in [0, 1, 3, 4].into_iter().zip(carets) {
        let midpoint = Vec2D::new(
            (caret.top.x + caret.bottom.x) / 2.0,
            (caret.top.y + caret.bottom.y) / 2.0,
        );
        let hit: Option<usize> = frame.text_hit(instance, text, midpoint);
        assert_eq!(hit, Some(byte_offset));
    }
    Ok(())
}

#[test]
fn shaped_combining_cluster_snaps_internal_utf8_boundary_to_the_cluster_end() -> Result<()> {
    let (mut scene, artboard, text) =
        overflow_text_scene(SceneTextOverflow::Visible, 80.0, 40.0, "a\u{0301}")?;
    let instance = scene.instantiate(artboard)?;

    let mut frame = scene.frame();
    let start = frame
        .text_caret(instance, text, 0)
        .expect("the cluster start is a valid UTF-8 boundary");
    let before_mark = frame
        .text_caret(instance, text, 1)
        .expect("the boundary before the combining mark remains addressable");
    let end = frame
        .text_caret(instance, text, 3)
        .expect("the cluster end is a valid UTF-8 boundary");

    assert!(start.top.x < end.top.x);
    assert_eq!(before_mark, end);
    assert_eq!(
        frame.text_selection_rects(instance, text, 0..1),
        vec![Aabb::new(
            start.top.x,
            start.top.y,
            end.bottom.x,
            end.bottom.y,
        )],
        "selection ends at the same snapped cluster edge as its caret"
    );
    assert!(
        frame.text_selection_rects(instance, text, 1..3).is_empty(),
        "two source boundaries snapped to the same cluster edge have no visual segment"
    );
    let midpoint = Vec2D::new(
        (end.top.x + end.bottom.x) / 2.0,
        (end.top.y + end.bottom.y) / 2.0,
    );
    assert_eq!(frame.text_hit(instance, text, midpoint), Some(3));
    Ok(())
}

#[test]
fn shaped_backtracking_multi_run_advances_preserve_visual_extrema_and_caret_hits() -> Result<()> {
    let (mut scene, artboard, text) = backtracking_multi_run_text_geometry_scene()?;
    let instance = scene.instantiate(artboard)?;

    let mut frame = scene.frame();
    let carets = (0..=3)
        .map(|byte_offset| {
            frame
                .text_caret(instance, text, byte_offset)
                .unwrap_or_else(|| panic!("byte boundary {byte_offset} has a caret"))
        })
        .collect::<Vec<_>>();
    assert!(carets[1].top.x > carets[3].top.x);
    let selection = frame.text_selection_rects(instance, text, 0..3);
    assert_eq!(selection.len(), 1);
    assert!(
        selection[0].max_x >= carets[1].top.x,
        "selection must retain the intermediate forward extent before the negative advance"
    );
    for (byte_offset, caret) in carets.into_iter().enumerate() {
        let point = Vec2D::new(
            (caret.top.x + caret.bottom.x) / 2.0,
            (caret.top.y + caret.bottom.y) / 2.0,
        );
        assert_eq!(frame.text_hit(instance, text, point), Some(byte_offset));
    }
    Ok(())
}

#[test]
fn shaped_combining_cluster_stays_indivisible_with_negative_spacing() -> Result<()> {
    let (mut scene, artboard, text) = text_geometry_scene_with_letter_spacing(
        SceneTextOverflow::Visible,
        80.0,
        40.0,
        "a\u{0301}",
        20.0,
        SceneTextAlign::Left,
        -2.0,
    )?;
    let instance = scene.instantiate(artboard)?;

    let mut frame = scene.frame();
    let internal = frame
        .text_caret(instance, text, 1)
        .expect("the internal scalar boundary remains addressable");
    let end = frame
        .text_caret(instance, text, 3)
        .expect("the combining cluster has an end caret");
    assert_eq!(internal, end);
    let point = Vec2D::new(
        (end.top.x + end.bottom.x) / 2.0,
        (end.top.y + end.bottom.y) / 2.0,
    );
    assert_eq!(frame.text_hit(instance, text, point), Some(3));
    Ok(())
}

#[test]
fn shaped_explicit_newlines_keep_both_carets_without_synthesizing_a_trailing_static_line()
-> Result<()> {
    let (mut scene, artboard, text) =
        overflow_text_scene(SceneTextOverflow::Visible, 80.0, 80.0, "a\nb\n")?;
    let instance = scene.instantiate(artboard)?;

    let mut frame = scene.frame();
    let first_start = frame
        .text_caret(instance, text, 0)
        .expect("the first line has a start caret");
    let before_first_newline = frame
        .text_caret(instance, text, 1)
        .expect("the first newline has a preceding caret");
    let after_first_newline = frame
        .text_caret(instance, text, 2)
        .expect("the first newline has a following caret");
    let before_trailing_newline = frame
        .text_caret(instance, text, 3)
        .expect("the trailing newline has a preceding caret");

    assert_eq!(before_first_newline.top.y, first_start.top.y);
    assert!(after_first_newline.top.y > before_first_newline.top.y);
    assert_eq!(after_first_newline.top.x, first_start.top.x);
    assert_eq!(before_trailing_newline.top.y, after_first_newline.top.y);
    assert_eq!(
        frame.text_caret(instance, text, 4),
        None,
        "static Text does not synthesize an editable trailing-empty paragraph"
    );
    assert!(
        frame.text_selection_rects(instance, text, 3..4).is_empty(),
        "the unshaped post-newline offset has no visual selection segment"
    );

    for (byte_offset, caret) in [
        (1, before_first_newline),
        (2, after_first_newline),
        (3, before_trailing_newline),
    ] {
        let midpoint = Vec2D::new(
            (caret.top.x + caret.bottom.x) / 2.0,
            (caret.top.y + caret.bottom.y) / 2.0,
        );
        assert_eq!(frame.text_hit(instance, text, midpoint), Some(byte_offset));
    }
    assert_eq!(
        frame.text_hit(instance, text, Vec2D::new(1_000.0, 1_000.0)),
        Some(3),
        "hit testing clamps to the last retained static glyph line, not the unshaped post-newline offset"
    );
    Ok(())
}

#[test]
fn shaped_crlf_is_one_authored_separator_with_no_internal_caret() -> Result<()> {
    let (mut scene, artboard, text) =
        overflow_text_scene(SceneTextOverflow::Visible, 80.0, 80.0, "a\r\nb")?;
    let instance = scene.instantiate(artboard)?;

    let mut frame = scene.frame();
    let before = frame
        .text_caret(instance, text, 1)
        .expect("the CRLF pair has a preceding caret");
    let after = frame
        .text_caret(instance, text, 3)
        .expect("the CRLF pair has a following caret");
    assert_eq!(frame.text_caret(instance, text, 2), None);
    assert!(after.top.y > before.top.y);
    assert!(frame.text_selection_rects(instance, text, 1..2).is_empty());
    assert!(frame.text_selection_rects(instance, text, 2..3).is_empty());
    for (byte_offset, caret) in [(1, before), (3, after)] {
        let point = Vec2D::new(
            (caret.top.x + caret.bottom.x) / 2.0,
            (caret.top.y + caret.bottom.y) / 2.0,
        );
        assert_eq!(frame.text_hit(instance, text, point), Some(byte_offset));
    }
    Ok(())
}

#[test]
fn shaped_soft_wrap_boundary_uses_downstream_caret_affinity() -> Result<()> {
    let (mut scene, artboard, text) = wrapped_text_geometry_scene("aa", 8.0)?;
    let instance = scene.instantiate(artboard)?;

    let mut frame = scene.frame();
    let start = frame
        .text_caret(instance, text, 0)
        .expect("the first visual line has a start caret");
    let boundary = frame
        .text_caret(instance, text, 1)
        .expect("the soft-wrap boundary has a canonical caret");
    let end = frame
        .text_caret(instance, text, 2)
        .expect("the second visual line has an end caret");
    assert_eq!(boundary.top.x, start.top.x);
    assert!(boundary.top.y > start.top.y);
    assert_eq!(end.top.y, boundary.top.y);
    assert!(end.top.x > boundary.top.x);

    let first_line = frame.text_selection_rects(instance, text, 0..1);
    assert_eq!(first_line.len(), 1);
    let first_line = first_line
        .first()
        .expect("the selected first visual line has one rectangle");
    let upstream_end = Vec2D::new(
        first_line.max_x,
        (first_line.min_y + first_line.max_y) / 2.0,
    );
    let downstream_start = Vec2D::new(
        (boundary.top.x + boundary.bottom.x) / 2.0,
        (boundary.top.y + boundary.bottom.y) / 2.0,
    );
    for point in [
        upstream_end,
        downstream_start,
        upstream_end,
        downstream_start,
    ] {
        assert_eq!(frame.text_hit(instance, text, point), Some(1));
    }
    Ok(())
}

#[test]
fn shaped_soft_wrap_preserves_skipped_ascii_and_multibyte_whitespace_boundaries() -> Result<()> {
    let content = "a  \u{2003}a";
    let (mut scene, artboard, text) = wrapped_text_geometry_scene(content, 8.0)?;
    let instance = scene.instantiate(artboard)?;

    let mut frame = scene.frame();
    let next_line_start = frame
        .text_caret(instance, text, 6)
        .expect("the boundary before the retained second-line glyph has a caret");
    let first_glyph_selection = frame.text_selection_rects(instance, text, 0..1);
    assert_eq!(first_glyph_selection.len(), 1);

    for byte_offset in [1, 2, 3, 6] {
        assert_eq!(
            frame.text_caret(instance, text, byte_offset),
            Some(next_line_start),
            "downstream affinity snaps every skipped whitespace boundary to the next-line start"
        );
        assert_eq!(
            frame.text_selection_rects(instance, text, 0..byte_offset),
            first_glyph_selection,
            "upstream selection affinity snaps every skipped whitespace boundary to the previous-line end"
        );
    }
    assert_eq!(frame.text_caret(instance, text, 4), None);
    assert_eq!(frame.text_caret(instance, text, 5), None);
    assert_eq!(
        frame.text_selection_rects(instance, text, 1..7),
        frame.text_selection_rects(instance, text, 6..7),
        "a selection beginning in skipped whitespace starts at the retained next-line glyph"
    );
    Ok(())
}

#[test]
fn shaped_soft_wrap_preserves_terminal_skipped_whitespace_boundaries_without_a_new_line()
-> Result<()> {
    let (mut scene, artboard, text) = wrapped_text_geometry_scene("a   ", 8.0)?;
    let instance = scene.instantiate(artboard)?;

    let mut frame = scene.frame();
    let retained_end = frame
        .text_caret(instance, text, 1)
        .expect("the retained glyph has an end caret");
    for byte_offset in [1, 2, 3, 4] {
        assert_eq!(
            frame.text_caret(instance, text, byte_offset),
            Some(retained_end),
            "terminal skipped whitespace shares the retained previous-line end"
        );
    }
    assert!(frame.text_selection_rects(instance, text, 1..4).is_empty());
    let shared_end = Vec2D::new(
        (retained_end.top.x + retained_end.bottom.x) / 2.0,
        (retained_end.top.y + retained_end.bottom.y) / 2.0,
    );
    assert_eq!(frame.text_hit(instance, text, shared_end), Some(4));
    Ok(())
}

#[test]
fn shaped_text_geometry_applies_the_authored_non_uniform_world_transform() -> Result<()> {
    const LOCAL_ADVANCE: f32 = 10.878_906;
    const LOCAL_CARET_HEIGHT: f32 = 22.721_352;
    const TRANSLATE_X: f32 = 30.0;
    const TRANSLATE_Y: f32 = 40.0;
    const ROTATION: f32 = std::f32::consts::FRAC_PI_2;
    const SCALE_X: f32 = 2.0;
    const SCALE_Y: f32 = 0.5;

    let mut scene = Scene::new();
    let ((artboard, text), _) = scene.edit(|tx| {
        let font = tx.create_font_asset(FontAssetSpec {
            name: "Roboto A".into(),
            bytes: fixture_font_bytes(),
        })?;
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Transform".into(),
            width: 200.0,
            height: 120.0,
        })?;
        let text = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Text(TextSpec {
                name: "Transformed Text".into(),
                x: TRANSLATE_X,
                y: TRANSLATE_Y,
                opacity: 1.0,
                rotation: ROTATION,
                scale_x: SCALE_X,
                scale_y: SCALE_Y,
                sizing: SceneTextSizing::Fixed,
                width: 80.0,
                height: 40.0,
                align: SceneTextAlign::Left,
                wrap: SceneTextWrap::NoWrap,
                overflow: SceneTextOverflow::Visible,
            }),
        )?;
        let style = tx.create(
            Parent::Object(text),
            NodeSpec::TextStylePaint(TextStylePaintSpec {
                name: "Transform Style".into(),
                font_size: 20.0,
                line_height: 20.0,
                letter_spacing: 0.0,
                font,
            }),
        )?;
        tx.create(
            Parent::Object(text),
            NodeSpec::TextValueRun(TextValueRunSpec {
                name: "Transform Run".into(),
                text: "a".into(),
                style,
            }),
        )?;
        Ok((artboard, text))
    })?;
    let instance = scene.instantiate(artboard)?;

    let transform = |point: Vec2D| {
        let (sin, cos) = ROTATION.sin_cos();
        Vec2D::new(
            cos * SCALE_X * point.x - sin * SCALE_Y * point.y + TRANSLATE_X,
            sin * SCALE_X * point.x + cos * SCALE_Y * point.y + TRANSLATE_Y,
        )
    };
    let expected_start_top = transform(Vec2D::new(0.0, 0.0));
    let expected_start_bottom = transform(Vec2D::new(0.0, LOCAL_CARET_HEIGHT));
    let expected_end_top = transform(Vec2D::new(LOCAL_ADVANCE, 0.0));
    let expected_end_bottom = transform(Vec2D::new(LOCAL_ADVANCE, LOCAL_CARET_HEIGHT));
    let assert_point = |actual: Vec2D, expected: Vec2D| {
        assert!(
            (actual.x - expected.x).abs() <= 0.001,
            "{actual:?} != {expected:?}"
        );
        assert!(
            (actual.y - expected.y).abs() <= 0.001,
            "{actual:?} != {expected:?}"
        );
    };

    let mut frame = scene.frame();
    let start = frame
        .text_caret(instance, text, 0)
        .expect("the transformed glyph has a start caret");
    let end = frame
        .text_caret(instance, text, 1)
        .expect("the transformed glyph has an end caret");
    assert_point(start.top, expected_start_top);
    assert_point(start.bottom, expected_start_bottom);
    assert_point(end.top, expected_end_top);
    assert_point(end.bottom, expected_end_bottom);

    let midpoint = Vec2D::new(
        (end.top.x + end.bottom.x) / 2.0,
        (end.top.y + end.bottom.y) / 2.0,
    );
    assert_eq!(frame.text_hit(instance, text, midpoint), Some(1));
    let rects = frame.text_selection_rects(instance, text, 0..1);
    assert_eq!(rects.len(), 1);
    let rect = rects
        .first()
        .expect("one selected line segment has one world AABB");
    let expected_points = [
        expected_start_top,
        expected_start_bottom,
        expected_end_top,
        expected_end_bottom,
    ];
    let expected = Aabb::new(
        expected_points
            .iter()
            .map(|point| point.x)
            .fold(f32::INFINITY, f32::min),
        expected_points
            .iter()
            .map(|point| point.y)
            .fold(f32::INFINITY, f32::min),
        expected_points
            .iter()
            .map(|point| point.x)
            .fold(f32::NEG_INFINITY, f32::max),
        expected_points
            .iter()
            .map(|point| point.y)
            .fold(f32::NEG_INFINITY, f32::max),
    );
    assert!((rect.min_x - expected.min_x).abs() <= 0.001);
    assert!((rect.min_y - expected.min_y).abs() <= 0.001);
    assert!((rect.max_x - expected.max_x).abs() <= 0.001);
    assert!((rect.max_y - expected.max_y).abs() <= 0.001);
    Ok(())
}

#[test]
fn authored_line_height_sets_each_later_glyph_baseline_without_moving_the_first_baseline()
-> Result<()> {
    let (mut scene, artboard, _) = overflow_text_scene_with_line_height(
        SceneTextOverflow::Visible,
        80.0,
        100.0,
        "a\na",
        40.0,
    )?;
    let instance = scene.instantiate(artboard)?;
    let points = drawn_path_points(&canonical_draw_stream(&mut scene, instance)?);
    assert_eq!(points.len() % 2, 0, "the two identical glyphs split evenly");
    let (first_line, second_line) = points.split_at(points.len() / 2);
    assert_eq!(first_line.len(), second_line.len());
    for (first, second) in first_line.iter().zip(second_line) {
        assert_eq!(second.0, first.0, "line height must not move glyphs on x");
        assert!(
            (second.1 - first.1 - 40.0).abs() <= 0.001,
            "the second glyph must be exactly one authored 40px line-height below the first: {first:?} -> {second:?}"
        );
    }
    Ok(())
}

fn text_box(name: &str, x: f32) -> NodeSpec {
    NodeSpec::Text(TextSpec {
        name: name.into(),
        x,
        y: 10.0,
        opacity: 1.0,
        rotation: 0.0,
        scale_x: 1.0,
        scale_y: 1.0,
        sizing: SceneTextSizing::Fixed,
        width: 80.0,
        height: 30.0,
        align: SceneTextAlign::Left,
        wrap: SceneTextWrap::Wrap,
        overflow: SceneTextOverflow::Visible,
    })
}

fn create_referenced_text_style(
    tx: &mut SceneTx<'_>,
) -> std::result::Result<(ObjectId, ObjectId, ObjectId), EditAbort> {
    let font = tx.create_font_asset(FontAssetSpec {
        name: "Roboto A".into(),
        bytes: fixture_font_bytes(),
    })?;
    let artboard = tx.create_artboard(ArtboardSpec {
        name: "Text".into(),
        width: 200.0,
        height: 100.0,
    })?;
    let first = tx.create(Parent::Artboard(artboard), text_box("First", 10.0))?;
    let style = tx.create(
        Parent::Object(first),
        NodeSpec::TextStylePaint(TextStylePaintSpec {
            name: "First Style".into(),
            font_size: 18.0,
            line_height: 22.0,
            letter_spacing: 0.0,
            font,
        }),
    )?;
    let run = tx.create(
        Parent::Object(first),
        NodeSpec::TextValueRun(TextValueRunSpec {
            name: "First Run".into(),
            text: "a".into(),
            style,
        }),
    )?;
    let second = tx.create(Parent::Artboard(artboard), text_box("Second", 100.0))?;
    Ok((second, style, run))
}

fn create_font_test_text(
    tx: &mut SceneTx<'_>,
    artboard: ArtboardId,
    styles: &[(&str, FontAssetId)],
) -> std::result::Result<(), EditAbort> {
    let text = tx.create(Parent::Artboard(artboard), text_box("Font Test", 10.0))?;
    for (name, font) in styles.iter().copied() {
        let style = tx.create(
            Parent::Object(text),
            NodeSpec::TextStylePaint(TextStylePaintSpec {
                name: format!("{name} Style"),
                font_size: 18.0,
                line_height: 22.0,
                letter_spacing: 0.0,
                font,
            }),
        )?;
        tx.create(
            Parent::Object(text),
            NodeSpec::TextValueRun(TextValueRunSpec {
                name: format!("{name} Run"),
                text: "a".into(),
                style,
            }),
        )?;
    }
    Ok(())
}

#[test]
fn embedded_font_text_draws_and_has_fixed_world_bounds_through_semantic_references() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, text), _) = scene.edit(|tx| {
        let font = tx.create_font_asset(FontAssetSpec {
            name: "Roboto A".into(),
            bytes: fixture_font_bytes(),
        })?;
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Text".into(),
            width: 200.0,
            height: 100.0,
        })?;
        let text = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Text(TextSpec {
                name: "Title".into(),
                x: 10.0,
                y: 20.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                sizing: SceneTextSizing::Fixed,
                width: 120.0,
                height: 40.0,
                align: SceneTextAlign::Left,
                wrap: SceneTextWrap::Wrap,
                overflow: SceneTextOverflow::Visible,
            }),
        )?;
        let style = tx.create(
            Parent::Object(text),
            NodeSpec::TextStylePaint(TextStylePaintSpec {
                name: "Title Style".into(),
                font_size: 24.0,
                line_height: 30.0,
                letter_spacing: 0.0,
                font,
            }),
        )?;
        let fill = tx.create(
            Parent::Object(style),
            NodeSpec::Fill(FillSpec {
                name: "Title Fill".into(),
            }),
        )?;
        tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Title Color".into(),
                color: 0xff11_2233,
            }),
        )?;
        tx.create(
            Parent::Object(text),
            NodeSpec::TextValueRun(TextValueRunSpec {
                name: "Title Run".into(),
                text: "a".into(),
                style,
            }),
        )?;
        Ok((artboard, text))
    })?;
    let instance = scene.instantiate(artboard)?;

    let stream = draw_stream(&mut scene, instance)?;
    assert_eq!(
        scene.frame().world_bounds(instance, text),
        Some(Aabb::new(10.0, 20.0, 130.0, 60.0))
    );
    assert!(
        stream.contains("drawPath"),
        "text must draw real glyph paths"
    );
    assert!(
        stream.contains("ff112233"),
        "text paint must reach the draw"
    );
    for (property, expected) in [
        (props::TRANSLATE_X, 10.0),
        (props::TRANSLATE_Y, 20.0),
        (props::WORLD_OPACITY, 1.0),
        (props::ROTATION, 0.0),
        (props::SCALE_X, 1.0),
        (props::SCALE_Y, 1.0),
    ] {
        let cursor = scene.cursor(instance, text, property)?;
        assert_eq!(scene.frame().get(cursor)?, expected);
    }
    let x = scene.cursor(instance, text, props::TRANSLATE_X)?;
    assert!(scene.frame().set(x, 30.0)?);
    assert_eq!(
        scene.frame().world_bounds(instance, text),
        Some(Aabb::new(30.0, 20.0, 150.0, 60.0))
    );
    Ok(())
}

#[test]
fn hidden_text_omits_lines_that_do_not_fully_fit_the_fixed_height() -> Result<()> {
    let (mut visible, visible_artboard, _) =
        overflow_text_scene(SceneTextOverflow::Visible, 40.0, 25.0, "a\na")?;
    let visible_instance = visible.instantiate(visible_artboard)?;
    let visible_stream = canonical_draw_stream(&mut visible, visible_instance)?;

    let (mut hidden, hidden_artboard, hidden_text) =
        overflow_text_scene(SceneTextOverflow::Hidden, 40.0, 25.0, "a\na")?;
    let hidden_instance = hidden.instantiate(hidden_artboard)?;
    let hidden_stream = canonical_draw_stream(&mut hidden, hidden_instance)?;

    assert_eq!(
        drawn_move_count(&visible_stream),
        drawn_move_count(&hidden_stream) * 2,
        "hidden overflow must retain the first full line and omit the partial second line"
    );
    assert_eq!(
        hidden_stream.matches("clipPath").count(),
        visible_stream.matches("clipPath").count(),
        "hidden overflow must not add a text clip"
    );
    assert_eq!(
        hidden.frame().world_bounds(hidden_instance, hidden_text),
        Some(Aabb::new(10.0, 10.0, 50.0, 35.0)),
        "overflow changes drawing, not the fixed text box"
    );
    Ok(())
}

#[test]
fn clipped_text_draws_partial_lines_through_the_fixed_text_box_clip() -> Result<()> {
    let (mut visible, visible_artboard, _) =
        overflow_text_scene(SceneTextOverflow::Visible, 40.0, 25.0, "a\na")?;
    let visible_instance = visible.instantiate(visible_artboard)?;
    let visible_stream = canonical_draw_stream(&mut visible, visible_instance)?;

    let (mut clipped, clipped_artboard, clipped_text) =
        overflow_text_scene(SceneTextOverflow::Clipped, 40.0, 25.0, "a\na")?;
    let clipped_instance = clipped.instantiate(clipped_artboard)?;
    let clipped_stream = canonical_draw_stream(&mut clipped, clipped_instance)?;

    assert_eq!(
        drawn_move_count(&clipped_stream),
        drawn_move_count(&visible_stream),
        "clipped overflow keeps a partially visible line in the glyph path"
    );
    assert_eq!(
        clipped_stream.matches("clipPath").count(),
        visible_stream.matches("clipPath").count() + 1,
        "clipped overflow must add a text-box render clip"
    );
    assert!(
        clipped_stream.lines().any(|line| {
            line.starts_with("clipPath ")
                && line.contains("points=[(10,10),(50,10),(50,35),(10,35)]")
        }),
        "the text clip must be the transformed 40x25 logical box"
    );
    assert_eq!(
        clipped.frame().world_bounds(clipped_instance, clipped_text),
        Some(Aabb::new(10.0, 10.0, 50.0, 35.0))
    );
    Ok(())
}

#[test]
fn fit_text_uniformly_scales_overwide_glyphs_inside_fixed_logical_bounds() -> Result<()> {
    let (mut visible, visible_artboard, _) =
        overflow_text_scene(SceneTextOverflow::Visible, 30.0, 40.0, "aaaaaaaa")?;
    let visible_instance = visible.instantiate(visible_artboard)?;
    let visible_stream = canonical_draw_stream(&mut visible, visible_instance)?;

    let (mut fit, fit_artboard, fit_text) =
        overflow_text_scene(SceneTextOverflow::Fit, 30.0, 40.0, "aaaaaaaa")?;
    let fit_instance = fit.instantiate(fit_artboard)?;
    let fit_stream = canonical_draw_stream(&mut fit, fit_instance)?;

    assert_eq!(
        smallest_positive_transform_scale(&visible_stream),
        Some(1.0)
    );
    let fit_scale =
        smallest_positive_transform_scale(&fit_stream).expect("fit text emits its glyph transform");
    assert_eq!(
        drawn_move_count(&fit_stream),
        drawn_move_count(&visible_stream)
    );
    assert!(
        fit_scale > 0.0 && fit_scale < 1.0,
        "fit overflow must scale the glyph paths down, got {fit_scale}"
    );
    assert_eq!(
        fit.frame().world_bounds(fit_instance, fit_text),
        Some(Aabb::new(10.0, 10.0, 40.0, 50.0)),
        "fit scales render content while retaining the fixed logical box"
    );
    Ok(())
}

#[test]
fn fixed_empty_text_keeps_its_logical_bounds_without_a_font_or_run() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, text), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Empty Text".into(),
            width: 200.0,
            height: 100.0,
        })?;
        let text = tx.create(Parent::Artboard(artboard), text_box("Empty", 10.0))?;
        Ok((artboard, text))
    })?;
    let instance = scene.instantiate(artboard)?;

    assert_eq!(
        scene.frame().world_bounds(instance, text),
        Some(Aabb::new(10.0, 10.0, 90.0, 40.0))
    );
    Ok(())
}

#[test]
fn empty_font_asset_rejects_atomically_without_an_artboard() -> Result<()> {
    let mut scene = Scene::new();
    let mut created_font = None;

    let error = scene
        .edit(|tx| {
            created_font = Some(tx.create_font_asset(FontAssetSpec {
                name: "Empty".into(),
                bytes: Vec::new(),
            })?);
            Ok(())
        })
        .expect_err("an empty embedded font must reject the edit");

    assert_eq!(error.kind(), EditErrorKind::CommitRejected);
    assert_eq!(error.diagnostic().operation_index, 0);
    assert_eq!(
        error.diagnostic().involved_ids,
        vec![EditId::FontAsset(
            created_font.expect("the transaction allocated the font identity")
        )]
    );
    assert_eq!(error.diagnostic().reason, EditReason::EmptyFontAsset);
    assert_eq!(scene.epoch(), StructureEpoch::INITIAL);
    Ok(())
}

#[test]
fn malformed_font_asset_rejects_before_it_can_invalidate_a_live_scene() -> Result<()> {
    let mut scene = Scene::new();
    let (artboard, _) = scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Existing".into(),
            width: 100.0,
            height: 100.0,
        })
    })?;
    let instance = scene.instantiate(artboard)?;
    let epoch_before = scene.epoch();
    let records_before = scene.export_records();
    let draw_before = draw_stream(&mut scene, instance)?;
    let mut created_font = None;

    let error = scene
        .edit(|tx| {
            created_font = Some(tx.create_font_asset(FontAssetSpec {
                name: "Malformed".into(),
                bytes: b"not a font".to_vec(),
            })?);
            Ok(())
        })
        .expect_err("font bytes must parse before the edit commits");

    assert_eq!(error.kind(), EditErrorKind::CommitRejected);
    assert_eq!(error.diagnostic().operation_index, 0);
    assert_eq!(
        error.diagnostic().involved_ids,
        vec![EditId::FontAsset(
            created_font.expect("the transaction allocated the font identity")
        )]
    );
    assert_eq!(error.diagnostic().reason, EditReason::InvalidFontAsset);
    assert_eq!(scene.epoch(), epoch_before);
    assert_eq!(scene.export_records(), records_before);
    assert_eq!(draw_stream(&mut scene, instance)?, draw_before);
    Ok(())
}

#[test]
fn font_assets_are_explicit_distinct_durable_scene_definitions() -> Result<()> {
    let mut scene = Scene::new();
    let (first, _) = scene.edit(|tx| {
        tx.create_font_asset(FontAssetSpec {
            name: "Shared".into(),
            bytes: fixture_font_bytes(),
        })
    })?;
    let (second, _) = scene.edit(|tx| {
        tx.create_font_asset(FontAssetSpec {
            name: "Shared".into(),
            bytes: fixture_font_bytes(),
        })
    })?;

    assert_ne!(
        first, second,
        "font creation does not implicitly deduplicate"
    );
    let (artboard, _) = scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Later Edit".into(),
            width: 100.0,
            height: 100.0,
        })
    })?;
    assert_eq!(
        scene
            .export_records()
            .records()
            .iter()
            .filter(|record| record.kind == ExportedObjectKind::FontAsset)
            .count(),
        0,
        "record export omits persistent fonts until the authored graph references them"
    );
    scene
        .edit(|tx| create_font_test_text(tx, artboard, &[("First", first), ("Second", second)]))?;
    assert_eq!(
        scene
            .export_records()
            .records()
            .iter()
            .filter(|record| record.kind == ExportedObjectKind::FontAsset)
            .count(),
        2,
        "stable font identities remain usable after unrelated edits"
    );
    Ok(())
}

fn create_image_node(
    tx: &mut SceneTx<'_>,
    artboard: ArtboardId,
    name: &str,
    image: ImageAssetId,
) -> std::result::Result<ObjectId, EditAbort> {
    tx.create(
        Parent::Artboard(artboard),
        NodeSpec::Image(ImageSpec {
            name: name.into(),
            x: 10.0,
            y: 20.0,
            opacity: 0.75,
            rotation: 0.25,
            scale_x: 1.5,
            scale_y: 2.0,
            image,
            origin_x: 0.25,
            origin_y: 0.75,
            fit: 0,
            alignment_x: 0.0,
            alignment_y: 0.0,
            crop: None,
        }),
    )
}

fn valid_40_by_20_png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x28, 0x00, 0x00, 0x00, 0x14, 0x08, 0x06, 0x00, 0x00, 0x00, 0xff,
        0x46, 0x7f, 0xbb, 0x00, 0x00, 0x00, 0x1a, 0x49, 0x44, 0x41, 0x54, 0x78, 0xda, 0xed, 0xc1,
        0x31, 0x01, 0x00, 0x00, 0x00, 0xc2, 0xa0, 0xf5, 0x4f, 0x6d, 0x0c, 0x1f, 0xa0, 0x00, 0x00,
        0x00, 0x7e, 0x06, 0x0c, 0x94, 0x00, 0x01, 0xac, 0x16, 0x5d, 0xaf, 0x00, 0x00, 0x00, 0x00,
        0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ]
}

#[test]
fn image_world_bounds_follow_the_successfully_presented_render_image() -> Result<()> {
    let png = valid_40_by_20_png();

    let mut scene = Scene::new();
    let ((artboard, image), _) = scene.edit(|tx| {
        let asset = tx.create_image_asset(ImageAssetSpec {
            name: "Presented image".into(),
            bytes: png,
        })?;
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Image bounds".into(),
            width: 200.0,
            height: 100.0,
        })?;
        let image = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Image(ImageSpec {
                name: "Centered image".into(),
                x: 100.0,
                y: 50.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 2.0,
                scale_y: 3.0,
                image: asset,
                origin_x: 0.5,
                origin_y: 0.5,
                fit: 0,
                alignment_x: 0.0,
                alignment_y: 0.0,
                crop: None,
            }),
        )?;
        Ok((artboard, image))
    })?;
    let instance = scene.instantiate(artboard)?;

    assert_eq!(
        scene.frame().world_bounds(instance, image),
        None,
        "Image bounds fail closed until a renderer has decoded and presented the asset",
    );

    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    let mut cache = scene.new_draw_token(instance)?;
    scene
        .frame()
        .draw(instance, &mut factory, &mut renderer, &mut cache)?;

    assert_eq!(
        scene.frame().world_bounds(instance, image),
        Some(Aabb::new(60.0, 20.0, 140.0, 80.0)),
        "the public geometry primitive uses the presented image dimensions, origin, and world transform",
    );
    Ok(())
}

#[test]
fn registered_intrinsic_image_dimensions_expose_world_bounds_before_first_draw() -> Result<()> {
    let png = valid_40_by_20_png();

    let mut scene = Scene::new();
    let ((artboard, image), _) = scene.edit(|tx| {
        let asset = tx.create_image_asset_with_intrinsic_dimensions(
            ImageAssetSpec {
                name: "Registered image".into(),
                bytes: png,
            },
            40.0,
            20.0,
        )?;
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Image bounds".into(),
            width: 200.0,
            height: 100.0,
        })?;
        let image = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Image(ImageSpec {
                name: "Centered image".into(),
                x: 100.0,
                y: 50.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 2.0,
                scale_y: 3.0,
                image: asset,
                origin_x: 0.5,
                origin_y: 0.5,
                fit: 0,
                alignment_x: 0.0,
                alignment_y: 0.0,
                crop: None,
            }),
        )?;
        Ok((artboard, image))
    })?;
    let exported = scene.export_records();
    let instance = scene.instantiate(artboard)?;

    assert_eq!(
        scene.frame().world_bounds(instance, image),
        Some(Aabb::new(60.0, 20.0, 140.0, 80.0)),
        "registered intrinsic facts make logical image geometry independent of presentation order",
    );
    assert_eq!(
        scene.export_records(),
        exported,
        "runtime-only intrinsic facts never alter the canonical Rive record stream",
    );
    Ok(())
}

#[test]
fn decoded_image_dimensions_must_match_registered_intrinsic_dimensions() -> Result<()> {
    let png = valid_40_by_20_png();

    let mut scene = Scene::new();
    let (artboard, _) = scene.edit(|tx| {
        let asset = tx.create_image_asset_with_intrinsic_dimensions(
            ImageAssetSpec {
                name: "Mismatched image".into(),
                bytes: png,
            },
            41.0,
            20.0,
        )?;
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Image dimensions".into(),
            width: 200.0,
            height: 100.0,
        })?;
        create_image_node(tx, artboard, "Mismatched image", asset)?;
        Ok(artboard)
    })?;
    let instance = scene.instantiate(artboard)?;
    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    let mut cache = scene.new_draw_token(instance)?;

    assert_eq!(
        scene
            .frame()
            .draw(instance, &mut factory, &mut renderer, &mut cache),
        Err(DrawError::ImageDimensionConflict),
        "a renderer-decoded resource must never silently replace conflicting registered facts",
    );
    Ok(())
}

#[test]
fn registered_intrinsic_image_dimensions_must_be_finite_and_positive() {
    for (width, height, expected) in [
        (
            f32::NAN,
            20.0,
            EditReason::NonFiniteProperty {
                property: "intrinsic_width",
            },
        ),
        (
            40.0,
            0.0,
            EditReason::NonPositiveProperty {
                property: "intrinsic_height",
            },
        ),
    ] {
        let mut scene = Scene::new();
        let error = scene
            .edit(|tx| {
                tx.create_image_asset_with_intrinsic_dimensions(
                    ImageAssetSpec {
                        name: "Invalid dimensions".into(),
                        bytes: vec![1],
                    },
                    width,
                    height,
                )?;
                Ok(())
            })
            .expect_err("invalid intrinsic dimensions abort the edit");
        assert_eq!(error.kind(), EditErrorKind::Aborted);
        assert_eq!(error.diagnostic().reason, expected);
    }
}

struct FailFirstImageDecodeFactory {
    inner: RecordingFactory,
    fail_next_image_decode: bool,
}

impl FailFirstImageDecodeFactory {
    fn new() -> Self {
        Self {
            inner: RecordingFactory::new(),
            fail_next_image_decode: true,
        }
    }
}

impl Factory for FailFirstImageDecodeFactory {
    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer> {
        self.inner
            .make_render_buffer(buffer_type, flags, size_in_bytes)
    }

    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        self.inner
            .make_linear_gradient(sx, sy, ex, ey, colors, stops)
    }

    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        self.inner
            .make_radial_gradient(cx, cy, radius, colors, stops)
    }

    fn make_render_path(&mut self, raw_path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
        self.inner.make_render_path(raw_path, fill_rule)
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        self.inner.make_empty_render_path()
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        self.inner.make_render_paint()
    }

    fn decode_image(
        &mut self,
        data: &[u8],
    ) -> std::result::Result<Box<dyn RenderImage>, ImageDecodeError> {
        if std::mem::take(&mut self.fail_next_image_decode) {
            Err(ImageDecodeError)
        } else {
            self.inner.decode_image(data)
        }
    }
}

#[test]
fn image_decode_failure_is_draw_typed_and_the_same_artboard_occurrence_can_recover() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, image), _) = scene.edit(|tx| {
        let asset = tx.create_image_asset(ImageAssetSpec {
            name: "Recoverable Image".into(),
            bytes: include_bytes!(
                "../../../fixtures/renderer/reference/metal/first-light-rectangle.png"
            )
            .to_vec(),
        })?;
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Images".into(),
            width: 200.0,
            height: 100.0,
        })?;
        let image = create_image_node(tx, artboard, "Recoverable Image", asset)?;
        Ok((artboard, image))
    })?;
    let instance = scene.instantiate(artboard)?;
    let x = scene.cursor(instance, image, props::TRANSLATE_X)?;
    let mut token = scene.new_draw_token(instance)?;
    let mut factory = FailFirstImageDecodeFactory::new();
    let mut renderer = factory.inner.make_renderer();

    assert_eq!(
        scene
            .frame()
            .draw(instance, &mut factory, &mut renderer, &mut token),
        Err(DrawError::ImageDecode)
    );
    assert_eq!(scene.frame().get(x)?, 10.0);
    assert!(scene.frame().set(x, 42.0)?);

    scene
        .frame()
        .draw(instance, &mut factory, &mut renderer, &mut token)?;
    assert_eq!(scene.frame().get(x)?, 42.0);
    assert!(factory.inner.stream().contains("decodeImage"));
    Ok(())
}

fn create_mixed_file_asset_nodes(
    tx: &mut SceneTx<'_>,
    artboard: ArtboardId,
    first_image: ImageAssetId,
    font: FontAssetId,
    second_image: ImageAssetId,
) -> std::result::Result<(), EditAbort> {
    create_image_node(tx, artboard, "First Image", first_image)?;
    create_font_test_text(tx, artboard, &[("Font", font)])?;
    create_image_node(tx, artboard, "Second Image", second_image)?;
    Ok(())
}

#[test]
fn mixed_file_assets_share_one_global_first_use_ordinal() -> Result<()> {
    let mut scene = Scene::new();
    let (artboard, _) = scene.edit(|tx| {
        let font = tx.create_font_asset(FontAssetSpec {
            name: "Font".into(),
            bytes: fixture_font_bytes(),
        })?;
        let second_image = tx.create_image_asset(ImageAssetSpec {
            name: "Second Image".into(),
            bytes: b"second image bytes".to_vec(),
        })?;
        let first_image = tx.create_image_asset(ImageAssetSpec {
            name: "First Image".into(),
            bytes: b"first image bytes".to_vec(),
        })?;
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Mixed Assets".into(),
            width: 200.0,
            height: 100.0,
        })?;
        create_mixed_file_asset_nodes(tx, artboard, first_image, font, second_image)?;
        Ok(artboard)
    })?;

    let exported = scene.export_records();
    assert_eq!(
        exported
            .records()
            .iter()
            .skip(1)
            .take(6)
            .map(|record| record.kind)
            .collect::<Vec<_>>(),
        vec![
            ExportedObjectKind::ImageAsset,
            ExportedObjectKind::FileAssetContents,
            ExportedObjectKind::FontAsset,
            ExportedObjectKind::FileAssetContents,
            ExportedObjectKind::ImageAsset,
            ExportedObjectKind::FileAssetContents,
        ],
        "file assets and their contents follow canonical node first use"
    );
    assert_eq!(
        exported
            .records()
            .iter()
            .filter(|record| {
                matches!(
                    record.kind,
                    ExportedObjectKind::FontAsset | ExportedObjectKind::ImageAsset
                )
            })
            .map(|record| {
                (
                    record.kind,
                    record
                        .properties
                        .iter()
                        .find_map(|property| match property {
                            ExportedProperty::FileAssetId(id) => Some(*id),
                            _ => None,
                        })
                        .expect("every file asset has one global record-local id"),
                )
            })
            .collect::<Vec<_>>(),
        vec![
            (ExportedObjectKind::ImageAsset, 0),
            (ExportedObjectKind::FontAsset, 1),
            (ExportedObjectKind::ImageAsset, 2),
        ]
    );
    assert_eq!(
        exported
            .records()
            .iter()
            .filter(|record| record.kind == ExportedObjectKind::Image)
            .map(|record| {
                record
                    .properties
                    .iter()
                    .find_map(|property| match property {
                        ExportedProperty::ImageAssetId(id) => Some(*id),
                        _ => None,
                    })
                    .expect("every image references its typed image asset")
            })
            .collect::<Vec<_>>(),
        vec![0, 2]
    );
    let style = exported
        .records()
        .iter()
        .find(|record| record.kind == ExportedObjectKind::TextStylePaint)
        .expect("mixed scene exports its text style");
    assert!(
        style
            .properties
            .contains(&ExportedProperty::TextStyleFontAssetId(1))
    );

    scene.instantiate(artboard)?;
    Ok(())
}

#[test]
fn mixed_file_asset_export_converges_from_historical_and_fresh_scenes() -> Result<()> {
    let mut historical = Scene::new();
    let ((historical_artboard, historical_font, historical_first, historical_second), _) =
        historical.edit(|tx| {
            let historical_first = tx.create_image_asset(ImageAssetSpec {
                name: "First Image".into(),
                bytes: b"first image bytes".to_vec(),
            })?;
            let historical_font = tx.create_font_asset(FontAssetSpec {
                name: "Font".into(),
                bytes: fixture_font_bytes(),
            })?;
            let historical_second = tx.create_image_asset(ImageAssetSpec {
                name: "Second Image".into(),
                bytes: b"second image bytes".to_vec(),
            })?;
            let historical_artboard = tx.create_artboard(ArtboardSpec {
                name: "Mixed Assets".into(),
                width: 200.0,
                height: 100.0,
            })?;
            create_mixed_file_asset_nodes(
                tx,
                historical_artboard,
                historical_second,
                historical_font,
                historical_first,
            )?;
            Ok((
                historical_artboard,
                historical_font,
                historical_first,
                historical_second,
            ))
        })?;
    historical.edit(|tx| {
        tx.clear_artboard(historical_artboard)?;
        create_mixed_file_asset_nodes(
            tx,
            historical_artboard,
            historical_first,
            historical_font,
            historical_second,
        )
    })?;

    let mut fresh = Scene::new();
    fresh.edit(|tx| {
        let fresh_second = tx.create_image_asset(ImageAssetSpec {
            name: "Second Image".into(),
            bytes: b"second image bytes".to_vec(),
        })?;
        let fresh_first = tx.create_image_asset(ImageAssetSpec {
            name: "First Image".into(),
            bytes: b"first image bytes".to_vec(),
        })?;
        let fresh_font = tx.create_font_asset(FontAssetSpec {
            name: "Font".into(),
            bytes: fixture_font_bytes(),
        })?;
        let fresh_artboard = tx.create_artboard(ArtboardSpec {
            name: "Mixed Assets".into(),
            width: 200.0,
            height: 100.0,
        })?;
        create_mixed_file_asset_nodes(tx, fresh_artboard, fresh_first, fresh_font, fresh_second)
    })?;

    assert_eq!(historical.export_records(), fresh.export_records());
    Ok(())
}

#[test]
fn image_assets_are_explicit_distinct_durable_scene_definitions() -> Result<()> {
    let mut scene = Scene::new();
    let (first, _) = scene.edit(|tx| {
        tx.create_image_asset(ImageAssetSpec {
            name: "Shared".into(),
            bytes: b"first png bytes".to_vec(),
        })
    })?;
    let (second, _) = scene.edit(|tx| {
        tx.create_image_asset(ImageAssetSpec {
            name: "Shared".into(),
            bytes: b"second png bytes".to_vec(),
        })
    })?;

    assert_ne!(
        first, second,
        "image creation does not implicitly deduplicate"
    );
    let (artboard, _) = scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Later Edit".into(),
            width: 100.0,
            height: 100.0,
        })
    })?;
    assert_eq!(
        scene
            .export_records()
            .records()
            .iter()
            .filter(|record| record.kind == ExportedObjectKind::ImageAsset)
            .count(),
        0,
        "record export omits persistent images until the authored graph references them"
    );
    scene.edit(|tx| {
        create_image_node(tx, artboard, "First Image", first)?;
        create_image_node(tx, artboard, "Second Image", second)?;
        Ok(())
    })?;
    assert_eq!(
        scene
            .export_records()
            .records()
            .iter()
            .filter(|record| record.kind == ExportedObjectKind::ImageAsset)
            .count(),
        2,
        "stable image identities remain usable after unrelated edits"
    );
    Ok(())
}

#[test]
fn image_export_order_and_asset_ids_follow_current_first_use_not_add_history() -> Result<()> {
    let mut historical = Scene::new();
    let (old_first, _) = historical.edit(|tx| {
        tx.create_image_asset(ImageAssetSpec {
            name: "Old First".into(),
            bytes: b"old first bytes".to_vec(),
        })
    })?;
    let (old_second, _) = historical.edit(|tx| {
        tx.create_image_asset(ImageAssetSpec {
            name: "Old Second".into(),
            bytes: b"old second bytes".to_vec(),
        })
    })?;
    historical.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Images".into(),
            width: 200.0,
            height: 100.0,
        })?;
        create_image_node(tx, artboard, "Second Image", old_second)?;
        create_image_node(tx, artboard, "First Image", old_first)?;
        Ok(())
    })?;

    let mut fresh = Scene::new();
    fresh.edit(|tx| {
        let new_second = tx.create_image_asset(ImageAssetSpec {
            name: "Old Second".into(),
            bytes: b"old second bytes".to_vec(),
        })?;
        let new_first = tx.create_image_asset(ImageAssetSpec {
            name: "Old First".into(),
            bytes: b"old first bytes".to_vec(),
        })?;
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Images".into(),
            width: 200.0,
            height: 100.0,
        })?;
        create_image_node(tx, artboard, "Second Image", new_second)?;
        create_image_node(tx, artboard, "First Image", new_first)?;
        Ok(())
    })?;

    let historical_records = historical.export_records();
    assert_eq!(historical_records, fresh.export_records());
    assert_eq!(
        historical_records
            .records()
            .iter()
            .filter(|record| record.kind == ExportedObjectKind::ImageAsset)
            .map(|record| record.properties.clone())
            .collect::<Vec<_>>(),
        vec![
            vec![
                ExportedProperty::AssetName("Old Second".into()),
                ExportedProperty::FileAssetId(0),
            ],
            vec![
                ExportedProperty::AssetName("Old First".into()),
                ExportedProperty::FileAssetId(1),
            ],
        ]
    );
    let image_asset_ids = historical_records
        .records()
        .iter()
        .filter(|record| record.kind == ExportedObjectKind::Image)
        .map(|record| {
            record
                .properties
                .iter()
                .find_map(|property| match property {
                    ExportedProperty::ImageAssetId(id) => Some(*id),
                    _ => None,
                })
                .expect("every exported image references a local image asset")
        })
        .collect::<Vec<_>>();
    assert_eq!(image_asset_ids, vec![0, 1]);
    Ok(())
}

#[test]
fn image_origin_zero_is_explicit_against_the_cpp_schema_default() -> Result<()> {
    let mut scene = Scene::new();
    scene.edit(|tx| {
        let image = tx.create_image_asset(ImageAssetSpec {
            name: "Default Origin".into(),
            bytes: b"default origin bytes".to_vec(),
        })?;
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Images".into(),
            width: 200.0,
            height: 100.0,
        })?;
        tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Image(ImageSpec {
                name: "Default Origin Image".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                image,
                origin_x: 0.0,
                origin_y: 0.0,
                fit: 0,
                alignment_x: 0.0,
                alignment_y: 0.0,
                crop: None,
            }),
        )?;
        Ok(())
    })?;

    let records = scene.export_records();
    let image = records
        .records()
        .iter()
        .find(|record| record.kind == ExportedObjectKind::Image)
        .expect("image node is exported");
    assert_eq!(
        image.properties,
        vec![
            ExportedProperty::ComponentName("Default Origin Image".into()),
            ExportedProperty::ImageAssetId(0),
            ExportedProperty::ImageOriginX(0.0),
            ExportedProperty::ImageOriginY(0.0),
        ],
        // C++ oracle d788e8ec: dev/defs/shapes/image.json and generated
        // image_base.hpp initialize both Image origins to 0.5. A top-left
        // authored origin is therefore observable and must survive export.
        "Image origin zero differs from the C++ Rive schema default of 0.5"
    );
    Ok(())
}

#[test]
fn image_rejects_an_asset_identity_owned_by_another_scene_atomically() -> Result<()> {
    let mut source = Scene::new();
    let (foreign_image, _) = source.edit(|tx| {
        tx.create_image_asset(ImageAssetSpec {
            name: "Foreign".into(),
            bytes: b"foreign bytes".to_vec(),
        })
    })?;
    let mut target = Scene::new();
    let mut image = None;

    let error = target
        .edit(|tx| {
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Target".into(),
                width: 100.0,
                height: 100.0,
            })?;
            image = Some(create_image_node(
                tx,
                artboard,
                "Foreign Image",
                foreign_image,
            )?);
            Ok(())
        })
        .expect_err("foreign image asset must fail during commit");

    assert_eq!(error.kind(), EditErrorKind::CommitRejected);
    assert_eq!(error.diagnostic().reason, EditReason::UnknownImageAsset);
    assert_eq!(
        error.diagnostic().involved_ids,
        vec![
            EditId::Object(image.expect("create returns a speculative object before commit")),
            EditId::ImageAsset(foreign_image),
        ]
    );
    assert_eq!(target.export_records().records().len(), 1);
    Ok(())
}

#[test]
fn switching_fonts_exports_only_the_current_reference_like_a_fresh_scene() -> Result<()> {
    let mut incremental = Scene::new();
    let ((artboard, _roboto), _) = incremental.edit(|tx| {
        let roboto = tx.create_font_asset(FontAssetSpec {
            name: "Roboto".into(),
            bytes: fixture_font_bytes(),
        })?;
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Text".into(),
            width: 200.0,
            height: 100.0,
        })?;
        create_font_test_text(tx, artboard, &[("Roboto", roboto)])?;
        Ok((artboard, roboto))
    })?;
    incremental.edit(|tx| {
        let inter = tx.create_font_asset(FontAssetSpec {
            name: "Inter".into(),
            bytes: fixture_font_bytes(),
        })?;
        tx.clear_artboard(artboard)?;
        create_font_test_text(tx, artboard, &[("Inter", inter)])
    })?;

    let mut fresh = Scene::new();
    fresh.edit(|tx| {
        let inter = tx.create_font_asset(FontAssetSpec {
            name: "Inter".into(),
            bytes: fixture_font_bytes(),
        })?;
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Text".into(),
            width: 200.0,
            height: 100.0,
        })?;
        create_font_test_text(tx, artboard, &[("Inter", inter)])
    })?;

    let incremental_records = incremental.export_records();
    assert_eq!(incremental_records, fresh.export_records());
    assert_eq!(
        incremental_records
            .records()
            .iter()
            .filter(|record| record.kind == ExportedObjectKind::FontAsset)
            .count(),
        1
    );
    assert_eq!(
        incremental_records
            .records()
            .iter()
            .filter(|record| record.kind == ExportedObjectKind::FileAssetContents)
            .count(),
        1
    );
    let style = incremental_records
        .records()
        .iter()
        .find(|record| record.kind == ExportedObjectKind::TextStylePaint)
        .expect("current text style is exported");
    assert!(
        style
            .properties
            .contains(&ExportedProperty::TextStyleFontAssetId(0))
    );
    Ok(())
}

#[test]
fn font_export_order_and_asset_ids_follow_current_first_use_not_add_history() -> Result<()> {
    let mut historical = Scene::new();
    let (inter, _) = historical.edit(|tx| {
        tx.create_font_asset(FontAssetSpec {
            name: "Inter".into(),
            bytes: fixture_font_bytes(),
        })
    })?;
    let (roboto, _) = historical.edit(|tx| {
        tx.create_font_asset(FontAssetSpec {
            name: "Roboto".into(),
            bytes: fixture_font_bytes(),
        })
    })?;
    historical.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Text".into(),
            width: 200.0,
            height: 100.0,
        })?;
        create_font_test_text(tx, artboard, &[("Roboto", roboto), ("Inter", inter)])
    })?;

    let mut fresh = Scene::new();
    fresh.edit(|tx| {
        let roboto = tx.create_font_asset(FontAssetSpec {
            name: "Roboto".into(),
            bytes: fixture_font_bytes(),
        })?;
        let inter = tx.create_font_asset(FontAssetSpec {
            name: "Inter".into(),
            bytes: fixture_font_bytes(),
        })?;
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Text".into(),
            width: 200.0,
            height: 100.0,
        })?;
        create_font_test_text(tx, artboard, &[("Roboto", roboto), ("Inter", inter)])
    })?;

    let historical_records = historical.export_records();
    assert_eq!(historical_records, fresh.export_records());
    assert_eq!(
        historical_records
            .records()
            .iter()
            .filter(|record| record.kind == ExportedObjectKind::FontAsset)
            .map(|record| record.properties.clone())
            .collect::<Vec<_>>(),
        vec![
            vec![
                ExportedProperty::AssetName("Roboto".into()),
                ExportedProperty::FileAssetId(0),
            ],
            vec![
                ExportedProperty::AssetName("Inter".into()),
                ExportedProperty::FileAssetId(1),
            ],
        ]
    );
    let style_asset_ids = historical_records
        .records()
        .iter()
        .filter(|record| record.kind == ExportedObjectKind::TextStylePaint)
        .map(|record| {
            record
                .properties
                .iter()
                .find_map(|property| match property {
                    ExportedProperty::TextStyleFontAssetId(id) => Some(*id),
                    _ => None,
                })
                .expect("every exported text style references a local font asset")
        })
        .collect::<Vec<_>>();
    assert_eq!(style_asset_ids, vec![0, 1]);
    Ok(())
}

#[test]
fn text_style_rejects_a_font_identity_owned_by_another_scene_atomically() -> Result<()> {
    let mut source = Scene::new();
    let (foreign_font, _) = source.edit(|tx| {
        tx.create_font_asset(FontAssetSpec {
            name: "Foreign".into(),
            bytes: fixture_font_bytes(),
        })
    })?;
    let mut target = Scene::new();
    let mut style = None;

    let error = target
        .edit(|tx| {
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Target".into(),
                width: 100.0,
                height: 100.0,
            })?;
            let text = tx.create(
                Parent::Artboard(artboard),
                NodeSpec::Text(TextSpec {
                    name: "Text".into(),
                    x: 0.0,
                    y: 0.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    sizing: SceneTextSizing::Fixed,
                    width: 80.0,
                    height: 30.0,
                    align: SceneTextAlign::Left,
                    wrap: SceneTextWrap::Wrap,
                    overflow: SceneTextOverflow::Visible,
                }),
            )?;
            style = Some(tx.create(
                Parent::Object(text),
                NodeSpec::TextStylePaint(TextStylePaintSpec {
                    name: "Foreign Style".into(),
                    font_size: 18.0,
                    line_height: 22.0,
                    letter_spacing: 0.0,
                    font: foreign_font,
                }),
            )?);
            Ok(())
        })
        .expect_err("a font identity is meaningful only in its owning scene");

    assert_eq!(error.kind(), EditErrorKind::CommitRejected);
    assert_eq!(error.diagnostic().operation_index, 2);
    assert_eq!(
        error.diagnostic().involved_ids,
        vec![
            EditId::Object(style.expect("the transaction allocated the style")),
            EditId::FontAsset(foreign_font),
        ]
    );
    assert_eq!(error.diagnostic().reason, EditReason::UnknownFontAsset);
    assert_eq!(target.epoch(), StructureEpoch::INITIAL);
    Ok(())
}

#[test]
fn text_run_rejects_a_style_owned_by_another_text() -> Result<()> {
    let mut scene = Scene::new();
    let mut run = None;
    let mut foreign_style = None;

    let error = scene
        .edit(|tx| {
            let font = tx.create_font_asset(FontAssetSpec {
                name: "Roboto A".into(),
                bytes: fixture_font_bytes(),
            })?;
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Text".into(),
                width: 200.0,
                height: 100.0,
            })?;
            let text_spec = |name: &str| {
                NodeSpec::Text(TextSpec {
                    name: name.into(),
                    x: 0.0,
                    y: 0.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    sizing: SceneTextSizing::Fixed,
                    width: 80.0,
                    height: 30.0,
                    align: SceneTextAlign::Left,
                    wrap: SceneTextWrap::Wrap,
                    overflow: SceneTextOverflow::Visible,
                })
            };
            let first_text = tx.create(Parent::Artboard(artboard), text_spec("First"))?;
            foreign_style = Some(tx.create(
                Parent::Object(first_text),
                NodeSpec::TextStylePaint(TextStylePaintSpec {
                    name: "First Style".into(),
                    font_size: 18.0,
                    line_height: 22.0,
                    letter_spacing: 0.0,
                    font,
                }),
            )?);
            let second_text = tx.create(Parent::Artboard(artboard), text_spec("Second"))?;
            run = Some(tx.create(
                Parent::Object(second_text),
                NodeSpec::TextValueRun(TextValueRunSpec {
                    name: "Invalid Run".into(),
                    text: "a".into(),
                    style: foreign_style.expect("the style was created"),
                }),
            )?);
            Ok(())
        })
        .expect_err("a text run cannot borrow a sibling text's style");

    assert_eq!(error.kind(), EditErrorKind::CommitRejected);
    assert_eq!(error.diagnostic().operation_index, 5);
    assert_eq!(
        error.diagnostic().involved_ids,
        vec![
            EditId::Object(run.expect("the transaction allocated the run")),
            EditId::Object(foreign_style.expect("the transaction allocated the style")),
        ]
    );
    assert_eq!(
        error.diagnostic().reason,
        EditReason::InvalidReference {
            expected: NodeKind::TextStylePaint,
            actual: Some(NodeKind::TextStylePaint),
        }
    );
    assert_eq!(scene.epoch(), StructureEpoch::INITIAL);
    Ok(())
}

#[test]
fn root_text_can_reorder_within_its_artboard() -> Result<()> {
    let mut scene = Scene::new();
    let (second, _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 200.0,
            height: 100.0,
        })?;
        tx.create(Parent::Artboard(artboard), text_box("First", 10.0))?;
        tx.create(Parent::Artboard(artboard), text_box("Second", 100.0))
    })?;

    scene.edit(|tx| tx.reorder(second, ChildIndex::First))?;

    assert_eq!(
        exported_component_names(&scene),
        ["Main", "Second", "First"]
    );
    Ok(())
}

#[test]
fn root_text_remove_and_restore_preserves_identity_and_records() -> Result<()> {
    let mut scene = Scene::new();
    let (text, _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 200.0,
            height: 100.0,
        })?;
        tx.create(Parent::Artboard(artboard), text_box("Title", 10.0))
    })?;
    let records_before = scene.export_records();

    let (removed, _) = scene.edit(|tx| tx.remove(text))?;
    let (restored, _) = scene.edit(|tx| tx.restore(removed))?;

    assert_eq!(restored, text);
    assert_eq!(scene.export_records(), records_before);
    Ok(())
}

#[test]
fn root_text_can_reparent_across_artboards() -> Result<()> {
    let mut scene = Scene::new();
    let ((text, target), _) = scene.edit(|tx| {
        let source = tx.create_artboard(ArtboardSpec {
            name: "Source".into(),
            width: 200.0,
            height: 100.0,
        })?;
        let target = tx.create_artboard(ArtboardSpec {
            name: "Target".into(),
            width: 200.0,
            height: 100.0,
        })?;
        let text = tx.create(Parent::Artboard(source), text_box("Moved", 10.0))?;
        Ok((text, target))
    })?;

    scene.edit(|tx| tx.reparent(text, Parent::Artboard(target), ChildIndex::First))?;

    assert_eq!(
        exported_component_names(&scene),
        ["Source", "Target", "Moved"]
    );
    Ok(())
}

#[test]
fn reparenting_a_referenced_style_reports_the_reparent_operation() -> Result<()> {
    let mut scene = Scene::new();
    let ((second, style, run), _) = scene.edit(create_referenced_text_style)?;
    let epoch_before = scene.epoch();
    let records_before = scene.export_records();

    let error = scene
        .edit(|tx| tx.reparent(style, Parent::Object(second), ChildIndex::First))
        .expect_err("moving a referenced style away from its run must reject commit");

    assert_eq!(error.kind(), EditErrorKind::CommitRejected);
    assert_eq!(error.diagnostic().operation_index, 0);
    assert_eq!(
        error.diagnostic().involved_ids,
        vec![EditId::Object(run), EditId::Object(style)]
    );
    assert_eq!(
        error.diagnostic().reason,
        EditReason::InvalidReference {
            expected: NodeKind::TextStylePaint,
            actual: Some(NodeKind::TextStylePaint),
        }
    );
    assert_eq!(scene.epoch(), epoch_before);
    assert_eq!(scene.export_records(), records_before);

    Ok(())
}

#[test]
fn removing_a_referenced_style_reports_the_remove_operation() -> Result<()> {
    let mut scene = Scene::new();
    let ((_, style, run), _) = scene.edit(create_referenced_text_style)?;
    let epoch_before = scene.epoch();
    let records_before = scene.export_records();

    let error = scene
        .edit(|tx| {
            tx.remove(style)?;
            Ok(())
        })
        .expect_err("removing a referenced style must reject commit");

    assert_eq!(error.kind(), EditErrorKind::CommitRejected);
    assert_eq!(error.diagnostic().operation_index, 0);
    assert_eq!(
        error.diagnostic().involved_ids,
        vec![EditId::Object(run), EditId::Object(style)]
    );
    assert_eq!(error.diagnostic().reason, EditReason::UnknownObject);
    assert_eq!(scene.epoch(), epoch_before);
    assert_eq!(scene.export_records(), records_before);
    Ok(())
}

#[test]
fn frame_advance_targets_only_the_requested_live_instance() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, _, color), _) = scene.edit(|tx| create_card(tx, "Main", 0xff11_2233))?;
    let first = scene.instantiate(artboard)?;
    let second = scene.instantiate(artboard)?;
    let first_cursor = scene.cursor(first, color, props::COLOR_VALUE)?;
    let second_cursor = scene.cursor(second, color, props::COLOR_VALUE)?;
    assert!(scene.frame().set(first_cursor, 0xff44_5566)?);
    assert!(scene.frame().set(second_cursor, 0xff77_8899)?);

    let mut events: Vec<SceneEvent> = Vec::new();
    assert!(scene.frame().advance(first, 1.0 / 120.0, &mut events));
    assert!(
        events.is_empty(),
        "the current static scene emits no events"
    );
    assert!(
        !scene.frame().advance(first, 0.0, &mut events),
        "the requested instance is settled"
    );
    assert!(
        scene.frame().advance(second, 0.0, &mut events),
        "advancing the first instance must not settle the second"
    );
    assert!(
        !scene.frame().advance(second, 0.0, &mut events),
        "the second instance settles independently"
    );

    scene.drop_instance(first);
    assert!(
        !scene.frame().advance(first, 1.0 / 120.0, &mut events),
        "a dropped instance is an unchanged frame"
    );
    Ok(())
}

#[test]
fn authored_linear_animation_exports_canonical_records_and_scrubs_the_live_instance() -> Result<()>
{
    let mut scene = Scene::new();
    let ((artboard, shape, animation), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Canvas".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Fader".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new("Card", 80.0, 60.0)),
        )?;
        let fill = tx.create(
            Parent::Object(shape),
            NodeSpec::Fill(FillSpec {
                name: "Card Fill".into(),
            }),
        )?;
        tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Card Color".into(),
                color: 0xff44_5566,
            }),
        )?;
        let animation = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Fade".into(),
                fps: 60,
                duration: 60,
            },
        )?;
        tx.animations()
            .set_key(animation, shape, props::WORLD_OPACITY, 0, 0.0)?;
        tx.animations()
            .set_key(animation, shape, props::WORLD_OPACITY, 60, 1.0)?;
        Ok((artboard, shape, animation))
    })?;

    assert_eq!(
        scene
            .export_records()
            .records()
            .iter()
            .map(|record| record.kind)
            .collect::<Vec<_>>(),
        vec![
            ExportedObjectKind::Backboard,
            ExportedObjectKind::Artboard,
            ExportedObjectKind::Shape,
            ExportedObjectKind::Rectangle,
            ExportedObjectKind::Fill,
            ExportedObjectKind::SolidColor,
            ExportedObjectKind::LinearAnimation,
            ExportedObjectKind::KeyedObject,
            ExportedObjectKind::KeyedProperty,
            ExportedObjectKind::KeyFrameDouble,
            ExportedObjectKind::KeyFrameDouble,
        ],
        "animation records follow every visual target and stay in ImportContext order"
    );
    assert_eq!(
        &scene.export_records().records()[6..],
        &[
            ExportedRecord {
                kind: ExportedObjectKind::LinearAnimation,
                properties: vec![
                    ExportedProperty::AnimationName("Fade".into()),
                    ExportedProperty::AnimationDuration(60),
                ],
            },
            ExportedRecord {
                kind: ExportedObjectKind::KeyedObject,
                properties: vec![ExportedProperty::KeyedObjectId(1)],
            },
            ExportedRecord {
                kind: ExportedObjectKind::KeyedProperty,
                properties: vec![ExportedProperty::KeyedProperty(
                    ExportedAnimatableProperty::WorldOpacity,
                )],
            },
            ExportedRecord {
                kind: ExportedObjectKind::KeyFrameDouble,
                properties: vec![
                    ExportedProperty::KeyFrame(0),
                    ExportedProperty::KeyFrameInterpolationLinear,
                    ExportedProperty::KeyFrameDoubleValue(0.0),
                ],
            },
            ExportedRecord {
                kind: ExportedObjectKind::KeyFrameDouble,
                properties: vec![
                    ExportedProperty::KeyFrame(60),
                    ExportedProperty::KeyFrameInterpolationLinear,
                    ExportedProperty::KeyFrameDoubleValue(1.0),
                ],
            },
        ]
    );

    let instance = scene.instantiate(artboard)?;
    let second_instance = scene.instantiate(artboard)?;
    let opacity = scene.cursor(instance, shape, props::WORLD_OPACITY)?;
    let second_opacity = scene.cursor(second_instance, shape, props::WORLD_OPACITY)?;
    scene.frame().scrub(instance, animation, 0.0)?;
    assert_eq!(scene.frame().get(opacity)?, 0.0);
    assert_eq!(scene.frame().get(second_opacity)?, 1.0);
    let transparent_draw = canonical_draw_stream(&mut scene, instance)?;
    scene.frame().scrub(instance, animation, 0.5)?;
    assert_eq!(scene.frame().get(opacity)?, 0.5);
    assert_eq!(scene.frame().get(second_opacity)?, 1.0);
    scene.frame().scrub(second_instance, animation, 0.25)?;
    assert_eq!(scene.frame().get(second_opacity)?, 0.25);
    assert_eq!(scene.frame().get(opacity)?, 0.5);
    scene.frame().scrub(instance, animation, 1.0)?;
    assert_eq!(scene.frame().get(opacity)?, 1.0);
    let opaque_draw = canonical_draw_stream(&mut scene, instance)?;
    assert_ne!(
        transparent_draw, opaque_draw,
        "scrubbed state is consumed by draw on the same retained InstanceId"
    );
    Ok(())
}

#[test]
fn authored_subobject_animation_uses_typed_geometry_paint_and_stroke_properties() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, rectangle, color, stroke, animation), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Canvas".into(),
            width: 220.0,
            height: 120.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Card".into(),
                x: 110.0,
                y: 60.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let rectangle = tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec {
                name: "Card Rectangle".into(),
                width: 96.0,
                height: 44.0,
                corner_radii: Some(RectangleCornerRadii {
                    top_left: 2.0,
                    top_right: 2.0,
                    bottom_right: 2.0,
                    bottom_left: 2.0,
                    linked: true,
                }),
            }),
        )?;
        let fill = tx.create(
            Parent::Object(shape),
            NodeSpec::Fill(FillSpec {
                name: "Card Fill".into(),
            }),
        )?;
        let color = tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Card Color".into(),
                color: 0xff3b_82f6,
            }),
        )?;
        let stroke = tx.create(
            Parent::Object(shape),
            NodeSpec::Stroke(StrokeSpec {
                name: "Card Stroke".into(),
                thickness: 1.0,
                cap: SceneStrokeCap::Butt,
                join: SceneStrokeJoin::Miter,
                transform_affects_stroke: true,
            }),
        )?;
        tx.create(
            Parent::Object(stroke),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Card Stroke Color".into(),
                color: 0xff11_2233,
            }),
        )?;

        let animation = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Restyle".into(),
                fps: 60,
                duration: 60,
            },
        )?;
        let mut animations = tx.animations();
        for (property, from, to) in [
            (props::PATH_WIDTH, 96.0, 160.0),
            (props::PATH_HEIGHT, 44.0, 68.0),
            (props::RECTANGLE_CORNER_RADIUS_TOP_LEFT, 2.0, 24.0),
            (props::RECTANGLE_CORNER_RADIUS_TOP_RIGHT, 2.0, 24.0),
            (props::RECTANGLE_CORNER_RADIUS_BOTTOM_RIGHT, 2.0, 24.0),
            (props::RECTANGLE_CORNER_RADIUS_BOTTOM_LEFT, 2.0, 24.0),
        ] {
            animations.set_key(animation, rectangle, property, 0, from)?;
            animations.set_key(animation, rectangle, property, 60, to)?;
        }
        animations.set_key(animation, stroke, props::STROKE_THICKNESS, 0, 0.0)?;
        animations.set_key(animation, stroke, props::STROKE_THICKNESS, 60, 6.0)?;
        animations.set_color_key(animation, color, props::COLOR_VALUE, 0, 0xff3b_82f6)?;
        animations.set_color_key_with_interpolation(
            animation,
            color,
            props::COLOR_VALUE,
            60,
            0xffef_4444,
            KeyInterpolation::Linear,
        )?;
        Ok((artboard, rectangle, color, stroke, animation))
    })?;

    assert!(
        scene
            .export_records()
            .records()
            .iter()
            .any(|record| record.kind == ExportedObjectKind::KeyFrameColor),
        "typed color keys must export the Rive color-keyframe record kind"
    );

    let instance = scene.instantiate(artboard)?;
    let width = scene.cursor(instance, rectangle, props::PATH_WIDTH)?;
    let height = scene.cursor(instance, rectangle, props::PATH_HEIGHT)?;
    let top_left = scene.cursor(instance, rectangle, props::RECTANGLE_CORNER_RADIUS_TOP_LEFT)?;
    let bottom_right = scene.cursor(
        instance,
        rectangle,
        props::RECTANGLE_CORNER_RADIUS_BOTTOM_RIGHT,
    )?;
    let thickness = scene.cursor(instance, stroke, props::STROKE_THICKNESS)?;
    let fill_color = scene.cursor(instance, color, props::COLOR_VALUE)?;
    scene.frame().scrub(instance, animation, 1.0)?;
    assert_eq!(scene.frame().get(width)?, 160.0);
    assert_eq!(scene.frame().get(height)?, 68.0);
    assert_eq!(scene.frame().get(top_left)?, 24.0);
    assert_eq!(scene.frame().get(bottom_right)?, 24.0);
    assert_eq!(scene.frame().get(thickness)?, 6.0);
    assert_eq!(scene.frame().get(fill_color)?, 0xffef_4444);
    Ok(())
}

#[test]
fn authored_cubic_ease_key_upserts_identity_and_changes_quarter_progress() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, linear_shape, eased_shape, linear, eased, eased_start), _) =
        scene.edit(|tx| {
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Canvas".into(),
                width: 100.0,
                height: 100.0,
            })?;
            let make_shape = |tx: &mut SceneTx<'_>, name: &str| {
                tx.create(
                    Parent::Artboard(artboard),
                    NodeSpec::Shape(ShapeSpec {
                        name: name.into(),
                        x: 0.0,
                        y: 0.0,
                        opacity: 1.0,
                        rotation: 0.0,
                        scale_x: 1.0,
                        scale_y: 1.0,
                    }),
                )
            };
            let linear_shape = make_shape(tx, "Linear")?;
            let eased_shape = make_shape(tx, "Smooth")?;
            let make_animation = |tx: &mut SceneTx<'_>, name: &str| {
                tx.animations().create_linear(
                    artboard,
                    LinearAnimationSpec {
                        name: name.into(),
                        fps: 60,
                        duration: 60,
                    },
                )
            };
            let linear = make_animation(tx, "Linear")?;
            tx.animations()
                .set_key(linear, linear_shape, props::WORLD_OPACITY, 0, 0.0)?;
            tx.animations()
                .set_key(linear, linear_shape, props::WORLD_OPACITY, 60, 1.0)?;

            let eased = make_animation(tx, "Smooth")?;
            let eased_start =
                tx.animations()
                    .set_key(eased, eased_shape, props::WORLD_OPACITY, 0, 0.0)?;
            let upserted = tx.animations().set_key_with_interpolation(
                eased,
                eased_shape,
                props::WORLD_OPACITY,
                0,
                0.0,
                KeyInterpolation::CubicEase {
                    x1: 0.42,
                    y1: 0.0,
                    x2: 0.58,
                    y2: 1.0,
                },
            )?;
            assert_eq!(upserted, eased_start);
            tx.animations()
                .set_key(eased, eased_shape, props::WORLD_OPACITY, 60, 1.0)?;
            Ok((
                artboard,
                linear_shape,
                eased_shape,
                linear,
                eased,
                eased_start,
            ))
        })?;

    let records = scene.export_records();
    let cubic_index = records
        .records()
        .iter()
        .position(|record| record.kind == ExportedObjectKind::CubicEaseInterpolator)
        .expect("one cubic interpolator is derived for the eased outgoing key");
    assert_eq!(
        records
            .records()
            .iter()
            .filter(|record| record.kind == ExportedObjectKind::CubicEaseInterpolator)
            .count(),
        1,
        "upserting a key must not accumulate helper records"
    );
    assert_eq!(
        records
            .records()
            .get(cubic_index)
            .expect("derived cubic record")
            .properties,
        vec![
            ExportedProperty::CubicEaseX1(0.42),
            ExportedProperty::CubicEaseY1(0.0),
            ExportedProperty::CubicEaseX2(0.58),
            ExportedProperty::CubicEaseY2(1.0),
        ]
    );
    let eased_key_record = records
        .records()
        .get(cubic_index + 1)
        .expect("derived interpolator is immediately followed by its keyframe");
    assert_eq!(eased_key_record.kind, ExportedObjectKind::KeyFrameDouble);
    assert!(
        eased_key_record
            .properties
            .contains(&ExportedProperty::KeyFrameInterpolatorId(3))
    );
    assert!(
        eased_key_record
            .properties
            .contains(&ExportedProperty::KeyFrameInterpolationCubic)
    );

    let instance = scene.instantiate(artboard)?;
    let linear_opacity = scene.cursor(instance, linear_shape, props::WORLD_OPACITY)?;
    let eased_opacity = scene.cursor(instance, eased_shape, props::WORLD_OPACITY)?;
    scene.frame().scrub(instance, linear, 0.25)?;
    scene.frame().scrub(instance, eased, 0.25)?;
    let linear_quarter = scene.frame().get(linear_opacity)?;
    let eased_quarter = scene.frame().get(eased_opacity)?;
    assert!((linear_quarter - 0.25).abs() < 0.000_01);
    assert!(
        eased_quarter < 0.2 && (eased_quarter - linear_quarter).abs() > 0.05,
        "smooth cubic quarter progress {eased_quarter} must differ from linear {linear_quarter}"
    );

    scene.edit(|tx| {
        let id = tx.animations().set_key_with_interpolation(
            eased,
            eased_shape,
            props::WORLD_OPACITY,
            0,
            0.0,
            KeyInterpolation::CubicEase {
                x1: 0.42,
                y1: 0.0,
                x2: 0.58,
                y2: 1.0,
            },
        )?;
        assert_eq!(id, eased_start);
        Ok(())
    })?;
    assert_eq!(scene.export_records(), records);
    Ok(())
}

#[test]
fn cubic_ease_control_points_fail_closed_without_mutating_the_key() -> Result<()> {
    let mut scene = Scene::new();
    let ((animation, shape), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Canvas".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Fader".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let animation = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Fade".into(),
                fps: 60,
                duration: 60,
            },
        )?;
        tx.animations()
            .set_key(animation, shape, props::WORLD_OPACITY, 0, 0.0)?;
        tx.animations()
            .set_key(animation, shape, props::WORLD_OPACITY, 60, 1.0)?;
        Ok((animation, shape))
    })?;
    let records_before = scene.export_records();
    let epoch_before = scene.epoch();

    for (interpolation, expected) in [
        (
            KeyInterpolation::CubicEase {
                x1: f32::NAN,
                y1: 0.0,
                x2: 0.58,
                y2: 1.0,
            },
            EditReason::NonFiniteProperty {
                property: "cubic_ease_x1",
            },
        ),
        (
            KeyInterpolation::CubicEase {
                x1: 0.42,
                y1: f32::INFINITY,
                x2: 0.58,
                y2: 1.0,
            },
            EditReason::NonFiniteProperty {
                property: "cubic_ease_y1",
            },
        ),
        (
            KeyInterpolation::CubicEase {
                x1: -0.01,
                y1: 0.0,
                x2: 0.58,
                y2: 1.0,
            },
            EditReason::OutOfRangeProperty {
                property: "cubic_ease_x1",
            },
        ),
        (
            KeyInterpolation::CubicEase {
                x1: 0.42,
                y1: 0.0,
                x2: 1.01,
                y2: 1.0,
            },
            EditReason::OutOfRangeProperty {
                property: "cubic_ease_x2",
            },
        ),
    ] {
        let error = scene
            .edit(|tx| {
                tx.animations().set_key_with_interpolation(
                    animation,
                    shape,
                    props::WORLD_OPACITY,
                    0,
                    0.0,
                    interpolation,
                )
            })
            .expect_err("invalid cubic control points must abort the transaction");
        assert_eq!(error.kind(), EditErrorKind::Aborted);
        assert_eq!(error.diagnostic().reason, expected);
        assert_eq!(scene.export_records(), records_before);
        assert_eq!(scene.epoch(), epoch_before);
    }
    Ok(())
}

#[test]
fn authored_trigger_machine_exports_one_canonical_executable_record_graph() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, machine), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Canvas".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Fader".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let idle = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Idle".into(),
                fps: 60,
                duration: 1,
            },
        )?;
        tx.animations()
            .set_key(idle, shape, props::WORLD_OPACITY, 0, 0.2)?;
        let active = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Active".into(),
                fps: 60,
                duration: 1,
            },
        )?;
        tx.animations()
            .set_key(active, shape, props::WORLD_OPACITY, 0, 0.8)?;

        let mut machines = tx.machines();
        let reached = machines.create_event(artboard, EventSpec { name: None })?;
        let machine = machines.create_machine(artboard, MachineSpec { name: None })?;
        let go = machines.create_trigger_input(machine, TriggerInputSpec { name: "Go".into() })?;
        let layer = machines.create_layer(machine, MachineLayerSpec { name: None })?;
        let entry = machines.create_entry_state(layer)?;
        let any = machines.create_any_state(layer)?;
        machines.create_exit_state(layer)?;
        let idle_state =
            machines.create_animation_state(layer, AnimationStateSpec { animation: idle })?;
        let active_state =
            machines.create_animation_state(layer, AnimationStateSpec { animation: active })?;
        machines.create_transition(entry, idle_state)?;
        let transition = machines.create_transition(any, active_state)?;
        machines.add_trigger_condition(transition, go)?;
        machines.add_fire_event(active_state, reached, FireEventOccurs::AtStart)?;
        Ok((artboard, machine))
    })?;

    assert_eq!(
        scene
            .export_records()
            .records()
            .iter()
            .map(|record| record.kind)
            .collect::<Vec<_>>(),
        vec![
            ExportedObjectKind::Backboard,
            ExportedObjectKind::Artboard,
            ExportedObjectKind::Shape,
            ExportedObjectKind::Event,
            ExportedObjectKind::LinearAnimation,
            ExportedObjectKind::KeyedObject,
            ExportedObjectKind::KeyedProperty,
            ExportedObjectKind::KeyFrameDouble,
            ExportedObjectKind::LinearAnimation,
            ExportedObjectKind::KeyedObject,
            ExportedObjectKind::KeyedProperty,
            ExportedObjectKind::KeyFrameDouble,
            ExportedObjectKind::StateMachine,
            ExportedObjectKind::StateMachineTrigger,
            ExportedObjectKind::StateMachineLayer,
            ExportedObjectKind::EntryState,
            ExportedObjectKind::StateTransition,
            ExportedObjectKind::AnyState,
            ExportedObjectKind::StateTransition,
            ExportedObjectKind::TransitionTriggerCondition,
            ExportedObjectKind::ExitState,
            ExportedObjectKind::AnimationState,
            ExportedObjectKind::AnimationState,
            ExportedObjectKind::StateMachineFireEvent,
        ]
    );
    assert_eq!(
        &scene.export_records().records()[3..],
        &[
            ExportedRecord {
                kind: ExportedObjectKind::Event,
                properties: vec![],
            },
            ExportedRecord {
                kind: ExportedObjectKind::LinearAnimation,
                properties: vec![
                    ExportedProperty::AnimationName("Idle".into()),
                    ExportedProperty::AnimationDuration(1),
                ],
            },
            ExportedRecord {
                kind: ExportedObjectKind::KeyedObject,
                properties: vec![ExportedProperty::KeyedObjectId(1)],
            },
            ExportedRecord {
                kind: ExportedObjectKind::KeyedProperty,
                properties: vec![ExportedProperty::KeyedProperty(
                    ExportedAnimatableProperty::WorldOpacity,
                )],
            },
            ExportedRecord {
                kind: ExportedObjectKind::KeyFrameDouble,
                properties: vec![
                    ExportedProperty::KeyFrame(0),
                    ExportedProperty::KeyFrameInterpolationLinear,
                    ExportedProperty::KeyFrameDoubleValue(0.2),
                ],
            },
            ExportedRecord {
                kind: ExportedObjectKind::LinearAnimation,
                properties: vec![
                    ExportedProperty::AnimationName("Active".into()),
                    ExportedProperty::AnimationDuration(1),
                ],
            },
            ExportedRecord {
                kind: ExportedObjectKind::KeyedObject,
                properties: vec![ExportedProperty::KeyedObjectId(1)],
            },
            ExportedRecord {
                kind: ExportedObjectKind::KeyedProperty,
                properties: vec![ExportedProperty::KeyedProperty(
                    ExportedAnimatableProperty::WorldOpacity,
                )],
            },
            ExportedRecord {
                kind: ExportedObjectKind::KeyFrameDouble,
                properties: vec![
                    ExportedProperty::KeyFrame(0),
                    ExportedProperty::KeyFrameInterpolationLinear,
                    ExportedProperty::KeyFrameDoubleValue(0.8),
                ],
            },
            ExportedRecord {
                kind: ExportedObjectKind::StateMachine,
                properties: vec![],
            },
            ExportedRecord {
                kind: ExportedObjectKind::StateMachineTrigger,
                properties: vec![ExportedProperty::StateMachineComponentName("Go".into())],
            },
            ExportedRecord {
                kind: ExportedObjectKind::StateMachineLayer,
                properties: vec![],
            },
            ExportedRecord {
                kind: ExportedObjectKind::EntryState,
                properties: vec![],
            },
            ExportedRecord {
                kind: ExportedObjectKind::StateTransition,
                properties: vec![
                    ExportedProperty::StateToId(3),
                    ExportedProperty::StateTransitionFlags(0),
                    ExportedProperty::StateTransitionDuration(0),
                    ExportedProperty::StateTransitionExitTime(0),
                    ExportedProperty::StateTransitionRandomWeight(1),
                ],
            },
            ExportedRecord {
                kind: ExportedObjectKind::AnyState,
                properties: vec![],
            },
            ExportedRecord {
                kind: ExportedObjectKind::StateTransition,
                properties: vec![
                    ExportedProperty::StateToId(4),
                    ExportedProperty::StateTransitionFlags(0),
                    ExportedProperty::StateTransitionDuration(0),
                    ExportedProperty::StateTransitionExitTime(0),
                    ExportedProperty::StateTransitionRandomWeight(1),
                ],
            },
            ExportedRecord {
                kind: ExportedObjectKind::TransitionTriggerCondition,
                properties: vec![ExportedProperty::StateMachineInputId(0)],
            },
            ExportedRecord {
                kind: ExportedObjectKind::ExitState,
                properties: vec![],
            },
            ExportedRecord {
                kind: ExportedObjectKind::AnimationState,
                properties: vec![
                    ExportedProperty::StateAnimationId(0),
                    ExportedProperty::StateSpeed(1.0),
                ],
            },
            ExportedRecord {
                kind: ExportedObjectKind::AnimationState,
                properties: vec![
                    ExportedProperty::StateAnimationId(1),
                    ExportedProperty::StateSpeed(1.0),
                ],
            },
            ExportedRecord {
                kind: ExportedObjectKind::StateMachineFireEvent,
                properties: vec![
                    ExportedProperty::EventId(2),
                    ExportedProperty::FireEventOccurs(FireEventOccurs::AtStart),
                ],
            },
        ]
    );

    let instance = scene.instantiate(artboard)?;
    let input = scene.machine_input(instance, machine, "Go")?;
    let mut events = Vec::new();
    let _ = scene.frame().advance(instance, 0.0, &mut events);
    scene.frame().fire(input)?;
    let _ = scene.frame().advance(instance, 0.0, &mut events);
    assert_eq!(events.len(), 1);
    match &events[0] {
        SceneEvent::Authored { name, .. } => {
            assert_eq!(
                name, &None,
                "an omitted event name stays semantically absent"
            );
        }
        _ => panic!("authored machine must emit an authored event"),
    }
    Ok(())
}

#[test]
fn authored_view_model_number_binds_state_transition_duration_records() -> Result<()> {
    let mut scene = Scene::new();
    scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Canvas".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Fader".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let idle = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Idle".into(),
                fps: 60,
                duration: 1,
            },
        )?;
        tx.animations()
            .set_key(idle, shape, props::WORLD_OPACITY, 0, 0.2)?;
        let active = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Active".into(),
                fps: 60,
                duration: 1,
            },
        )?;
        tx.animations()
            .set_key(active, shape, props::WORLD_OPACITY, 0, 0.8)?;

        let (transition, go) = {
            let mut machines = tx.machines();
            let machine = machines.create_machine(
                artboard,
                MachineSpec {
                    name: Some("Switcher".into()),
                },
            )?;
            let go =
                machines.create_trigger_input(machine, TriggerInputSpec { name: "Go".into() })?;
            let layer = machines.create_layer(machine, MachineLayerSpec { name: None })?;
            let entry = machines.create_entry_state(layer)?;
            let any = machines.create_any_state(layer)?;
            machines.create_exit_state(layer)?;
            let idle_state =
                machines.create_animation_state(layer, AnimationStateSpec { animation: idle })?;
            let active_state =
                machines.create_animation_state(layer, AnimationStateSpec { animation: active })?;
            machines.create_transition(entry, idle_state)?;
            let transition = machines.create_transition(any, active_state)?;
            (transition, go)
        };

        {
            let mut view_models = tx.view_models();
            let model = view_models.create(ViewModelSpec {
                scope: ViewModelScope::Local,
                name: "Playback".into(),
            })?;
            let duration = view_models.create_number(
                model,
                ViewModelNumberSpec {
                    name: "Duration".into(),
                },
            )?;
            let defaults = view_models.create_instance(
                model,
                ViewModelInstanceSpec {
                    name: Some("Defaults".into()),
                },
            )?;
            view_models.set_number(defaults, duration, 0.0)?;
            view_models.set_artboard_default(artboard, defaults)?;
            view_models.bind_transition_duration(transition, duration)?;
        }
        tx.machines().add_trigger_condition(transition, go)?;
        Ok(())
    })?;

    let records = scene.export_records();
    assert_eq!(
        &records.records()[..5],
        &[
            ExportedRecord {
                kind: ExportedObjectKind::Backboard,
                properties: vec![],
            },
            ExportedRecord {
                kind: ExportedObjectKind::ViewModel,
                properties: vec![ExportedProperty::ViewModelName("Playback".into())],
            },
            ExportedRecord {
                kind: ExportedObjectKind::ViewModelPropertyNumber,
                properties: vec![ExportedProperty::ViewModelName("Duration".into())],
            },
            ExportedRecord {
                kind: ExportedObjectKind::ViewModelInstance,
                properties: vec![
                    ExportedProperty::ComponentName("Defaults".into()),
                    ExportedProperty::ViewModelId(0),
                ],
            },
            ExportedRecord {
                kind: ExportedObjectKind::ViewModelInstanceNumber,
                properties: vec![
                    ExportedProperty::ViewModelPropertyId(0),
                    ExportedProperty::ViewModelNumberValue(0.0),
                ],
            },
        ]
    );
    let artboard = records
        .records()
        .iter()
        .find(|record| record.kind == ExportedObjectKind::Artboard)
        .expect("artboard record");
    assert!(
        artboard
            .properties
            .contains(&ExportedProperty::ArtboardViewModelId(0))
    );

    let transition = records
        .records()
        .iter()
        .position(|record| record.kind == ExportedObjectKind::StateTransition)
        .expect("entry transition");
    let bound_transition = records
        .records()
        .iter()
        .enumerate()
        .skip(transition + 1)
        .find(|(_, record)| record.kind == ExportedObjectKind::StateTransition)
        .map(|(index, _)| index)
        .expect("bound transition");
    assert_eq!(
        records.records().get(bound_transition + 1),
        Some(&ExportedRecord {
            kind: ExportedObjectKind::DataBindContext,
            properties: vec![
                ExportedProperty::DataBindPropertyKey(158),
                ExportedProperty::DataBindFlags(0),
                ExportedProperty::DataBindSourcePath(vec![0, 0]),
            ],
        })
    );
    assert_eq!(
        records
            .records()
            .get(bound_transition + 2)
            .map(|record| record.kind),
        Some(ExportedObjectKind::TransitionTriggerCondition),
        "the transition-owned data bind precedes conditions"
    );
    Ok(())
}

#[test]
fn typed_vertical_component_list_exports_imports_advances_and_draws_two_view_model_items()
-> Result<()> {
    let mut scene = Scene::new();
    let ((root_artboard, item_shape, component_list, pressed), _) = scene.edit(|tx| {
        let root_artboard = tx.create_artboard(ArtboardSpec {
            name: "Root".into(),
            width: 120.0,
            height: 70.0,
        })?;
        let item_artboard = tx.create_artboard(ArtboardSpec {
            name: "Item".into(),
            width: 20.0,
            height: 20.0,
        })?;
        let item_shape = tx.create(
            Parent::Artboard(item_artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Item shape".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Object(item_shape),
            NodeSpec::Rectangle(RectangleSpec::new("Item bounds", 20.0, 20.0)),
        )?;
        let fill = tx.create(
            Parent::Object(item_shape),
            NodeSpec::Fill(FillSpec {
                name: "Item fill".into(),
            }),
        )?;
        tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Item color".into(),
                color: 0xffab_cdef,
            }),
        )?;

        let (root_model, item_model, root_defaults, item_a, item_b, root_items) = {
            let mut view_models = tx.view_models();
            view_models.create(ViewModelSpec {
                scope: ViewModelScope::Local,
                name: "Decoy".into(),
            })?;
            let root_model = view_models.create(ViewModelSpec {
                scope: ViewModelScope::Local,
                name: "Root model".into(),
            })?;
            let item_model = view_models.create(ViewModelSpec {
                scope: ViewModelScope::Local,
                name: "Item model".into(),
            })?;
            let root_items = view_models.create_list(
                root_model,
                ViewModelListSpec {
                    name: "items".into(),
                },
            )?;
            let root_defaults = view_models.create_instance(
                root_model,
                ViewModelInstanceSpec {
                    name: Some("Root defaults".into()),
                },
            )?;
            let item_a = view_models.create_instance(
                item_model,
                ViewModelInstanceSpec {
                    name: Some("Item A".into()),
                },
            )?;
            let item_b = view_models.create_instance(
                item_model,
                ViewModelInstanceSpec {
                    name: Some("Item B".into()),
                },
            )?;
            view_models.set_list_items(root_defaults, root_items, &[item_a, item_b])?;
            view_models.set_artboard_default(root_artboard, root_defaults)?;
            view_models.set_artboard_default(item_artboard, item_a)?;
            (
                root_model,
                item_model,
                root_defaults,
                item_a,
                item_b,
                root_items,
            )
        };
        let _ = (root_model, root_defaults, item_a, item_b);

        let component_list = tx.create_component_list(
            root_artboard,
            ArtboardComponentListSpec {
                name: "Items".into(),
                x: 5.0,
                y: 7.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                flow: Some(ArtboardComponentListFlow {
                    axis: ArtboardComponentListAxis::Vertical,
                    reverse: false,
                    gap: 6.0,
                }),
                source: ViewModelListSource::direct(root_items),
                map_rules: vec![ArtboardListMapRuleSpec {
                    view_model: item_model,
                    artboard: item_artboard,
                    state_machines: Vec::new(),
                }],
            },
        )?;
        // The machine's transparent root-artboard hit target recognizes the
        // press, while the context must identify the concrete rendered list
        // item under that press rather than this implementation-only proxy.
        let listener_target = tx.create(
            Parent::Artboard(root_artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Second item listener target".into(),
                x: 10.0,
                y: 36.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Object(listener_target),
            NodeSpec::Rectangle(RectangleSpec::new("Second item hit bounds", 20.0, 20.0)),
        )?;
        let mut machines = tx.machines();
        let pressed = machines.create_event(
            root_artboard,
            EventSpec {
                name: Some("List item pressed".into()),
            },
        )?;
        let machine = machines.create_machine(
            root_artboard,
            MachineSpec {
                name: Some("List interactions".into()),
            },
        )?;
        let layer = machines.create_layer(machine, MachineLayerSpec { name: None })?;
        machines.create_any_state(layer)?;
        machines.create_entry_state(layer)?;
        machines.create_exit_state(layer)?;
        let listener = machines.create_listener(
            machine,
            listener_target,
            MachineListenerSpec {
                name: Some("Press second item".into()),
                listener_type: MachineListenerType::Click,
            },
        )?;
        machines.add_listener_fire_event_action(listener, pressed)?;
        Ok((root_artboard, item_shape, component_list, pressed))
    })?;

    let records = scene.export_records();
    assert!(records.records().iter().any(|record| {
        record.kind == ExportedObjectKind::ViewModelPropertyList
            && record
                .properties
                .contains(&ExportedProperty::ViewModelName("items".into()))
    }));
    let list_items = records
        .records()
        .iter()
        .filter(|record| record.kind == ExportedObjectKind::ViewModelInstanceListItem)
        .map(|record| {
            let model = record
                .properties
                .iter()
                .find_map(|property| match property {
                    ExportedProperty::ViewModelListItemViewModelId(value) => Some(*value),
                    _ => None,
                });
            let instance = record
                .properties
                .iter()
                .find_map(|property| match property {
                    ExportedProperty::ViewModelListItemInstanceId(value) => Some(*value),
                    _ => None,
                });
            (model, instance)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        list_items,
        vec![(Some(2), Some(0)), (Some(2), Some(1))],
        "each list item resolves its actual model and per-model instance ordinals"
    );
    let list_index = records
        .records()
        .iter()
        .position(|record| record.kind == ExportedObjectKind::ArtboardComponentList)
        .expect("typed component-list host is exported");
    let list = &records.records()[list_index];
    assert!(list.properties.contains(&ExportedProperty::ParentId(1)));
    assert!(list.properties.contains(&ExportedProperty::TranslateX(5.0)));
    assert!(list.properties.contains(&ExportedProperty::TranslateY(7.0)));
    assert!(
        !list
            .properties
            .iter()
            .any(|property| matches!(property, ExportedProperty::LayoutComponentStyleId(_))),
        "ArtboardComponentList is a Drawable and must not receive LayoutComponent.styleId",
    );
    let layout_component = records
        .records()
        .iter()
        .find(|record| record.kind == ExportedObjectKind::LayoutComponent)
        .expect("typed component-list flow exports one native layout wrapper");
    assert!(
        !layout_component.properties.iter().any(|property| matches!(
            property,
            ExportedProperty::TranslateX(_) | ExportedProperty::TranslateY(_)
        )),
        "the durable component-list record continues to own controller-visible transforms",
    );
    let style_local_id = layout_component
        .properties
        .iter()
        .find_map(|property| match property {
            ExportedProperty::LayoutComponentStyleId(local_id) => Some(*local_id),
            _ => None,
        })
        .expect("the native layout wrapper references its style");
    assert_eq!(
        records.records().get(list_index + 1),
        Some(&ExportedRecord {
            kind: ExportedObjectKind::DataBindContext,
            properties: vec![
                ExportedProperty::DataBindArtboardComponentListSource,
                ExportedProperty::DataBindFlags(0),
                ExportedProperty::DataBindSourcePath(vec![1, 0]),
            ],
        }),
        "the typed source bind immediately follows its target without exposing a raw schema key"
    );
    let layout = records
        .records()
        .iter()
        .find(|record| record.kind == ExportedObjectKind::LayoutComponentStyle)
        .expect("typed component-list flow exports one native layout style");
    assert!(layout.properties.contains(&ExportedProperty::ParentId(1)));
    assert_eq!(style_local_id, 5);
    assert!(
        layout
            .properties
            .contains(&ExportedProperty::LayoutFlexDirection(0))
    );
    assert!(
        layout
            .properties
            .contains(&ExportedProperty::LayoutGapVertical(6.0))
    );
    let map_rule = records
        .records()
        .iter()
        .find(|record| record.kind == ExportedObjectKind::ArtboardListMapRule)
        .expect("typed list map rule is exported");
    assert!(
        map_rule
            .properties
            .contains(&ExportedProperty::ArtboardListMapRuleViewModelId(2)),
        "the map rule resolves the item model ordinal instead of hard-coding zero"
    );
    assert!(
        map_rule
            .properties
            .contains(&ExportedProperty::ArtboardListMapRuleArtboardId(1))
    );

    let instance = scene.instantiate(root_artboard)?;
    let mut events = Vec::new();
    let _ = scene.frame().advance(instance, 0.0, &mut events);
    assert!(events.is_empty());
    let stream = canonical_draw_stream(&mut scene, instance)?;
    assert_eq!(
        drawn_move_count(&stream),
        2,
        "both list contexts import, instantiate the mapped item artboard, and draw"
    );
    let occurrences = [
        (nuxie::Vec2D::new(6.0, 8.0), 0),
        (nuxie::Vec2D::new(6.0, 34.0), 1),
    ];
    for (point, item_index) in occurrences {
        let hits = scene.frame().hit_test_paths_with_bounds(instance, point);
        assert_eq!(hits.len(), 1);
        assert_eq!(
            hits[0].path().objects(),
            [component_list.object_id(), item_shape]
        );
        assert_eq!(hits[0].occurrence().len(), 1);
        assert_eq!(hits[0].occurrence()[0].host(), component_list.object_id());
        assert_eq!(hits[0].occurrence()[0].item_index(), item_index);
        assert_eq!(scene.frame().hit_test(instance, point), vec![item_shape]);
    }

    assert!(
        scene
            .frame()
            .hit_test_paths_with_bounds(instance, nuxie::Vec2D::new(26.0, 8.0))
            .is_empty(),
        "vertical flow must not leave the second item on the legacy horizontal axis",
    );
    let second_item = nuxie::Vec2D::new(6.0, 34.0);
    assert!(scene.frame().pointer_down(instance, second_item, 42));
    assert!(scene.frame().pointer_up(instance, second_item, 42));
    assert!(scene.frame().advance(instance, 0.0, &mut events));
    let [
        SceneEvent::Authored {
            event,
            context: Some(context),
            ..
        },
    ] = events.as_slice()
    else {
        panic!("list press must report one exact rendered occurrence: {events:?}");
    };
    assert_eq!(*event, pressed);
    assert_eq!(
        context.path().objects(),
        [component_list.object_id(), item_shape]
    );
    assert_eq!(context.occurrence().len(), 1);
    assert_eq!(context.occurrence()[0].host(), component_list.object_id());
    assert_eq!(context.occurrence()[0].item_index(), 1);
    Ok(())
}

#[test]
fn nested_view_model_list_path_imports_advances_and_draws_the_mapped_item() -> Result<()> {
    let mut scene = Scene::new();
    let ((root_artboard, item_artboard), _) = scene.edit(|tx| {
        let font = tx.create_font_asset(FontAssetSpec {
            name: "Roboto".into(),
            bytes: fixture_font_bytes(),
        })?;
        let root_artboard = tx.create_artboard(ArtboardSpec {
            name: "Root".into(),
            width: 120.0,
            height: 40.0,
        })?;
        let item_artboard = tx.create_artboard(ArtboardSpec {
            name: "Item".into(),
            width: 20.0,
            height: 20.0,
        })?;
        let item_text = tx.create(
            Parent::Artboard(item_artboard),
            NodeSpec::Text(TextSpec {
                name: "Product name".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                width: 120.0,
                height: 30.0,
                sizing: SceneTextSizing::Fixed,
                align: SceneTextAlign::Left,
                wrap: SceneTextWrap::NoWrap,
                overflow: SceneTextOverflow::Visible,
            }),
        )?;
        let style = tx.create(
            Parent::Object(item_text),
            NodeSpec::TextStylePaint(TextStylePaintSpec {
                name: "Product style".into(),
                font_size: 18.0,
                line_height: 22.0,
                letter_spacing: 0.0,
                font,
            }),
        )?;
        let paint = tx.create(
            Parent::Object(style),
            NodeSpec::Fill(FillSpec {
                name: "Product fill".into(),
            }),
        )?;
        tx.create(
            Parent::Object(paint),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Item color".into(),
                color: 0xffab_cdef,
            }),
        )?;
        let item_run = tx.create(
            Parent::Object(item_text),
            NodeSpec::TextValueRun(TextValueRunSpec {
                name: "Product run".into(),
                text: String::new(),
                style,
            }),
        )?;

        let (item_model, paywall, products) = {
            let mut view_models = tx.view_models();
            view_models.create(ViewModelSpec {
                scope: ViewModelScope::Local,
                name: "Seed model".into(),
            })?;
            let root_model = view_models.create(ViewModelSpec {
                scope: ViewModelScope::Local,
                name: "Root model".into(),
            })?;
            let paywall_model = view_models.create(ViewModelSpec {
                scope: ViewModelScope::Local,
                name: "Paywall model".into(),
            })?;
            let item_model = view_models.create(ViewModelSpec {
                scope: ViewModelScope::Local,
                name: "Product model".into(),
            })?;
            view_models.create_number(
                root_model,
                ViewModelNumberSpec {
                    name: "root seed".into(),
                },
            )?;
            view_models.create_number(
                paywall_model,
                ViewModelNumberSpec {
                    name: "paywall seed".into(),
                },
            )?;
            view_models.create_number(
                item_model,
                ViewModelNumberSpec {
                    name: "product seed".into(),
                },
            )?;
            let product_name = view_models.create_string(
                item_model,
                ViewModelStringSpec {
                    name: "name".into(),
                },
            )?;
            let paywall = view_models.create_child(
                root_model,
                ViewModelChildSpec {
                    name: "paywall".into(),
                    view_model: paywall_model,
                },
            )?;
            let products = view_models.create_list(
                paywall_model,
                ViewModelListSpec {
                    name: "products".into(),
                },
            )?;
            let root_defaults = view_models.create_instance(
                root_model,
                ViewModelInstanceSpec {
                    name: Some("Root defaults".into()),
                },
            )?;
            let paywall_defaults = view_models.create_instance(
                paywall_model,
                ViewModelInstanceSpec {
                    name: Some("Paywall defaults".into()),
                },
            )?;
            let empty_item_defaults = view_models.create_instance(
                item_model,
                ViewModelInstanceSpec {
                    name: Some("Empty product defaults".into()),
                },
            )?;
            let populated_item = view_models.create_instance(
                item_model,
                ViewModelInstanceSpec {
                    name: Some("Populated product".into()),
                },
            )?;
            view_models.set_child(root_defaults, paywall, paywall_defaults)?;
            view_models.set_list_items(paywall_defaults, products, &[populated_item])?;
            // The compact fixture font contains only the `a` glyph. The exact
            // string-record test below separately pins arbitrary UTF-8 bytes.
            view_models.set_string(populated_item, product_name, "a")?;
            let product_name_by_alias = view_models
                .string_by_name(item_model, "name")
                .expect("the document alias resolves by authored name");
            assert_eq!(product_name_by_alias, product_name);
            assert_eq!(
                view_models.string_by_name(item_model, "2369371622"),
                None,
                "a ProjectDO provenance hash is not a runtime property identity"
            );
            view_models.set_artboard_default(root_artboard, root_defaults)?;
            view_models.set_artboard_default(item_artboard, empty_item_defaults)?;
            view_models.bind_text(item_run, product_name_by_alias)?;
            (item_model, paywall, products)
        };

        tx.create_component_list(
            root_artboard,
            ArtboardComponentListSpec {
                name: "Products".into(),
                x: 5.0,
                y: 7.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                flow: None,
                source: ViewModelListSource::nested([paywall], products),
                map_rules: vec![ArtboardListMapRuleSpec {
                    view_model: item_model,
                    artboard: item_artboard,
                    state_machines: Vec::new(),
                }],
            },
        )?;
        Ok((root_artboard, item_artboard))
    })?;

    let records = scene.export_records();
    let list_index = records
        .records()
        .iter()
        .position(|record| record.kind == ExportedObjectKind::ArtboardComponentList)
        .expect("nested component-list host is exported");
    assert_eq!(
        records.records().get(list_index + 1),
        Some(&ExportedRecord {
            kind: ExportedObjectKind::DataBindContext,
            properties: vec![
                ExportedProperty::DataBindArtboardComponentListSource,
                ExportedProperty::DataBindFlags(0),
                ExportedProperty::DataBindSourcePath(vec![1, 1, 1]),
            ],
        }),
        "the type-447 component-list bind retains every nonzero semantic ordinal"
    );
    let run_index = records
        .records()
        .iter()
        .position(|record| record.kind == ExportedObjectKind::TextValueRun)
        .expect("bound item TextValueRun is exported");
    assert_eq!(
        records.records().get(run_index + 1),
        Some(&ExportedRecord {
            kind: ExportedObjectKind::DataBindContext,
            properties: vec![
                ExportedProperty::DataBindTextValueRunTextTarget,
                ExportedProperty::DataBindFlags(0),
                ExportedProperty::DataBindSourcePath(vec![3, 1]),
            ],
        }),
        "the type-447 text bind retains its target, flags, and nonzero source ordinals"
    );
    let direct_item = scene.instantiate(item_artboard)?;
    let mut direct_events = Vec::new();
    let _ = scene.frame().advance(direct_item, 0.0, &mut direct_events);
    let direct_stream = canonical_draw_stream(&mut scene, direct_item)?;
    assert_eq!(
        drawn_move_count(&direct_stream),
        0,
        "the empty item default must not hide a broken list-item context: {direct_stream}"
    );
    let instance = scene.instantiate(root_artboard)?;
    let mut events = Vec::new();
    let _ = scene.frame().advance(instance, 0.0, &mut events);
    let stream = canonical_draw_stream(&mut scene, instance)?;
    assert!(
        drawn_move_count(&stream) > 0,
        "the mapped item must draw glyph paths from its bound string: {stream}"
    );
    Ok(())
}

#[test]
fn typed_list_string_equality_mutates_stable_items_rejects_mismatches_and_replays() -> Result<()> {
    let mut scene = Scene::new();
    let (
        (
            root_artboard,
            paywall_defaults,
            selected_product,
            paywall,
            products,
            product_id,
            is_selected,
            selected_badge,
            product_list,
        ),
        _,
    ) = scene.edit(|tx| {
        let root_artboard = tx.create_artboard(ArtboardSpec {
            name: "Root".into(),
            width: 120.0,
            height: 40.0,
        })?;
        let item_artboard = tx.create_artboard(ArtboardSpec {
            name: "Product".into(),
            width: 20.0,
            height: 20.0,
        })?;
        let selected_badge = tx.create(
            Parent::Artboard(item_artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Selected badge".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Object(selected_badge),
            NodeSpec::Rectangle(RectangleSpec::new("Badge bounds", 12.0, 12.0)),
        )?;
        let fill = tx.create(
            Parent::Object(selected_badge),
            NodeSpec::Fill(FillSpec {
                name: "Badge fill".into(),
            }),
        )?;
        tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Badge color".into(),
                color: 0xff44_aa66,
            }),
        )?;

        let (
            paywall_model,
            product_model,
            root_defaults,
            paywall_defaults,
            pro,
            paywall,
            selected_product,
            products,
            product_id,
            is_selected,
        ) = {
            let mut view_models = tx.view_models();
            let root_model = view_models.create(ViewModelSpec {
                scope: ViewModelScope::Local,
                name: "Runtime".into(),
            })?;
            let paywall_model = view_models.create(ViewModelSpec {
                scope: ViewModelScope::Local,
                name: "Paywall".into(),
            })?;
            let product_model = view_models.create(ViewModelSpec {
                scope: ViewModelScope::Local,
                name: "Product".into(),
            })?;
            let paywall = view_models.create_child(
                root_model,
                ViewModelChildSpec {
                    name: "paywall".into(),
                    view_model: paywall_model,
                },
            )?;
            let selected_product = view_models.create_string(
                paywall_model,
                ViewModelStringSpec {
                    name: "selectedProductId".into(),
                },
            )?;
            let products = view_models.create_list(
                paywall_model,
                ViewModelListSpec {
                    name: "products".into(),
                },
            )?;
            let product_id = view_models.create_string(
                product_model,
                ViewModelStringSpec {
                    name: "productId".into(),
                },
            )?;
            let is_selected = view_models.create_boolean(
                product_model,
                ViewModelBooleanSpec {
                    name: "isSelected".into(),
                },
            )?;
            let root_defaults = view_models.create_instance(
                root_model,
                ViewModelInstanceSpec {
                    name: Some("Runtime defaults".into()),
                },
            )?;
            let paywall_defaults = view_models.create_instance(
                paywall_model,
                ViewModelInstanceSpec {
                    name: Some("Paywall defaults".into()),
                },
            )?;
            let pro = view_models.create_instance(
                product_model,
                ViewModelInstanceSpec {
                    name: Some("Pro".into()),
                },
            )?;
            let basic = view_models.create_instance(
                product_model,
                ViewModelInstanceSpec {
                    name: Some("Basic".into()),
                },
            )?;
            view_models.set_child(root_defaults, paywall, paywall_defaults)?;
            view_models.set_string(paywall_defaults, selected_product, "pro")?;
            view_models.set_string(pro, product_id, "pro")?;
            view_models.set_boolean(pro, is_selected, true)?;
            view_models.set_string(basic, product_id, "basic")?;
            view_models.set_boolean(basic, is_selected, false)?;
            view_models.set_list_items(paywall_defaults, products, &[pro, basic])?;
            view_models.set_artboard_default(root_artboard, root_defaults)?;
            view_models.set_artboard_default(item_artboard, pro)?;
            view_models.bind_visibility(
                selected_badge,
                is_selected,
                VisibilityCondition::WhenTrue,
                1.0,
            )?;
            (
                paywall_model,
                product_model,
                root_defaults,
                paywall_defaults,
                pro,
                paywall,
                selected_product,
                products,
                product_id,
                is_selected,
            )
        };
        let _ = (paywall_model, root_defaults, pro);
        let product_list = tx.create_component_list(
            root_artboard,
            ArtboardComponentListSpec {
                name: "Products".into(),
                x: 5.0,
                y: 7.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                flow: None,
                source: ViewModelListSource::nested([paywall], products),
                map_rules: vec![ArtboardListMapRuleSpec {
                    view_model: product_model,
                    artboard: item_artboard,
                    state_machines: Vec::new(),
                }],
            },
        )?;
        Ok((
            root_artboard,
            paywall_defaults,
            selected_product,
            paywall,
            products,
            product_id,
            is_selected,
            selected_badge,
            product_list,
        ))
    })?;

    let instance = scene.instantiate(root_artboard)?;
    let mut events = Vec::new();
    let _ = scene.frame().advance(instance, 0.0, &mut events);
    let initial = canonical_draw_stream(&mut scene, instance)?;
    let initial_hits = scene
        .frame()
        .hit_test_paths(instance, nuxie::Vec2D::new(6.0, 8.0));
    assert_eq!(initial_hits.len(), 1);
    assert_eq!(
        initial_hits[0].objects(),
        &[product_list.object_id(), selected_badge],
        "the visible first list item resolves through its typed host and child artboard",
    );
    let source = ViewModelListSource::nested([paywall], products);
    assert!(
        matches!(
            scene.vm_list_string_match_boolean_cursor(
                instance,
                paywall_defaults,
                selected_product,
                &ViewModelListSource::direct(products),
                product_id,
                is_selected,
            ),
            Err(ResolveError::UnknownViewModelList)
        ),
        "the list must belong to the exact typed child path",
    );
    assert!(
        matches!(
            scene.vm_list_string_match_boolean_cursor(
                instance,
                paywall_defaults,
                selected_product,
                &source,
                selected_product,
                is_selected,
            ),
            Err(ResolveError::UnknownViewModelBoolean)
        ),
        "the item string and boolean must belong to the same typed model",
    );
    let cursor = scene.vm_list_string_match_boolean_cursor(
        instance,
        paywall_defaults,
        selected_product,
        &source,
        product_id,
        is_selected,
    )?;
    assert!(
        scene
            .frame()
            .set_vm_list_string_match_boolean(cursor, "basic")?
    );
    let _ = scene.frame().advance(instance, 0.0, &mut events);
    let selected = canonical_draw_stream(&mut scene, instance)?;
    let selected_hits = scene
        .frame()
        .hit_test_paths(instance, nuxie::Vec2D::new(26.0, 8.0));
    assert_eq!(selected_hits.len(), 1);
    assert_eq!(
        selected_hits[0].objects(),
        &[product_list.object_id(), selected_badge],
        "the selected branch moves semantic hits to the second retained list item",
    );
    assert_ne!(
        selected, initial,
        "the stable item contexts swap visibility"
    );
    assert!(
        !scene
            .frame()
            .set_vm_list_string_match_boolean(cursor, "basic")?,
        "an identical selection write is a hot-path no-op",
    );

    scene.edit(|tx| {
        tx.create(
            Parent::Artboard(root_artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Non-drawing structural marker".into(),
                x: 0.0,
                y: 0.0,
                opacity: 0.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        Ok(())
    })?;
    assert_eq!(
        scene
            .frame()
            .set_vm_list_string_match_boolean(cursor, "basic"),
        Err(StaleCursor),
    );
    let replay = scene.vm_list_string_match_boolean_cursor(
        instance,
        paywall_defaults,
        selected_product,
        &source,
        product_id,
        is_selected,
    )?;
    let _ = scene
        .frame()
        .set_vm_list_string_match_boolean(replay, "basic")?;
    let _ = scene.frame().advance(instance, 0.0, &mut events);
    assert_eq!(
        canonical_draw_stream(&mut scene, instance)?,
        selected,
        "re-resolved typed equality replays the selected list-item branch after remount",
    );
    Ok(())
}

#[test]
fn unset_child_view_model_values_reject_the_authoring_transaction() -> Result<()> {
    let mut scene = Scene::new();
    let epoch = scene.epoch();
    let records = scene.export_records();

    let error = scene
        .edit(|tx| {
            let mut view_models = tx.view_models();
            let root = view_models.create(ViewModelSpec {
                scope: ViewModelScope::Local,
                name: "Root".into(),
            })?;
            let paywall = view_models.create(ViewModelSpec {
                scope: ViewModelScope::Local,
                name: "Paywall".into(),
            })?;
            view_models.create_child(
                root,
                ViewModelChildSpec {
                    name: "paywall".into(),
                    view_model: paywall,
                },
            )?;
            view_models.create_instance(
                root,
                ViewModelInstanceSpec {
                    name: Some("Root defaults".into()),
                },
            )?;
            Ok(())
        })
        .expect_err("every child ViewModel property needs an explicit instance value");

    assert_eq!(error.kind(), EditErrorKind::CommitRejected);
    assert_eq!(error.diagnostic().reason, EditReason::UnknownObject);
    assert_eq!(scene.epoch(), epoch);
    assert_eq!(scene.export_records(), records);
    Ok(())
}

#[test]
fn typed_view_model_strings_export_and_import_as_runtime_instance_values() -> Result<()> {
    let mut scene = Scene::new();
    let (artboard, _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Catalog".into(),
            width: 120.0,
            height: 40.0,
        })?;
        let mut view_models = tx.view_models();
        let product = view_models.create(ViewModelSpec {
            scope: ViewModelScope::Local,
            name: "Product".into(),
        })?;
        let name = view_models.create_string(
            product,
            ViewModelStringSpec {
                name: "name".into(),
            },
        )?;
        let defaults = view_models.create_instance(
            product,
            ViewModelInstanceSpec {
                name: Some("Default product".into()),
            },
        )?;
        view_models.set_string(defaults, name, "Nuxie Pro")?;
        view_models.set_artboard_default(artboard, defaults)?;
        Ok(artboard)
    })?;

    let records = scene.export_records();
    assert!(records.records().iter().any(|record| {
        record.kind == ExportedObjectKind::ViewModelPropertyString
            && record
                .properties
                .contains(&ExportedProperty::ViewModelName("name".into()))
    }));
    assert!(records.records().iter().any(|record| {
        record.kind == ExportedObjectKind::ViewModelInstanceString
            && record.properties
                == vec![
                    ExportedProperty::ViewModelPropertyId(0),
                    ExportedProperty::ViewModelStringValue("Nuxie Pro".into()),
                ]
    }));

    let instance = scene.instantiate(artboard)?;
    let mut events = Vec::new();
    let _ = scene.frame().advance(instance, 0.0, &mut events);
    assert!(events.is_empty());
    Ok(())
}

#[test]
fn typed_view_model_extended_scalars_export_and_import_exact_values() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, defaults, tint, poster, status, submit, list_index, alternate_asset), _) =
        scene.edit(|tx| {
            let poster_asset = tx.create_image_asset(ImageAssetSpec {
                name: "Poster image".into(),
                bytes: b"view-model-only image bytes".to_vec(),
            })?;
            let alternate_asset = tx.create_image_asset(ImageAssetSpec {
                name: "Alternate poster image".into(),
                bytes: b"alternate view-model-only image bytes".to_vec(),
            })?;
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Extended scalars".into(),
                width: 120.0,
                height: 40.0,
            })?;
            let mut view_models = tx.view_models();
            let runtime = view_models.create(ViewModelSpec {
                scope: ViewModelScope::Local,
                name: "Runtime".into(),
            })?;
            let tint = view_models.create_color(
                runtime,
                ViewModelColorSpec {
                    name: "tint".into(),
                },
            )?;
            let poster = view_models.create_image(
                runtime,
                ViewModelImageSpec {
                    name: "poster".into(),
                },
            )?;
            let status = view_models.create_enum(
                runtime,
                ViewModelEnumSpec {
                    name: "status".into(),
                    values: vec!["idle".into(), "ready".into()],
                },
            )?;
            let submit = view_models.create_trigger(
                runtime,
                ViewModelTriggerSpec {
                    name: "submit".into(),
                },
            )?;
            let list_index = view_models.create_list_index(
                runtime,
                ViewModelListIndexSpec {
                    name: "list_index".into(),
                },
            )?;
            let defaults = view_models.create_instance(
                runtime,
                ViewModelInstanceSpec {
                    name: Some("Runtime defaults".into()),
                },
            )?;
            view_models.set_color(defaults, tint, 0xff12_3456)?;
            view_models.set_image(defaults, poster, poster_asset)?;
            view_models.set_enum(defaults, status, 1)?;
            view_models.set_trigger(defaults, submit, 3)?;
            view_models.set_list_index(defaults, list_index, 7)?;
            let alternate = view_models.create_instance(
                runtime,
                ViewModelInstanceSpec {
                    name: Some("Alternate runtime values".into()),
                },
            )?;
            view_models.set_image(alternate, poster, alternate_asset)?;
            view_models.set_artboard_default(artboard, defaults)?;
            Ok((
                artboard,
                defaults,
                tint,
                poster,
                status,
                submit,
                list_index,
                alternate_asset,
            ))
        })?;

    let records = scene.export_records();
    let kinds = records
        .records()
        .iter()
        .map(|record| record.kind)
        .collect::<Vec<_>>();
    for kind in [
        ExportedObjectKind::DataEnumCustom,
        ExportedObjectKind::DataEnumValue,
        ExportedObjectKind::ViewModelPropertyColor,
        ExportedObjectKind::ViewModelPropertyImage,
        ExportedObjectKind::ViewModelPropertyEnum,
        ExportedObjectKind::ViewModelPropertyTrigger,
        ExportedObjectKind::ViewModelPropertyListIndex,
        ExportedObjectKind::ViewModelInstanceColor,
        ExportedObjectKind::ViewModelInstanceImage,
        ExportedObjectKind::ViewModelInstanceEnum,
        ExportedObjectKind::ViewModelInstanceTrigger,
        ExportedObjectKind::ViewModelInstanceListIndex,
    ] {
        assert!(kinds.contains(&kind), "missing exact {kind:?} record");
    }
    assert!(records.records().iter().any(|record| {
        record.kind == ExportedObjectKind::ViewModelInstanceColor
            && record
                .properties
                .contains(&ExportedProperty::ViewModelColorValue(0xff12_3456))
    }));
    assert!(records.records().iter().any(|record| {
        record.kind == ExportedObjectKind::ViewModelInstanceImage
            && record
                .properties
                .contains(&ExportedProperty::ViewModelImageValue(0))
    }));
    assert!(records.records().iter().any(|record| {
        record.kind == ExportedObjectKind::ViewModelInstanceEnum
            && record
                .properties
                .contains(&ExportedProperty::ViewModelEnumValue(1))
    }));
    assert!(records.records().iter().any(|record| {
        record.kind == ExportedObjectKind::ViewModelInstanceTrigger
            && record
                .properties
                .contains(&ExportedProperty::ViewModelTriggerValue(3))
    }));
    assert!(records.records().iter().any(|record| {
        record.kind == ExportedObjectKind::ViewModelInstanceListIndex
            && record
                .properties
                .contains(&ExportedProperty::ViewModelListIndexValue(7))
    }));

    // Materialization imports the exact authored stream into nuxie-runtime.
    let records_before_hot_writes = scene.export_records();
    let instance = scene.instantiate(artboard)?;
    assert!(scene.set_vm_color(instance, defaults, tint, 0xff65_4321)?);
    assert!(!scene.set_vm_color(instance, defaults, tint, 0xff65_4321)?);
    assert!(scene.set_vm_image(instance, defaults, poster, alternate_asset)?);
    assert!(!scene.set_vm_image(instance, defaults, poster, alternate_asset)?);
    assert!(scene.set_vm_enum(instance, defaults, status, 0)?);
    assert!(!scene.set_vm_enum(instance, defaults, status, 0)?);
    assert!(scene.set_vm_trigger(instance, defaults, submit, 4)?);
    assert!(!scene.set_vm_trigger(instance, defaults, submit, 4)?);
    assert!(scene.set_vm_list_index(instance, defaults, list_index, 8)?);
    assert!(!scene.set_vm_list_index(instance, defaults, list_index, 8)?);
    assert_eq!(scene.export_records(), records_before_hot_writes);
    let mut events = Vec::new();
    let _ = scene.frame().advance(instance, 0.0, &mut events);
    Ok(())
}

#[test]
fn typed_view_model_enum_validation_rejects_lossy_authoring() -> Result<()> {
    let mut scene = Scene::new();
    let error = scene
        .edit(|tx| {
            tx.create_artboard(ArtboardSpec {
                name: "Validation".into(),
                width: 120.0,
                height: 40.0,
            })?;
            let mut view_models = tx.view_models();
            let runtime = view_models.create(ViewModelSpec {
                scope: ViewModelScope::Local,
                name: "Runtime".into(),
            })?;
            let status = view_models.create_enum(
                runtime,
                ViewModelEnumSpec {
                    name: "status".into(),
                    values: vec!["idle".into(), "ready".into()],
                },
            )?;
            let defaults = view_models.create_instance(
                runtime,
                ViewModelInstanceSpec {
                    name: Some("Defaults".into()),
                },
            )?;
            view_models.set_enum(defaults, status, 2)?;
            Ok(())
        })
        .expect_err("out-of-range enum ordinals must never be truncated or defaulted");
    assert_eq!(error.kind(), EditErrorKind::Aborted);
    assert_eq!(
        error.diagnostic().reason,
        EditReason::OutOfRangeProperty {
            property: "view_model_enum",
        }
    );

    let error = scene
        .edit(|tx| {
            tx.create_artboard(ArtboardSpec {
                name: "Duplicate keys".into(),
                width: 120.0,
                height: 40.0,
            })?;
            let mut view_models = tx.view_models();
            let runtime = view_models.create(ViewModelSpec {
                scope: ViewModelScope::Local,
                name: "Runtime".into(),
            })?;
            view_models.create_enum(
                runtime,
                ViewModelEnumSpec {
                    name: "status".into(),
                    values: vec!["idle".into(), "idle".into()],
                },
            )?;
            Ok(())
        })
        .expect_err("duplicate enum keys must never collapse in the DataEnum record stream");
    assert_eq!(error.kind(), EditErrorKind::Aborted);
    assert_eq!(error.diagnostic().reason, EditReason::IdentityCollision);
    Ok(())
}

#[derive(Clone, Copy)]
struct ViewModelStringFixture {
    artboard: ArtboardId,
    shape: ObjectId,
    defaults: ViewModelInstanceId,
    label: ViewModelStringId,
}

fn create_view_model_string_fixture(
    tx: &mut SceneTx<'_>,
) -> std::result::Result<ViewModelStringFixture, EditAbort> {
    let artboard = tx.create_artboard(ArtboardSpec {
        name: "String cursor".into(),
        width: 100.0,
        height: 100.0,
    })?;
    let shape = tx.create(
        Parent::Artboard(artboard),
        NodeSpec::Shape(ShapeSpec {
            name: "Remount target".into(),
            x: 0.0,
            y: 0.0,
            opacity: 1.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        }),
    )?;
    let mut view_models = tx.view_models();
    let model = view_models.create(ViewModelSpec {
        scope: ViewModelScope::Local,
        name: "Copy".into(),
    })?;
    let label = view_models.create_string(
        model,
        ViewModelStringSpec {
            name: "label".into(),
        },
    )?;
    let defaults = view_models.create_instance(
        model,
        ViewModelInstanceSpec {
            name: Some("Defaults".into()),
        },
    )?;
    view_models.set_string(defaults, label, "before")?;
    view_models.set_artboard_default(artboard, defaults)?;
    Ok(ViewModelStringFixture {
        artboard,
        shape,
        defaults,
        label,
    })
}

#[test]
fn typed_view_model_string_cursors_hot_write_and_carry_across_same_schema_remounts() -> Result<()> {
    let mut scene = Scene::new();
    let (fixture, _) = scene.edit(create_view_model_string_fixture)?;
    let instance = scene.instantiate(fixture.artboard)?;
    let cursor = scene.vm_string_cursor(instance, fixture.defaults, fixture.label)?;
    let epoch = scene.epoch();
    let records = scene.export_records().into_records();

    assert_eq!(scene.frame().get_vm_string(cursor)?, "before");
    assert!(scene.frame().set_vm_string(cursor, "after")?);
    assert!(!scene.frame().set_vm_string(cursor, "after")?);
    assert_eq!(scene.frame().get_vm_string(cursor)?, "after");
    assert_eq!(scene.epoch(), epoch);
    assert_eq!(scene.export_records().into_records(), records);

    scene.edit(|tx| tx.set(fixture.shape, props::TRANSLATE_X, 12.0))?;
    assert_eq!(scene.frame().get_vm_string(cursor), Err(StaleCursor));
    let carried = scene.vm_string_cursor(instance, fixture.defaults, fixture.label)?;
    assert_eq!(scene.frame().get_vm_string(carried)?, "after");

    scene.drop_instance(instance);
    assert_eq!(scene.frame().get_vm_string(carried), Err(StaleCursor));
    assert_eq!(
        scene.frame().set_vm_string(carried, "stale"),
        Err(StaleCursor)
    );
    Ok(())
}

#[test]
fn unnamed_view_model_instance_omits_the_optional_component_name_record() -> Result<()> {
    let mut scene = Scene::new();
    scene.edit(|tx| {
        let mut view_models = tx.view_models();
        let model = view_models.create(ViewModelSpec {
            scope: ViewModelScope::Local,
            name: "Playback".into(),
        })?;
        let duration = view_models.create_number(
            model,
            ViewModelNumberSpec {
                name: "Duration".into(),
            },
        )?;
        let unnamed = view_models.create_instance(model, ViewModelInstanceSpec { name: None })?;
        view_models.set_number(unnamed, duration, 0.0)?;
        Ok(())
    })?;

    assert_eq!(
        scene.export_records().into_records(),
        vec![
            ExportedRecord {
                kind: ExportedObjectKind::Backboard,
                properties: vec![],
            },
            ExportedRecord {
                kind: ExportedObjectKind::ViewModel,
                properties: vec![ExportedProperty::ViewModelName("Playback".into())],
            },
            ExportedRecord {
                kind: ExportedObjectKind::ViewModelPropertyNumber,
                properties: vec![ExportedProperty::ViewModelName("Duration".into())],
            },
            ExportedRecord {
                kind: ExportedObjectKind::ViewModelInstance,
                properties: vec![ExportedProperty::ViewModelId(0)],
            },
            ExportedRecord {
                kind: ExportedObjectKind::ViewModelInstanceNumber,
                properties: vec![
                    ExportedProperty::ViewModelPropertyId(0),
                    ExportedProperty::ViewModelNumberValue(0.0),
                ],
            },
        ]
    );
    Ok(())
}

#[derive(Clone, Copy)]
struct ViewModelDurationFixture {
    artboard: ArtboardId,
    shape: ObjectId,
    machine: MachineId,
    transition: MachineTransitionId,
    model: ViewModelId,
    defaults: ViewModelInstanceId,
    duration: ViewModelNumberId,
    bind: DataBindId,
}

fn create_view_model_duration_machine(
    tx: &mut SceneTx<'_>,
) -> std::result::Result<ViewModelDurationFixture, EditAbort> {
    let artboard = tx.create_artboard(ArtboardSpec {
        name: "Canvas".into(),
        width: 100.0,
        height: 100.0,
    })?;
    let shape = tx.create(
        Parent::Artboard(artboard),
        NodeSpec::Shape(ShapeSpec {
            name: "Fader".into(),
            x: 0.0,
            y: 0.0,
            opacity: 1.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        }),
    )?;
    let idle = tx.animations().create_linear(
        artboard,
        LinearAnimationSpec {
            name: "Idle".into(),
            fps: 60,
            duration: 1,
        },
    )?;
    tx.animations()
        .set_key(idle, shape, props::WORLD_OPACITY, 0, 0.2)?;
    let active = tx.animations().create_linear(
        artboard,
        LinearAnimationSpec {
            name: "Active".into(),
            fps: 60,
            duration: 1,
        },
    )?;
    tx.animations()
        .set_key(active, shape, props::WORLD_OPACITY, 0, 0.8)?;

    let (machine, transition, go) = {
        let mut machines = tx.machines();
        let machine = machines.create_machine(
            artboard,
            MachineSpec {
                name: Some("Switcher".into()),
            },
        )?;
        let go = machines.create_trigger_input(machine, TriggerInputSpec { name: "Go".into() })?;
        let layer = machines.create_layer(machine, MachineLayerSpec { name: None })?;
        let entry = machines.create_entry_state(layer)?;
        let any = machines.create_any_state(layer)?;
        machines.create_exit_state(layer)?;
        let idle_state =
            machines.create_animation_state(layer, AnimationStateSpec { animation: idle })?;
        let active_state =
            machines.create_animation_state(layer, AnimationStateSpec { animation: active })?;
        machines.create_transition(entry, idle_state)?;
        let transition = machines.create_transition(any, active_state)?;
        (machine, transition, go)
    };

    let (model, duration, defaults, bind) = {
        let mut view_models = tx.view_models();
        let model = view_models.create(ViewModelSpec {
            scope: ViewModelScope::Local,
            name: "Playback".into(),
        })?;
        let duration = view_models.create_number(
            model,
            ViewModelNumberSpec {
                name: "Duration".into(),
            },
        )?;
        let defaults = view_models.create_instance(
            model,
            ViewModelInstanceSpec {
                name: Some("Defaults".into()),
            },
        )?;
        view_models.set_number(defaults, duration, 0.0)?;
        view_models.set_artboard_default(artboard, defaults)?;
        let bind = view_models.bind_transition_duration(transition, duration)?;
        (model, duration, defaults, bind)
    };
    tx.machines().add_trigger_condition(transition, go)?;
    Ok(ViewModelDurationFixture {
        artboard,
        shape,
        machine,
        transition,
        model,
        defaults,
        duration,
        bind,
    })
}

#[test]
fn live_view_model_numbers_drive_transition_duration_without_recompiling() -> Result<()> {
    let mut scene = Scene::new();
    let (fixture, _) = scene.edit(create_view_model_duration_machine)?;
    let first = scene.instantiate(fixture.artboard)?;
    let second = scene.instantiate(fixture.artboard)?;
    let first_duration = scene.vm_cursor(first, fixture.defaults, fixture.duration)?;
    let second_duration = scene.vm_cursor(second, fixture.defaults, fixture.duration)?;
    let first_opacity = scene.cursor(first, fixture.shape, props::WORLD_OPACITY)?;
    let second_opacity = scene.cursor(second, fixture.shape, props::WORLD_OPACITY)?;
    let first_go = scene.machine_input(first, fixture.machine, "Go")?;
    let second_go = scene.machine_input(second, fixture.machine, "Go")?;
    let epoch = scene.epoch();
    let records = scene.export_records().into_records();
    let mut events = Vec::new();

    assert_eq!(scene.frame().get_vm(first_duration)?, 0.0);
    assert_eq!(scene.frame().get_vm(second_duration)?, 0.0);
    let _ = scene.frame().advance(first, 0.0, &mut events);
    let _ = scene.frame().advance(second, 0.0, &mut events);
    assert_eq!(scene.frame().get(first_opacity)?, 0.2);
    assert_eq!(scene.frame().get(second_opacity)?, 0.2);

    assert!(scene.frame().set_vm(first_duration, 1_000.0)?);
    assert!(!scene.frame().set_vm(first_duration, 1_000.0)?);
    assert_eq!(scene.frame().get_vm(first_duration)?, 1_000.0);
    assert_eq!(scene.frame().get_vm(second_duration)?, 0.0);
    assert_eq!(scene.epoch(), epoch);
    assert_eq!(scene.export_records().into_records(), records);

    scene.frame().fire(first_go)?;
    let _ = scene.frame().advance(first, 0.0, &mut events);
    assert_eq!(scene.frame().get(first_opacity)?, 0.2);
    let _ = scene.frame().advance(first, 0.5, &mut events);
    assert!((scene.frame().get(first_opacity)? - 0.5).abs() <= 0.001);
    let _ = scene.frame().advance(first, 0.5, &mut events);
    assert!((scene.frame().get(first_opacity)? - 0.8).abs() <= 0.001);

    scene.frame().fire(second_go)?;
    let _ = scene.frame().advance(second, 0.0, &mut events);
    assert_eq!(scene.frame().get(second_opacity)?, 0.8);
    assert_eq!(scene.frame().get_vm(second_duration)?, 0.0);
    Ok(())
}

#[test]
fn view_model_cursors_are_fenced_and_same_schema_remounts_carry_live_numbers() -> Result<()> {
    let mut scene = Scene::new();
    let (fixture, _) = scene.edit(create_view_model_duration_machine)?;
    let instance = scene.instantiate(fixture.artboard)?;
    let cursor = scene.vm_cursor(instance, fixture.defaults, fixture.duration)?;
    let epoch = scene.epoch();
    let records = scene.export_records().into_records();

    assert!(scene.frame().set_vm(cursor, 625.0)?);
    assert_eq!(scene.frame().set_vm(cursor, f32::NAN), Ok(false));
    assert_eq!(scene.frame().set_vm(cursor, f32::INFINITY), Ok(false));
    assert_eq!(scene.frame().get_vm(cursor)?, 625.0);
    assert_eq!(scene.epoch(), epoch);
    assert_eq!(scene.export_records().into_records(), records);

    let mut foreign = Scene::new();
    let (foreign_fixture, _) = foreign.edit(create_view_model_duration_machine)?;
    let foreign_instance = foreign.instantiate(foreign_fixture.artboard)?;
    assert_eq!(foreign.frame().set_vm(cursor, 100.0), Err(StaleCursor));
    assert!(matches!(
        scene.vm_cursor(instance, foreign_fixture.defaults, fixture.duration),
        Err(ResolveError::UnknownViewModelInstance)
    ));
    assert!(matches!(
        scene.vm_cursor(instance, fixture.defaults, foreign_fixture.duration),
        Err(ResolveError::UnknownViewModelNumber)
    ));
    assert_eq!(
        foreign
            .vm_cursor(
                foreign_instance,
                foreign_fixture.defaults,
                foreign_fixture.duration,
            )
            .map(|_| ()),
        Ok(())
    );

    scene.edit(|tx| tx.set(fixture.shape, props::TRANSLATE_X, 12.0))?;
    assert_eq!(scene.frame().get_vm(cursor), Err(StaleCursor));
    let carried = scene.vm_cursor(instance, fixture.defaults, fixture.duration)?;
    assert_eq!(scene.frame().get_vm(carried)?, 625.0);

    scene.edit(|tx| {
        tx.view_models()
            .set_number_name(fixture.duration, "Renamed duration".into())?;
        Ok(())
    })?;
    assert_eq!(scene.frame().get_vm(carried), Err(StaleCursor));
    let renamed = scene.vm_cursor(instance, fixture.defaults, fixture.duration)?;
    assert_eq!(
        scene.frame().get_vm(renamed)?,
        0.0,
        "a schema rename must start from the authored default"
    );

    scene.drop_instance(instance);
    assert_eq!(scene.frame().get_vm(renamed), Err(StaleCursor));
    assert_eq!(scene.frame().set_vm(renamed, 10.0), Err(StaleCursor));
    Ok(())
}

#[test]
fn unoverridden_view_model_number_uses_a_new_authored_default_after_remount() -> Result<()> {
    let mut scene = Scene::new();
    let (fixture, _) = scene.edit(create_view_model_duration_machine)?;
    let instance = scene.instantiate(fixture.artboard)?;
    let old = scene.vm_cursor(instance, fixture.defaults, fixture.duration)?;
    assert_eq!(scene.frame().get_vm(old)?, 0.0);

    scene.edit(|tx| {
        tx.view_models()
            .set_number(fixture.defaults, fixture.duration, 250.0)?;
        Ok(())
    })?;

    assert_eq!(scene.frame().get_vm(old), Err(StaleCursor));
    let fresh = scene.vm_cursor(instance, fixture.defaults, fixture.duration)?;
    assert_eq!(
        scene.frame().get_vm(fresh)?,
        250.0,
        "an untouched live slot must adopt the edited authored default"
    );
    Ok(())
}

#[test]
fn explicit_noop_view_model_write_remains_an_override_after_remount() -> Result<()> {
    let mut scene = Scene::new();
    let (fixture, _) = scene.edit(create_view_model_duration_machine)?;
    let instance = scene.instantiate(fixture.artboard)?;
    let old = scene.vm_cursor(instance, fixture.defaults, fixture.duration)?;
    assert_eq!(scene.frame().get_vm(old)?, 0.0);
    assert_eq!(scene.frame().set_vm(old, 0.0), Ok(false));

    scene.edit(|tx| {
        tx.view_models()
            .set_number(fixture.defaults, fixture.duration, 250.0)?;
        Ok(())
    })?;

    assert_eq!(scene.frame().get_vm(old), Err(StaleCursor));
    let fresh = scene.vm_cursor(instance, fixture.defaults, fixture.duration)?;
    assert_eq!(
        scene.frame().get_vm(fresh)?,
        0.0,
        "a finite explicit write is an override even when the value was unchanged"
    );
    Ok(())
}

#[test]
fn changing_the_authored_default_instance_does_not_cross_carry_live_values() -> Result<()> {
    let mut scene = Scene::new();
    let (fixture, _) = scene.edit(create_view_model_duration_machine)?;
    let instance = scene.instantiate(fixture.artboard)?;
    let old = scene.vm_cursor(instance, fixture.defaults, fixture.duration)?;
    assert!(scene.frame().set_vm(old, 900.0)?);

    let (replacement, _) = scene.edit(|tx| {
        let mut view_models = tx.view_models();
        let replacement = view_models.create_instance(
            fixture.model,
            ViewModelInstanceSpec {
                name: Some("Alternate defaults".into()),
            },
        )?;
        view_models.set_number(replacement, fixture.duration, 250.0)?;
        view_models.set_artboard_default(fixture.artboard, replacement)?;
        Ok(replacement)
    })?;

    assert_eq!(scene.frame().get_vm(old), Err(StaleCursor));
    assert!(matches!(
        scene.vm_cursor(instance, fixture.defaults, fixture.duration),
        Err(ResolveError::UnknownViewModelInstance)
    ));
    let replacement_cursor = scene.vm_cursor(instance, replacement, fixture.duration)?;
    assert_eq!(scene.frame().get_vm(replacement_cursor)?, 250.0);
    Ok(())
}

#[test]
fn view_model_catalog_replacement_is_atomic_and_burns_runtime_identities() -> Result<()> {
    let mut scene = Scene::new();
    let (fixture, _) = scene.edit(create_view_model_duration_machine)?;
    let instance = scene.instantiate(fixture.artboard)?;
    let old = scene.vm_cursor(instance, fixture.defaults, fixture.duration)?;
    assert!(scene.frame().set_vm(old, 875.0)?);
    let epoch = scene.epoch();
    let records = scene.export_records().into_records();

    let rejected = scene.edit(|tx| {
        tx.view_models().clear_catalog()?;
        Ok(())
    });
    assert!(
        rejected.is_err(),
        "dangling defaults and binds reject commit"
    );
    assert_eq!(scene.epoch(), epoch);
    assert_eq!(scene.export_records().into_records(), records);
    assert_eq!(scene.frame().get_vm(old)?, 875.0);

    let ((new_defaults, new_duration), _) = scene.edit(|tx| {
        let mut view_models = tx.view_models();
        view_models.remove_transition_duration_bind(fixture.bind)?;
        view_models.clear_artboard_default(fixture.artboard)?;
        view_models.clear_catalog()?;
        let model = view_models.create(ViewModelSpec {
            scope: ViewModelScope::Local,
            name: "Replacement playback".into(),
        })?;
        let duration = view_models.create_number(
            model,
            ViewModelNumberSpec {
                name: "Duration".into(),
            },
        )?;
        let defaults = view_models.create_instance(
            model,
            ViewModelInstanceSpec {
                name: Some("Replacement defaults".into()),
            },
        )?;
        view_models.set_number(defaults, duration, 125.0)?;
        view_models.set_artboard_default(fixture.artboard, defaults)?;
        view_models.bind_transition_duration(fixture.transition, duration)?;
        Ok((defaults, duration))
    })?;

    assert_eq!(scene.frame().get_vm(old), Err(StaleCursor));
    assert!(matches!(
        scene.vm_cursor(instance, fixture.defaults, fixture.duration),
        Err(ResolveError::UnknownViewModelInstance)
    ));
    let replacement = scene.vm_cursor(instance, new_defaults, new_duration)?;
    assert_eq!(
        scene.frame().get_vm(replacement)?,
        125.0,
        "clear/recreate has new runtime identities and starts from authored defaults"
    );
    Ok(())
}

fn append_authored_trigger_machine(
    tx: &mut SceneTx<'_>,
    artboard: ArtboardId,
    idle: AnimationId,
    active: AnimationId,
    machine_name: &str,
    trigger_name: &str,
    event_name: &str,
) -> std::result::Result<(MachineId, MachineInputId, EventId), EditAbort> {
    let mut machines = tx.machines();
    let event = machines.create_event(
        artboard,
        EventSpec {
            name: Some(event_name.into()),
        },
    )?;
    let machine = machines.create_machine(
        artboard,
        MachineSpec {
            name: Some(machine_name.into()),
        },
    )?;
    let trigger = machines.create_trigger_input(
        machine,
        TriggerInputSpec {
            name: trigger_name.into(),
        },
    )?;
    let layer = machines.create_layer(
        machine,
        MachineLayerSpec {
            name: Some("Main".into()),
        },
    )?;
    let entry = machines.create_entry_state(layer)?;
    let any = machines.create_any_state(layer)?;
    machines.create_exit_state(layer)?;
    let idle_state =
        machines.create_animation_state(layer, AnimationStateSpec { animation: idle })?;
    let active_state =
        machines.create_animation_state(layer, AnimationStateSpec { animation: active })?;
    machines.create_transition(entry, idle_state)?;
    let transition = machines.create_transition(any, active_state)?;
    machines.add_trigger_condition(transition, trigger)?;
    machines.add_fire_event(active_state, event, FireEventOccurs::AtStart)?;
    Ok((machine, trigger, event))
}

fn create_authored_trigger_machine(
    tx: &mut SceneTx<'_>,
) -> std::result::Result<(ArtboardId, ObjectId, MachineId, MachineInputId, EventId), EditAbort> {
    let artboard = tx.create_artboard(ArtboardSpec {
        name: "Canvas".into(),
        width: 100.0,
        height: 100.0,
    })?;
    let shape = tx.create(
        Parent::Artboard(artboard),
        NodeSpec::Shape(ShapeSpec {
            name: "Fader".into(),
            x: 0.0,
            y: 0.0,
            opacity: 1.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        }),
    )?;
    let idle = tx.animations().create_linear(
        artboard,
        LinearAnimationSpec {
            name: "Idle".into(),
            fps: 60,
            duration: 1,
        },
    )?;
    tx.animations()
        .set_key(idle, shape, props::WORLD_OPACITY, 0, 0.2)?;
    let active = tx.animations().create_linear(
        artboard,
        LinearAnimationSpec {
            name: "Active".into(),
            fps: 60,
            duration: 1,
        },
    )?;
    tx.animations()
        .set_key(active, shape, props::WORLD_OPACITY, 0, 0.8)?;
    let (machine, trigger, event) = append_authored_trigger_machine(
        tx,
        artboard,
        idle,
        active,
        "Switcher",
        "Go",
        "Reached active",
    )?;
    Ok((artboard, shape, machine, trigger, event))
}

#[test]
fn trigger_machine_changes_visual_state_and_reports_one_semantic_event() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, shape, machine, _, event), _) = scene.edit(create_authored_trigger_machine)?;
    let instance = scene.instantiate(artboard)?;
    let opacity = scene.cursor(instance, shape, props::WORLD_OPACITY)?;
    let go = scene.machine_input(instance, machine, "Go")?;
    let mut events = Vec::new();

    assert!(scene.frame().advance(instance, 0.0, &mut events));
    assert_eq!(scene.frame().get(opacity)?, 0.2);
    assert!(events.is_empty());

    scene.frame().fire(go)?;
    assert!(scene.frame().advance(instance, 0.0, &mut events));
    assert_eq!(scene.frame().get(opacity)?, 0.8);
    assert_eq!(
        events,
        vec![SceneEvent::Authored {
            event,
            name: Some("Reached active".into()),
            string_properties: Vec::new(),
            seconds_delay: 0.0,
            context: None,
        }]
    );

    scene.frame().advance(instance, 0.0, &mut events);
    assert!(events.is_empty(), "events are scoped to one advance call");
    assert_eq!(scene.frame().get(opacity)?, 0.8);

    let second_instance = scene.instantiate(artboard)?;
    let second_opacity = scene.cursor(second_instance, shape, props::WORLD_OPACITY)?;
    assert!(scene.frame().advance(second_instance, 0.0, &mut events));
    assert_eq!(scene.frame().get(second_opacity)?, 0.2);
    assert!(
        events.is_empty(),
        "machine state is isolated per live instance"
    );
    Ok(())
}

#[test]
fn event_string_properties_survive_exact_export_and_runtime_reporting() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, machine, event), _) = scene.edit(|tx| {
        let (artboard, _, machine, _, event) = create_authored_trigger_machine(tx)?;
        let mut machines = tx.machines();
        machines.create_event_string_property(
            event,
            EventStringPropertySpec {
                name: "nuxieTrigger".into(),
                value: "press".into(),
            },
        )?;
        machines.create_event_string_property(
            event,
            EventStringPropertySpec {
                name: "componentId".into(),
                value: "cta".into(),
            },
        )?;
        Ok((artboard, machine, event))
    })?;

    let records = scene.export_records().into_records();
    let properties = records
        .iter()
        .filter(|record| record.kind == ExportedObjectKind::CustomPropertyString)
        .collect::<Vec<_>>();
    assert_eq!(properties.len(), 2);
    assert_eq!(
        properties[0].properties,
        vec![
            ExportedProperty::ComponentName("nuxieTrigger".into()),
            ExportedProperty::ParentId(2),
            ExportedProperty::CustomPropertyStringValue("press".into()),
        ]
    );
    assert_eq!(
        properties[1].properties,
        vec![
            ExportedProperty::ComponentName("componentId".into()),
            ExportedProperty::ParentId(2),
            ExportedProperty::CustomPropertyStringValue("cta".into()),
        ]
    );

    let instance = scene.instantiate(artboard)?;
    let go = scene.machine_input(instance, machine, "Go")?;
    let mut events = Vec::new();
    scene.frame().advance(instance, 0.0, &mut events);
    scene.frame().fire(go)?;
    scene.frame().advance(instance, 0.0, &mut events);
    assert_eq!(
        events,
        vec![SceneEvent::Authored {
            event,
            name: Some("Reached active".into()),
            string_properties: vec![
                SceneEventStringProperty {
                    name: "nuxieTrigger".into(),
                    value: "press".into(),
                },
                SceneEventStringProperty {
                    name: "componentId".into(),
                    value: "cta".into(),
                },
            ],
            seconds_delay: 0.0,
            context: None,
        }]
    );
    Ok(())
}

#[test]
fn authored_click_listener_hits_the_target_and_drives_its_machine_after_exact_export() -> Result<()>
{
    let mut scene = Scene::new();
    let ((artboard, shape, machine, event), _) = scene.edit(|tx| {
        let (artboard, shape, machine, trigger, event) = create_authored_trigger_machine(tx)?;
        tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new("Hit target", 80.0, 80.0)),
        )?;
        let mut machines = tx.machines();
        let listener = machines.create_listener(
            machine,
            shape,
            MachineListenerSpec {
                name: Some("Activate card".into()),
                listener_type: MachineListenerType::Click,
            },
        )?;
        machines.add_listener_trigger_action(listener, trigger)?;
        Ok((artboard, shape, machine, event))
    })?;

    let records = scene.export_records().into_records();
    assert!(records.iter().any(|record| {
        record.kind == ExportedObjectKind::StateMachineListener
            && record
                .properties
                .contains(&ExportedProperty::ListenerTargetId(1))
            && record
                .properties
                .contains(&ExportedProperty::ListenerType(MachineListenerType::Click))
    }));
    assert!(records.iter().any(|record| {
        record.kind == ExportedObjectKind::ListenerTriggerChange
            && record
                .properties
                .contains(&ExportedProperty::ListenerInputId(0))
    }));

    let instance = scene.instantiate(artboard)?;
    let opacity = scene.cursor(instance, shape, props::WORLD_OPACITY)?;
    let mut events = Vec::new();
    assert!(scene.frame().advance(instance, 0.0, &mut events));
    assert_eq!(scene.frame().get(opacity)?, 0.2);

    assert!(
        scene
            .frame()
            .pointer_down(instance, Vec2D::new(0.0, 0.0), 7)
    );
    assert!(scene.frame().pointer_up(instance, Vec2D::new(0.0, 0.0), 7));
    assert!(scene.frame().advance(instance, 0.0, &mut events));
    assert_eq!(scene.frame().get(opacity)?, 0.8);
    assert_eq!(
        events,
        vec![SceneEvent::Authored {
            event,
            name: Some("Reached active".into()),
            string_properties: Vec::new(),
            seconds_delay: 0.0,
            context: None,
        }]
    );
    assert!(scene.machine_input(instance, machine, "Go").is_ok());

    scene.frame().advance(instance, 0.0, &mut events);
    assert!(
        events.is_empty(),
        "click-triggered events report for one frame"
    );
    Ok(())
}

#[test]
fn authored_event_and_view_model_listeners_export_typed_sources_and_view_model_actions_execute()
-> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, shape, defaults, progress, label), _) = scene.edit(|tx| {
        let (artboard, shape, machine, trigger, event) = create_authored_trigger_machine(tx)?;
        let alternate_image = tx.create_image_asset(ImageAssetSpec {
            name: "Alternate image".into(),
            bytes: b"listener image bytes".to_vec(),
        })?;
        tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new("Hit target", 80.0, 80.0)),
        )?;

        let (defaults, progress, label, enabled, tint, poster, status, submit, selected) = {
            let mut view_models = tx.view_models();
            let model = view_models.create(ViewModelSpec {
                scope: ViewModelScope::Local,
                name: "Interaction".into(),
            })?;
            let progress = view_models.create_number(
                model,
                ViewModelNumberSpec {
                    name: "progress".into(),
                },
            )?;
            let label = view_models.create_string(
                model,
                ViewModelStringSpec {
                    name: "label".into(),
                },
            )?;
            let enabled = view_models.create_boolean(
                model,
                ViewModelBooleanSpec {
                    name: "enabled".into(),
                },
            )?;
            let tint = view_models.create_color(
                model,
                ViewModelColorSpec {
                    name: "tint".into(),
                },
            )?;
            let poster = view_models.create_image(
                model,
                ViewModelImageSpec {
                    name: "poster".into(),
                },
            )?;
            let status = view_models.create_enum(
                model,
                ViewModelEnumSpec {
                    name: "status".into(),
                    values: vec!["idle".into(), "ready".into()],
                },
            )?;
            let submit = view_models.create_trigger(
                model,
                ViewModelTriggerSpec {
                    name: "submit".into(),
                },
            )?;
            let selected = view_models.create_list_index(
                model,
                ViewModelListIndexSpec {
                    name: "selected".into(),
                },
            )?;
            let defaults = view_models.create_instance(
                model,
                ViewModelInstanceSpec {
                    name: Some("Defaults".into()),
                },
            )?;
            view_models.set_number(defaults, progress, 0.0)?;
            view_models.set_string(defaults, label, "idle")?;
            view_models.set_boolean(defaults, enabled, false)?;
            view_models.set_color(defaults, tint, 0)?;
            view_models.set_enum(defaults, status, 0)?;
            view_models.set_trigger(defaults, submit, 0)?;
            view_models.set_list_index(defaults, selected, 0)?;
            view_models.set_artboard_default(artboard, defaults)?;
            (
                defaults, progress, label, enabled, tint, poster, status, submit, selected,
            )
        };

        let progress_source = nuxie::ViewModelNumberSource::direct(progress);
        let mut machines = tx.machines();
        let event_listener = machines.create_event_listener(
            machine,
            shape,
            event,
            Some("When active is reported".into()),
        )?;
        machines.add_listener_view_model_string_action(
            event_listener,
            nuxie::ViewModelStringSource::direct(label),
            "reported",
        )?;
        let view_model_listener = machines.create_view_model_listener(
            machine,
            shape,
            nuxie::MachineViewModelSource::Number(progress_source.clone()),
            Some("When progress changes".into()),
        )?;
        machines.add_listener_trigger_action(view_model_listener, trigger)?;
        let click = machines.create_listener(
            machine,
            shape,
            MachineListenerSpec {
                name: Some("Set progress".into()),
                listener_type: MachineListenerType::Click,
            },
        )?;
        machines.add_listener_view_model_number_action(click, progress_source, 0.75)?;
        machines.add_listener_view_model_boolean_action(
            click,
            nuxie::ViewModelBooleanSource::direct(enabled),
            true,
        )?;
        machines.add_listener_view_model_color_action(
            click,
            nuxie::ViewModelColorSource::direct(tint),
            0xff12_3456,
        )?;
        machines.add_listener_view_model_image_action(
            click,
            nuxie::ViewModelImageSource::direct(poster),
            alternate_image,
        )?;
        machines.add_listener_view_model_enum_action(
            click,
            nuxie::ViewModelEnumSource::direct(status),
            1,
        )?;
        machines.add_listener_view_model_trigger_action(
            click,
            nuxie::ViewModelTriggerSource::direct(submit),
        )?;
        machines.add_listener_view_model_list_index_action(
            click,
            nuxie::ViewModelListIndexSource::direct(selected),
            4,
        )?;
        Ok((artboard, shape, defaults, progress, label))
    })?;

    let records = scene.export_records().into_records();
    assert!(records.iter().any(|record| {
        record.kind == ExportedObjectKind::StateMachineListener
            && record
                .properties
                .contains(&ExportedProperty::ListenerType(MachineListenerType::Event))
            && record
                .properties
                .iter()
                .any(|property| matches!(property, ExportedProperty::ListenerEventId(_)))
    }));
    assert!(records.iter().any(|record| {
        record.kind == ExportedObjectKind::StateMachineListener
            && record.properties.contains(&ExportedProperty::ListenerType(
                MachineListenerType::ViewModel,
            ))
            && record
                .properties
                .contains(&ExportedProperty::ListenerViewModelPath(vec![0, 0]))
    }));
    let assert_bindable_triplet = |kind: ExportedObjectKind,
                                   value: ExportedProperty,
                                   property_key: u32| {
        let bindable_index = records
            .iter()
            .position(|record| record.kind == kind && record.properties.contains(&value))
            .expect("typed listener ViewModel write bindable");
        assert_eq!(
            records[bindable_index + 1].kind,
            ExportedObjectKind::DataBindContext
        );
        assert!(
            records[bindable_index + 1]
                .properties
                .contains(&ExportedProperty::DataBindPropertyKey(property_key))
        );
        assert!(
            records[bindable_index + 1]
                .properties
                .contains(&ExportedProperty::DataBindFlags(1))
        );
        assert!(records[bindable_index + 1]
                .properties
                .iter()
                .any(|property| matches!(property, ExportedProperty::DataBindSourcePath(path) if path.len() == 2)));
        assert_eq!(
            records[bindable_index + 2].kind,
            ExportedObjectKind::ListenerViewModelChange
        );
    };
    for (kind, value, property_key) in [
        (
            ExportedObjectKind::BindablePropertyNumber,
            ExportedProperty::BindablePropertyNumberValue(0.75),
            636,
        ),
        (
            ExportedObjectKind::BindablePropertyString,
            ExportedProperty::BindablePropertyStringValue("reported".into()),
            635,
        ),
        (
            ExportedObjectKind::BindablePropertyBoolean,
            ExportedProperty::BindablePropertyBooleanValue(true),
            634,
        ),
        (
            ExportedObjectKind::BindablePropertyColor,
            ExportedProperty::BindablePropertyColorValue(0xff12_3456),
            638,
        ),
        (
            ExportedObjectKind::BindablePropertyAsset,
            ExportedProperty::BindablePropertyAssetValue(0),
            823,
        ),
        (
            ExportedObjectKind::BindablePropertyEnum,
            ExportedProperty::BindablePropertyEnumValue(1),
            637,
        ),
        (
            ExportedObjectKind::BindablePropertyTrigger,
            ExportedProperty::BindablePropertyTriggerValue(1),
            686,
        ),
        (
            ExportedObjectKind::BindablePropertyInteger,
            ExportedProperty::BindablePropertyIntegerValue(4),
            686,
        ),
    ] {
        assert_bindable_triplet(kind, value, property_key);
    }

    let instance = scene.instantiate(artboard)?;
    let progress = scene.vm_cursor(instance, defaults, progress)?;
    let label = scene.vm_string_cursor(instance, defaults, label)?;
    assert_eq!(scene.frame().get_vm(progress)?, 0.0);
    assert_eq!(scene.frame().get_vm_string(label)?, "idle");
    assert!(
        scene
            .frame()
            .pointer_down(instance, Vec2D::new(0.0, 0.0), 11)
    );
    assert!(scene.frame().pointer_up(instance, Vec2D::new(0.0, 0.0), 11));
    assert_eq!(scene.frame().get_vm(progress)?, 0.75);
    let opacity = scene.cursor(instance, shape, props::WORLD_OPACITY)?;
    let mut events = Vec::new();
    assert!(scene.frame().advance(instance, 0.0, &mut events));
    assert_eq!(scene.frame().get(opacity)?, 0.8);
    assert_eq!(
        scene.frame().get_vm_string(label)?,
        "idle",
        "the transition reports its event during layer advance, after this frame's applyEvents"
    );
    assert_eq!(
        events.len(),
        1,
        "the source event remains externally visible once"
    );
    // C++ `StateMachineInstance::applyEvents` runs at the NEXT frame start
    // and drains chained notifications to completion in that same call
    // (`state_machine_instance.cpp:2320-2343`). The event-listener write is
    // therefore visible after this one advance, while the source report is
    // not externally replayed.
    assert!(scene.frame().advance(instance, 0.0, &mut events));
    assert_eq!(scene.frame().get_vm_string(label)?, "reported");
    assert!(
        events.is_empty(),
        "Event-listener delivery does not replay the source"
    );

    let epoch = scene.epoch();
    let records_after = scene.export_records().into_records();
    assert_eq!(scene.epoch(), epoch);
    assert_eq!(
        records_after, records,
        "listener writes stay on the retained hot path"
    );
    assert!(scene.cursor(instance, shape, props::WORLD_OPACITY).is_ok());
    Ok(())
}

#[test]
fn authored_drag_listener_captures_pointer_and_dispatches_start_drag_end_after_exact_export()
-> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, machine), _) = scene.edit(|tx| {
        let (artboard, shape, machine, _, _) = create_authored_trigger_machine(tx)?;
        tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new("Hit target", 80.0, 80.0)),
        )?;

        let mut machines = tx.machines();
        let started = machines.create_number_input(
            machine,
            NumberInputSpec {
                name: "Started".into(),
                default_value: 0.0,
            },
        )?;
        let dragged = machines.create_number_input(
            machine,
            NumberInputSpec {
                name: "Dragged".into(),
                default_value: 0.0,
            },
        )?;
        let ended = machines.create_number_input(
            machine,
            NumberInputSpec {
                name: "Ended".into(),
                default_value: 0.0,
            },
        )?;
        for (name, listener_type, input, value) in [
            ("Start drag", MachineListenerType::DragStart, started, 1.0),
            ("Drag", MachineListenerType::Drag, dragged, 2.0),
            ("End drag", MachineListenerType::DragEnd, ended, 3.0),
        ] {
            let listener = machines.create_listener(
                machine,
                shape,
                MachineListenerSpec {
                    name: Some(name.into()),
                    listener_type,
                },
            )?;
            machines.add_listener_number_action(listener, input, value)?;
        }
        Ok((artboard, machine))
    })?;

    let records = scene.export_records().into_records();
    for listener_type in [
        MachineListenerType::DragStart,
        MachineListenerType::Drag,
        MachineListenerType::DragEnd,
    ] {
        assert!(records.iter().any(|record| {
            record.kind == ExportedObjectKind::StateMachineListener
                && record
                    .properties
                    .contains(&ExportedProperty::ListenerType(listener_type))
        }));
    }

    let instance = scene.instantiate(artboard)?;
    let started = scene.machine_number_input(instance, machine, "Started")?;
    let dragged = scene.machine_number_input(instance, machine, "Dragged")?;
    let ended = scene.machine_number_input(instance, machine, "Ended")?;
    assert!(
        scene
            .frame()
            .pointer_down(instance, Vec2D::new(0.0, 0.0), 7)
    );
    assert_eq!(scene.frame().get_number(started)?, 0.0);
    assert_eq!(scene.frame().get_number(dragged)?, 0.0);
    assert_eq!(scene.frame().get_number(ended)?, 0.0);

    assert!(
        !scene
            .frame()
            .pointer_move(instance, Vec2D::new(f32::NAN, 0.0), 0.016, 7,)
    );
    assert_eq!(scene.frame().get_number(started)?, 0.0);
    assert_eq!(scene.frame().get_number(dragged)?, 0.0);
    assert_eq!(scene.frame().get_number(ended)?, 0.0);

    assert!(
        scene
            .frame()
            .pointer_move(instance, Vec2D::new(10.0, 10.0), 0.016, 7)
    );
    assert_eq!(scene.frame().get_number(started)?, 1.0);
    assert_eq!(scene.frame().get_number(dragged)?, 2.0);
    assert_eq!(scene.frame().get_number(ended)?, 0.0);

    assert!(
        !scene
            .frame()
            .pointer_move(instance, Vec2D::new(90.0, 90.0), 0.016, 8),
        "a foreign pointer must not inherit another pointer's drag capture",
    );
    assert!(
        scene
            .frame()
            .pointer_move(instance, Vec2D::new(90.0, 90.0), 0.016, 7),
        "the armed pointer keeps driving drag actions outside the hit target",
    );
    assert_eq!(scene.frame().get_number(dragged)?, 2.0);

    assert!(
        scene
            .frame()
            .pointer_up(instance, Vec2D::new(10.0, 10.0), 7)
    );
    assert_eq!(scene.frame().get_number(ended)?, 3.0);
    assert!(
        !scene
            .frame()
            .pointer_move(instance, Vec2D::new(90.0, 90.0), 0.016, 7),
        "pointer up must release drag capture",
    );

    assert!(
        scene
            .frame()
            .pointer_down(instance, Vec2D::new(0.0, 0.0), 9)
    );
    assert!(
        scene
            .frame()
            .pointer_move(instance, Vec2D::new(10.0, 10.0), 0.016, 9)
    );
    assert!(
        !scene
            .frame()
            .pointer_exit(instance, Vec2D::new(10.0, 10.0), 9)
    );
    assert!(
        !scene
            .frame()
            .pointer_move(instance, Vec2D::new(90.0, 90.0), 0.016, 9),
        "pointer exit cancels and releases drag capture",
    );
    Ok(())
}

#[test]
fn authored_drag_lifecycle_stays_on_the_captured_target() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, machine), _) = scene.edit(|tx| {
        let (artboard, first, machine, _, _) = create_authored_trigger_machine(tx)?;
        tx.create(
            Parent::Object(first),
            NodeSpec::Rectangle(RectangleSpec::new("First hit target", 20.0, 20.0)),
        )?;
        let second = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Second target".into(),
                x: 70.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Object(second),
            NodeSpec::Rectangle(RectangleSpec::new("Second hit target", 20.0, 20.0)),
        )?;

        let mut machines = tx.machines();
        let mut add_lifecycle = |target, prefix: &str, base: f32| {
            let started = machines.create_number_input(
                machine,
                NumberInputSpec {
                    name: format!("{prefix} started"),
                    default_value: 0.0,
                },
            )?;
            let dragged = machines.create_number_input(
                machine,
                NumberInputSpec {
                    name: format!("{prefix} dragged"),
                    default_value: 0.0,
                },
            )?;
            let ended = machines.create_number_input(
                machine,
                NumberInputSpec {
                    name: format!("{prefix} ended"),
                    default_value: 0.0,
                },
            )?;
            for (listener_type, input, value) in [
                (MachineListenerType::DragStart, started, base + 1.0),
                (MachineListenerType::Drag, dragged, base + 2.0),
                (MachineListenerType::DragEnd, ended, base + 3.0),
            ] {
                let listener = machines.create_listener(
                    machine,
                    target,
                    MachineListenerSpec {
                        name: None,
                        listener_type,
                    },
                )?;
                machines.add_listener_number_action(listener, input, value)?;
            }
            Ok::<_, EditAbort>((started, dragged, ended))
        };
        let _ = add_lifecycle(first, "First", 0.0)?;
        let _ = add_lifecycle(second, "Second", 10.0)?;
        Ok((artboard, machine))
    })?;

    let instance = scene.instantiate(artboard)?;
    let first_started = scene.machine_number_input(instance, machine, "First started")?;
    let first_dragged = scene.machine_number_input(instance, machine, "First dragged")?;
    let first_ended = scene.machine_number_input(instance, machine, "First ended")?;
    let second_started = scene.machine_number_input(instance, machine, "Second started")?;
    let second_dragged = scene.machine_number_input(instance, machine, "Second dragged")?;
    let second_ended = scene.machine_number_input(instance, machine, "Second ended")?;

    assert!(
        scene
            .frame()
            .pointer_down(instance, Vec2D::new(0.0, 0.0), 17)
    );
    assert!(
        scene
            .frame()
            .pointer_move(instance, Vec2D::new(70.0, 0.0), 0.016, 17)
    );
    assert!(
        scene
            .frame()
            .pointer_up(instance, Vec2D::new(70.0, 0.0), 17)
    );

    assert_eq!(scene.frame().get_number(first_started)?, 1.0);
    assert_eq!(scene.frame().get_number(first_dragged)?, 2.0);
    assert_eq!(scene.frame().get_number(first_ended)?, 3.0);
    assert_eq!(scene.frame().get_number(second_started)?, 0.0);
    assert_eq!(scene.frame().get_number(second_dragged)?, 0.0);
    assert_eq!(scene.frame().get_number(second_ended)?, 0.0);
    Ok(())
}

#[test]
fn authored_listener_fire_event_survives_until_the_next_frame_report() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, shape, machine, event), _) = scene.edit(|tx| {
        let (artboard, shape, machine, _, event) = create_authored_trigger_machine(tx)?;
        tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new("Hit target", 80.0, 80.0)),
        )?;
        let fill = tx.create(
            Parent::Object(shape),
            NodeSpec::Fill(FillSpec {
                name: "Hit target fill".into(),
            }),
        )?;
        tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Hit target color".into(),
                color: 0xff112233,
            }),
        )?;
        let mut machines = tx.machines();
        let listener = machines.create_listener(
            machine,
            shape,
            MachineListenerSpec {
                name: Some("Report click".into()),
                listener_type: MachineListenerType::Click,
            },
        )?;
        machines.add_listener_fire_event_action(listener, event)?;
        Ok((artboard, shape, machine, event))
    })?;

    let records = scene.export_records().into_records();
    assert!(records.iter().any(|record| {
        record.kind == ExportedObjectKind::ListenerFireEvent
            && record
                .properties
                .iter()
                .any(|property| matches!(property, ExportedProperty::ListenerFireEventId(_)))
    }));

    let instance = scene.instantiate(artboard)?;
    let mut events = Vec::new();
    assert!(scene.frame().advance(instance, 0.0, &mut events));
    assert!(events.is_empty());
    assert!(
        scene
            .frame()
            .pointer_down(instance, Vec2D::new(0.0, 0.0), 9)
    );
    assert!(scene.frame().pointer_up(instance, Vec2D::new(0.0, 0.0), 9));
    assert!(scene.frame().advance(instance, 0.0, &mut events));
    let [
        SceneEvent::Authored {
            event: reported_event,
            name,
            string_properties,
            seconds_delay,
            context: Some(context),
        },
    ] = events.as_slice()
    else {
        panic!("listener event must retain its concrete pointer occurrence: {events:?}");
    };
    assert_eq!(*reported_event, event);
    assert_eq!(name.as_deref(), Some("Reached active"));
    assert!(string_properties.is_empty());
    assert_eq!(seconds_delay.to_bits(), 0.0f32.to_bits());
    assert_eq!(context.path().objects(), [shape]);
    assert!(context.occurrence().is_empty());
    scene.frame().advance(instance, 0.0, &mut events);
    assert!(events.is_empty(), "listener events remain frame scoped");
    assert!(scene.machine_input(instance, machine, "Go").is_ok());
    Ok(())
}

#[test]
fn transition_fire_event_reports_once_when_the_transition_is_taken() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, machine, event), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Canvas".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let idle = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Idle".into(),
                fps: 60,
                duration: 1,
            },
        )?;
        let active = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Active".into(),
                fps: 60,
                duration: 1,
            },
        )?;

        let mut machines = tx.machines();
        let event = machines.create_event(
            artboard,
            EventSpec {
                name: Some("Transition taken".into()),
            },
        )?;
        let machine = machines.create_machine(
            artboard,
            MachineSpec {
                name: Some("Switcher".into()),
            },
        )?;
        let trigger =
            machines.create_trigger_input(machine, TriggerInputSpec { name: "Go".into() })?;
        let layer = machines.create_layer(machine, MachineLayerSpec { name: None })?;
        let entry = machines.create_entry_state(layer)?;
        let any = machines.create_any_state(layer)?;
        machines.create_exit_state(layer)?;
        let idle_state =
            machines.create_animation_state(layer, AnimationStateSpec { animation: idle })?;
        let active_state =
            machines.create_animation_state(layer, AnimationStateSpec { animation: active })?;
        machines.create_transition(entry, idle_state)?;
        let transition = machines.create_transition(any, active_state)?;
        machines.add_trigger_condition(transition, trigger)?;
        machines.add_transition_fire_event(transition, event, FireEventOccurs::AtStart)?;
        Ok((artboard, machine, event))
    })?;

    let instance = scene.instantiate(artboard)?;
    let go = scene.machine_input(instance, machine, "Go")?;
    let mut events = Vec::new();
    scene.frame().advance(instance, 0.0, &mut events);
    assert!(events.is_empty());

    scene.frame().fire(go)?;
    scene.frame().advance(instance, 0.0, &mut events);
    assert_eq!(
        events,
        vec![SceneEvent::Authored {
            event,
            name: Some("Transition taken".into()),
            string_properties: Vec::new(),
            seconds_delay: 0.0,
            context: None,
        }]
    );

    scene.frame().advance(instance, 0.0, &mut events);
    assert!(events.is_empty(), "transition events fire only when taken");
    Ok(())
}

#[test]
fn authored_state_and_transition_options_export_typed_rive_properties() -> Result<()> {
    let mut scene = Scene::new();
    scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Canvas".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let animation = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Animated".into(),
                fps: 60,
                duration: 60,
            },
        )?;
        let mut machines = tx.machines();
        let machine = machines.create_machine(artboard, MachineSpec { name: None })?;
        let layer = machines.create_layer(machine, MachineLayerSpec { name: None })?;
        machines.create_any_state(layer)?;
        let entry = machines.create_entry_state(layer)?;
        machines.create_exit_state(layer)?;
        let animated = machines.create_animation_state_with_options(
            layer,
            AnimationStateSpec { animation },
            AnimationStateOptions {
                speed: 2.0,
                flags: MachineStateFlags {
                    random: true,
                    reset: true,
                },
            },
        )?;
        machines.create_transition_with_spec(
            entry,
            animated,
            MachineTransitionSpec {
                duration: 150,
                enable_exit_time: true,
                enable_early_exit: true,
                ..MachineTransitionSpec::default()
            },
        )?;
        Ok(())
    })?;

    let records = scene.export_records().into_records();
    let animation_state = records
        .iter()
        .find(|record| record.kind == ExportedObjectKind::AnimationState)
        .expect("typed animation state exports");
    assert!(
        animation_state
            .properties
            .contains(&ExportedProperty::LayerStateFlags(3))
    );
    assert!(
        animation_state
            .properties
            .contains(&ExportedProperty::StateSpeed(2.0))
    );
    let transition = records
        .iter()
        .find(|record| record.kind == ExportedObjectKind::StateTransition)
        .expect("typed transition exports");
    assert!(
        transition
            .properties
            .contains(&ExportedProperty::StateTransitionFlags((1 << 2) | (1 << 5)))
    );
    assert!(
        transition
            .properties
            .contains(&ExportedProperty::StateTransitionDuration(150))
    );
    Ok(())
}

#[test]
fn blend_states_author_and_execute_typed_input_and_literal_sources() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, shape, direct_shape, machine), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Canvas".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Fader".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let idle = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Idle".into(),
                fps: 60,
                duration: 1,
            },
        )?;
        tx.animations()
            .set_key(idle, shape, props::WORLD_OPACITY, 0, 0.2)?;
        let active = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Active".into(),
                fps: 60,
                duration: 1,
            },
        )?;
        tx.animations()
            .set_key(active, shape, props::WORLD_OPACITY, 0, 0.8)?;
        let direct_shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Direct Fader".into(),
                x: 0.0,
                y: 0.0,
                opacity: 0.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let direct_animation = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Direct".into(),
                fps: 60,
                duration: 1,
            },
        )?;
        tx.animations()
            .set_key(direct_animation, direct_shape, props::WORLD_OPACITY, 0, 0.8)?;

        let mut machines = tx.machines();
        let machine = machines.create_machine(
            artboard,
            MachineSpec {
                name: Some("Blend".into()),
            },
        )?;
        let amount = machines.create_number_input(
            machine,
            NumberInputSpec {
                name: "Amount".into(),
                default_value: 0.5,
            },
        )?;
        let layer = machines.create_layer(machine, MachineLayerSpec { name: None })?;
        let entry = machines.create_entry_state(layer)?;
        machines.create_any_state(layer)?;
        machines.create_exit_state(layer)?;
        let blend = machines.create_blend_1d_state(
            layer,
            BlendState1DSpec {
                value: BlendValueSource::StateMachineInput(amount),
                flags: MachineStateFlags {
                    reset: true,
                    ..MachineStateFlags::default()
                },
            },
        )?;
        machines.add_blend_animation_1d(
            blend,
            BlendAnimation1DSpec {
                animation: active,
                value: 1.0,
            },
        )?;
        machines.add_blend_animation_1d(
            blend,
            BlendAnimation1DSpec {
                animation: idle,
                value: 0.0,
            },
        )?;
        machines.create_transition(entry, blend)?;

        let direct_layer = machines.create_layer(
            machine,
            MachineLayerSpec {
                name: Some("Direct".into()),
            },
        )?;
        let direct_entry = machines.create_entry_state(direct_layer)?;
        machines.create_any_state(direct_layer)?;
        machines.create_exit_state(direct_layer)?;
        let direct =
            machines.create_blend_direct_state(direct_layer, MachineStateFlags::default())?;
        machines.add_blend_animation_direct(
            direct,
            BlendAnimationDirectSpec {
                animation: direct_animation,
                mix: BlendValueSource::Literal(25.0),
            },
        )?;
        machines.create_transition(direct_entry, direct)?;
        Ok((artboard, shape, direct_shape, machine))
    })?;

    let records = scene.export_records().into_records();
    assert!(records.iter().any(|record| {
        record.kind == ExportedObjectKind::BlendState1DInput
            && record
                .properties
                .contains(&ExportedProperty::BlendState1DInputId(0))
    }));
    assert_eq!(
        records
            .iter()
            .filter(|record| record.kind == ExportedObjectKind::BlendAnimation1D)
            .map(|record| record
                .properties
                .iter()
                .find_map(|property| match property {
                    ExportedProperty::BlendAnimation1DValue(value) => Some(*value),
                    _ => None,
                })
                .expect("1D threshold"))
            .collect::<Vec<_>>(),
        vec![0.0, 1.0],
        "1D blend children export in threshold order"
    );
    assert!(records.iter().any(|record| {
        record.kind == ExportedObjectKind::BlendAnimationDirect
            && record
                .properties
                .contains(&ExportedProperty::BlendAnimationDirectSource(1))
            && record
                .properties
                .contains(&ExportedProperty::BlendAnimationDirectMix(25.0))
    }));

    let instance = scene.instantiate(artboard)?;
    let opacity = scene.cursor(instance, shape, props::WORLD_OPACITY)?;
    let direct_opacity = scene.cursor(instance, direct_shape, props::WORLD_OPACITY)?;
    let amount = scene.machine_number_input(instance, machine, "Amount")?;
    let mut events = Vec::new();
    scene.frame().advance(instance, 0.0, &mut events);
    assert_eq!(scene.frame().get(opacity)?, 0.5);
    assert_eq!(scene.frame().get(direct_opacity)?, 0.2);
    assert!(scene.frame().set_number(amount, 1.0)?);
    scene.frame().advance(instance, 0.0, &mut events);
    assert_eq!(scene.frame().get(opacity)?, 0.8);
    Ok(())
}

#[test]
fn blend_state_authoring_rejects_foreign_and_invalid_sources_atomically() -> Result<()> {
    let mut scene = Scene::new();
    let (
        (
            artboard,
            other_artboard,
            layer,
            blend_1d,
            blend_direct,
            animation,
            foreign,
            other_animation,
        ),
        _,
    ) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Canvas".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let animation = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Local".into(),
                fps: 60,
                duration: 1,
            },
        )?;
        let other_artboard = tx.create_artboard(ArtboardSpec {
            name: "Other".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let other_animation = tx.animations().create_linear(
            other_artboard,
            LinearAnimationSpec {
                name: "Foreign timeline".into(),
                fps: 60,
                duration: 1,
            },
        )?;

        let mut machines = tx.machines();
        let machine = machines.create_machine(artboard, MachineSpec { name: None })?;
        let layer = machines.create_layer(machine, MachineLayerSpec { name: None })?;
        machines.create_any_state(layer)?;
        machines.create_entry_state(layer)?;
        machines.create_exit_state(layer)?;
        let blend_1d = machines.create_blend_1d_state(
            layer,
            BlendState1DSpec {
                value: BlendValueSource::Literal(0.5),
                flags: MachineStateFlags::default(),
            },
        )?;
        let blend_direct =
            machines.create_blend_direct_state(layer, MachineStateFlags::default())?;

        let foreign_machine = machines.create_machine(
            artboard,
            MachineSpec {
                name: Some("Foreign".into()),
            },
        )?;
        let foreign = machines.create_number_input(
            foreign_machine,
            NumberInputSpec {
                name: "Foreign amount".into(),
                default_value: 0.0,
            },
        )?;
        let foreign_layer =
            machines.create_layer(foreign_machine, MachineLayerSpec { name: None })?;
        machines.create_any_state(foreign_layer)?;
        machines.create_entry_state(foreign_layer)?;
        machines.create_exit_state(foreign_layer)?;
        Ok((
            artboard,
            other_artboard,
            layer,
            blend_1d,
            blend_direct,
            animation,
            foreign,
            other_animation,
        ))
    })?;
    let epoch = scene.epoch();
    let records = scene.export_records();

    let foreign_input = scene
        .edit(|tx| {
            tx.machines().create_blend_1d_state(
                layer,
                BlendState1DSpec {
                    value: BlendValueSource::StateMachineInput(foreign),
                    flags: MachineStateFlags::default(),
                },
            )?;
            Ok(())
        })
        .expect_err("a blend state cannot consume another machine's input");
    assert_eq!(foreign_input.kind(), EditErrorKind::Aborted);
    assert_eq!(
        foreign_input.diagnostic().reason,
        EditReason::InvalidMachineReference
    );
    assert_eq!(scene.epoch(), epoch);
    assert_eq!(scene.export_records(), records);

    let cross_artboard_animation = scene
        .edit(|tx| {
            tx.machines().add_blend_animation_1d(
                blend_1d,
                BlendAnimation1DSpec {
                    animation: other_animation,
                    value: 1.0,
                },
            )?;
            Ok(())
        })
        .expect_err("a blend child cannot reference another artboard's timeline");
    assert_eq!(cross_artboard_animation.kind(), EditErrorKind::Aborted);
    assert_eq!(
        cross_artboard_animation.diagnostic().reason,
        EditReason::CrossArtboardReference {
            source: artboard,
            target: other_artboard,
        }
    );
    assert_eq!(scene.epoch(), epoch);
    assert_eq!(scene.export_records(), records);

    let non_finite_threshold = scene
        .edit(|tx| {
            tx.machines().add_blend_animation_1d(
                blend_1d,
                BlendAnimation1DSpec {
                    animation,
                    value: f32::NAN,
                },
            )?;
            Ok(())
        })
        .expect_err("a 1D threshold must be finite");
    assert_eq!(non_finite_threshold.kind(), EditErrorKind::Aborted);
    assert_eq!(
        non_finite_threshold.diagnostic().reason,
        EditReason::NonFiniteProperty { property: "value" }
    );
    assert_eq!(scene.epoch(), epoch);
    assert_eq!(scene.export_records(), records);

    let non_finite_mix = scene
        .edit(|tx| {
            tx.machines().add_blend_animation_direct(
                blend_direct,
                BlendAnimationDirectSpec {
                    animation,
                    mix: BlendValueSource::Literal(f32::INFINITY),
                },
            )?;
            Ok(())
        })
        .expect_err("a direct mix must be finite");
    assert_eq!(non_finite_mix.kind(), EditErrorKind::Aborted);
    assert_eq!(
        non_finite_mix.diagnostic().reason,
        EditReason::NonFiniteProperty {
            property: "blend_value"
        }
    );
    assert_eq!(scene.epoch(), epoch);
    assert_eq!(scene.export_records(), records);

    let wrong_state_kind = scene
        .edit(|tx| {
            tx.machines().add_blend_animation_direct(
                blend_1d,
                BlendAnimationDirectSpec {
                    animation,
                    mix: BlendValueSource::Literal(25.0),
                },
            )?;
            Ok(())
        })
        .expect_err("direct children require a direct blend state");
    assert_eq!(wrong_state_kind.kind(), EditErrorKind::Aborted);
    assert!(matches!(
        wrong_state_kind.diagnostic().reason,
        EditReason::RecordPropertyOwnerMismatch {
            property: "state",
            ..
        }
    ));
    assert_eq!(scene.epoch(), epoch);
    assert_eq!(scene.export_records(), records);
    Ok(())
}

#[test]
fn view_model_numbers_drive_blend_states_without_reauthoring_records() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, shape, direct_shape, defaults, amount, direct_mix), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Canvas".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "One dimensional".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let direct_shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Direct".into(),
                x: 0.0,
                y: 0.0,
                opacity: 0.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let idle = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Idle".into(),
                fps: 60,
                duration: 1,
            },
        )?;
        tx.animations()
            .set_key(idle, shape, props::WORLD_OPACITY, 0, 0.2)?;
        let active = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Active".into(),
                fps: 60,
                duration: 1,
            },
        )?;
        tx.animations()
            .set_key(active, shape, props::WORLD_OPACITY, 0, 0.8)?;
        let direct = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Direct".into(),
                fps: 60,
                duration: 1,
            },
        )?;
        tx.animations()
            .set_key(direct, direct_shape, props::WORLD_OPACITY, 0, 0.8)?;

        let (defaults, amount, direct_mix) = {
            let mut view_models = tx.view_models();
            let model = view_models.create(ViewModelSpec {
                scope: ViewModelScope::Local,
                name: "Blend values".into(),
            })?;
            let amount = view_models.create_number(
                model,
                ViewModelNumberSpec {
                    name: "Amount".into(),
                },
            )?;
            let direct_mix = view_models.create_number(
                model,
                ViewModelNumberSpec {
                    name: "Direct mix".into(),
                },
            )?;
            let defaults = view_models.create_instance(
                model,
                ViewModelInstanceSpec {
                    name: Some("Defaults".into()),
                },
            )?;
            view_models.set_number(defaults, amount, 0.5)?;
            view_models.set_number(defaults, direct_mix, 25.0)?;
            view_models.set_artboard_default(artboard, defaults)?;
            (defaults, amount, direct_mix)
        };

        let mut machines = tx.machines();
        let machine = machines.create_machine(
            artboard,
            MachineSpec {
                name: Some("Blend".into()),
            },
        )?;
        let layer = machines.create_layer(machine, MachineLayerSpec { name: None })?;
        let entry = machines.create_entry_state(layer)?;
        machines.create_any_state(layer)?;
        machines.create_exit_state(layer)?;
        let blend = machines.create_blend_1d_state(
            layer,
            BlendState1DSpec {
                value: BlendValueSource::ViewModelNumber(amount),
                flags: MachineStateFlags {
                    reset: true,
                    ..MachineStateFlags::default()
                },
            },
        )?;
        machines.add_blend_animation_1d(
            blend,
            BlendAnimation1DSpec {
                animation: idle,
                value: 0.0,
            },
        )?;
        machines.add_blend_animation_1d(
            blend,
            BlendAnimation1DSpec {
                animation: active,
                value: 1.0,
            },
        )?;
        machines.create_transition(entry, blend)?;

        let direct_layer = machines.create_layer(machine, MachineLayerSpec { name: None })?;
        let direct_entry = machines.create_entry_state(direct_layer)?;
        machines.create_any_state(direct_layer)?;
        machines.create_exit_state(direct_layer)?;
        let direct_blend =
            machines.create_blend_direct_state(direct_layer, MachineStateFlags::default())?;
        machines.add_blend_animation_direct(
            direct_blend,
            BlendAnimationDirectSpec {
                animation: direct,
                mix: BlendValueSource::ViewModelNumber(direct_mix),
            },
        )?;
        machines.create_transition(direct_entry, direct_blend)?;
        Ok((artboard, shape, direct_shape, defaults, amount, direct_mix))
    })?;

    let records = scene.export_records().into_records();
    assert!(
        records
            .iter()
            .any(|record| record.kind == ExportedObjectKind::BlendState1DViewModel)
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record.kind == ExportedObjectKind::BindablePropertyNumber)
            .count(),
        2,
    );

    let instance = scene.instantiate(artboard)?;
    let amount = scene.vm_cursor(instance, defaults, amount)?;
    let direct_mix = scene.vm_cursor(instance, defaults, direct_mix)?;
    let opacity = scene.cursor(instance, shape, props::WORLD_OPACITY)?;
    let direct_opacity = scene.cursor(instance, direct_shape, props::WORLD_OPACITY)?;
    let mut events = Vec::new();
    scene.frame().advance(instance, 0.0, &mut events);
    assert!((scene.frame().get(opacity)? - 0.5).abs() < 0.000_001);
    let initial_direct_opacity = scene.frame().get(direct_opacity)?;
    assert!(
        (initial_direct_opacity - 0.2).abs() < 0.000_001,
        "expected the initial 25% ViewModel mix to produce 0.2, got {initial_direct_opacity}",
    );

    let epoch = scene.epoch();
    assert!(scene.frame().set_vm(amount, 1.0)?);
    assert!(scene.frame().set_vm(direct_mix, 50.0)?);
    scene.frame().advance(instance, 0.0, &mut events);
    assert!((scene.frame().get(opacity)? - 0.8).abs() < 0.000_001);
    let updated_direct_opacity = scene.frame().get(direct_opacity)?;
    assert!(
        (updated_direct_opacity - 0.5).abs() < 0.000_001,
        "expected a 50% ViewModel mix from the prior 0.2 frame to produce 0.5, got {updated_direct_opacity}",
    );
    assert_eq!(scene.epoch(), epoch);
    assert_eq!(scene.export_records().into_records(), records);
    Ok(())
}

#[test]
fn boolean_machine_input_changes_visual_state_without_rebuilding_the_scene() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, shape, machine, event), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Canvas".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Fader".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let idle = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Idle".into(),
                fps: 60,
                duration: 1,
            },
        )?;
        tx.animations()
            .set_key(idle, shape, props::WORLD_OPACITY, 0, 0.2)?;
        let active = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Active".into(),
                fps: 60,
                duration: 1,
            },
        )?;
        tx.animations()
            .set_key(active, shape, props::WORLD_OPACITY, 0, 0.8)?;

        let mut machines = tx.machines();
        let event = machines.create_event(
            artboard,
            EventSpec {
                name: Some("Reached active".into()),
            },
        )?;
        let machine = machines.create_machine(
            artboard,
            MachineSpec {
                name: Some("Switcher".into()),
            },
        )?;
        let armed = machines.create_boolean_input(
            machine,
            BooleanInputSpec {
                name: "Armed".into(),
                default_value: false,
            },
        )?;
        let layer = machines.create_layer(machine, MachineLayerSpec { name: None })?;
        let entry = machines.create_entry_state(layer)?;
        let any = machines.create_any_state(layer)?;
        machines.create_exit_state(layer)?;
        let idle_state =
            machines.create_animation_state(layer, AnimationStateSpec { animation: idle })?;
        let active_state =
            machines.create_animation_state(layer, AnimationStateSpec { animation: active })?;
        machines.create_transition(entry, idle_state)?;
        let transition = machines.create_transition(any, active_state)?;
        machines.add_boolean_equals_condition(transition, armed, true)?;
        machines.add_fire_event(active_state, event, FireEventOccurs::AtStart)?;
        Ok((artboard, shape, machine, event))
    })?;

    assert!(scene.export_records().records().iter().any(|record| {
        record.kind == ExportedObjectKind::StateMachineBoolean
            && record.properties
                == vec![
                    ExportedProperty::StateMachineComponentName("Armed".into()),
                    ExportedProperty::StateMachineBooleanValue(false),
                ]
    }));
    assert!(scene.export_records().records().iter().any(|record| {
        record.kind == ExportedObjectKind::TransitionBooleanEqualsCondition
            && record.properties
                == vec![
                    ExportedProperty::StateMachineInputId(0),
                    ExportedProperty::BooleanEqualsValue(true),
                ]
    }));

    let instance = scene.instantiate(artboard)?;
    let opacity = scene.cursor(instance, shape, props::WORLD_OPACITY)?;
    let armed = scene.machine_boolean_input(instance, machine, "Armed")?;
    assert!(matches!(
        scene.machine_input(instance, machine, "Armed"),
        Err(ResolveError::UnsupportedInputKind)
    ));
    let epoch = scene.epoch();
    let records = scene.export_records().into_records();
    let mut events = Vec::new();

    assert!(scene.frame().advance(instance, 0.0, &mut events));
    assert_eq!(scene.frame().get(opacity)?, 0.2);
    assert!(events.is_empty());
    assert!(scene.frame().set_boolean(armed, true)?);
    assert!(!scene.frame().set_boolean(armed, true)?);
    assert_eq!(scene.epoch(), epoch);
    assert_eq!(scene.export_records().into_records(), records);
    assert!(scene.frame().advance(instance, 0.0, &mut events));
    assert_eq!(scene.frame().get(opacity)?, 0.8);
    assert_eq!(
        events,
        vec![SceneEvent::Authored {
            event,
            name: Some("Reached active".into()),
            string_properties: Vec::new(),
            seconds_delay: 0.0,
            context: None,
        }]
    );

    scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Other".into(),
            width: 10.0,
            height: 10.0,
        })
    })?;
    assert_eq!(scene.frame().set_boolean(armed, false), Err(StaleCursor));
    Ok(())
}

#[test]
fn number_input_and_comparators_export_typed_rive_encodings() -> Result<()> {
    let mut scene = Scene::new();
    scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Canvas".into(),
            width: 10.0,
            height: 10.0,
        })?;
        let mut machines = tx.machines();
        let machine = machines.create_machine(artboard, MachineSpec { name: None })?;
        let amount = machines.create_number_input(
            machine,
            NumberInputSpec {
                name: "Amount".into(),
                default_value: 0.25,
            },
        )?;
        let layer = machines.create_layer(machine, MachineLayerSpec { name: None })?;
        machines.create_any_state(layer)?;
        let entry = machines.create_entry_state(layer)?;
        let exit = machines.create_exit_state(layer)?;
        let transition = machines.create_transition(entry, exit)?;
        for (comparator, target) in [
            (NumberComparator::Eq, 1.0),
            (NumberComparator::Neq, 2.0),
            (NumberComparator::Lte, 3.0),
            (NumberComparator::Gte, 4.0),
            (NumberComparator::Lt, 5.0),
            (NumberComparator::Gt, 6.0),
        ] {
            machines.add_number_condition(transition, amount, comparator, target)?;
        }
        Ok(())
    })?;

    assert!(scene.export_records().records().iter().any(|record| {
        record.kind == ExportedObjectKind::StateMachineNumber
            && record.properties
                == vec![
                    ExportedProperty::StateMachineComponentName("Amount".into()),
                    ExportedProperty::StateMachineNumberValue(0.25),
                ]
    }));
    let conditions = scene
        .export_records()
        .into_records()
        .into_iter()
        .filter(|record| record.kind == ExportedObjectKind::TransitionNumberCondition)
        .collect::<Vec<_>>();
    assert_eq!(conditions.len(), 6);
    for (index, record) in conditions.iter().enumerate() {
        assert_eq!(
            record.properties,
            vec![
                ExportedProperty::StateMachineInputId(0),
                ExportedProperty::NumberConditionComparator(
                    [
                        NumberComparator::Eq,
                        NumberComparator::Neq,
                        NumberComparator::Lte,
                        NumberComparator::Gte,
                        NumberComparator::Lt,
                        NumberComparator::Gt,
                    ][index],
                ),
                ExportedProperty::NumberConditionTarget(index as f32 + 1.0),
            ]
        );
        assert_eq!(
            u32::from(
                [
                    NumberComparator::Eq,
                    NumberComparator::Neq,
                    NumberComparator::Lte,
                    NumberComparator::Gte,
                    NumberComparator::Lt,
                    NumberComparator::Gt,
                ][index]
            ),
            index as u32
        );
    }
    Ok(())
}

#[test]
fn number_input_cursor_reads_and_writes_retained_machine_without_rebuilding() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, other_artboard, shape, machine), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Canvas".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let other_artboard = tx.create_artboard(ArtboardSpec {
            name: "Other".into(),
            width: 10.0,
            height: 10.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Fader".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let idle = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Idle".into(),
                fps: 60,
                duration: 1,
            },
        )?;
        tx.animations()
            .set_key(idle, shape, props::WORLD_OPACITY, 0, 0.2)?;
        let active = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Active".into(),
                fps: 60,
                duration: 1,
            },
        )?;
        tx.animations()
            .set_key(active, shape, props::WORLD_OPACITY, 0, 0.8)?;
        let mut machines = tx.machines();
        let machine = machines.create_machine(artboard, MachineSpec { name: None })?;
        let amount = machines.create_number_input(
            machine,
            NumberInputSpec {
                name: "Amount".into(),
                default_value: 0.25,
            },
        )?;
        machines.create_trigger_input(machine, TriggerInputSpec { name: "Go".into() })?;
        let layer = machines.create_layer(machine, MachineLayerSpec { name: None })?;
        let entry = machines.create_entry_state(layer)?;
        let any = machines.create_any_state(layer)?;
        machines.create_exit_state(layer)?;
        let idle_state =
            machines.create_animation_state(layer, AnimationStateSpec { animation: idle })?;
        let active_state =
            machines.create_animation_state(layer, AnimationStateSpec { animation: active })?;
        machines.create_transition(entry, idle_state)?;
        let transition = machines.create_transition(any, active_state)?;
        machines.add_number_condition(transition, amount, NumberComparator::Gte, 0.75)?;
        Ok((artboard, other_artboard, shape, machine))
    })?;
    let instance = scene.instantiate(artboard)?;
    let other_instance = scene.instantiate(other_artboard)?;
    let opacity = scene.cursor(instance, shape, props::WORLD_OPACITY)?;
    let amount = scene.machine_number_input(instance, machine, "Amount")?;

    assert_eq!(scene.frame().get_number(amount)?, 0.25);
    assert!(matches!(
        scene.machine_number_input(instance, machine, "Go"),
        Err(ResolveError::UnsupportedInputKind)
    ));
    assert!(matches!(
        scene.machine_number_input(other_instance, machine, "Amount"),
        Err(ResolveError::DifferentArtboard)
    ));

    let epoch = scene.epoch();
    let records = scene.export_records();
    let mut events = Vec::new();
    assert!(scene.frame().advance(instance, 0.0, &mut events));
    assert_eq!(scene.frame().get(opacity)?, 0.2);
    assert!(!scene.frame().set_number(amount, f32::NAN)?);
    assert_eq!(scene.frame().get_number(amount)?, 0.25);
    assert!(scene.frame().set_number(amount, 0.75)?);
    assert!(!scene.frame().set_number(amount, 0.75)?);
    assert_eq!(scene.frame().get_number(amount)?, 0.75);
    assert_eq!(scene.epoch(), epoch);
    assert_eq!(scene.export_records(), records);

    assert!(scene.frame().advance(instance, 0.0, &mut events));
    assert_eq!(scene.frame().get(opacity)?, 0.8);

    scene.edit(|tx| {
        tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Structural edit".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        Ok(())
    })?;
    assert_eq!(scene.frame().get_number(amount), Err(StaleCursor));
    assert_eq!(scene.frame().set_number(amount, 0.5), Err(StaleCursor));
    Ok(())
}

#[test]
fn number_input_authoring_validates_names_values_kinds_and_machine_ownership() -> Result<()> {
    let mut scene = Scene::new();
    let ((machine, transition, trigger, foreign_number), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Canvas".into(),
            width: 10.0,
            height: 10.0,
        })?;
        let mut machines = tx.machines();
        let machine = machines.create_machine(
            artboard,
            MachineSpec {
                name: Some("Main".into()),
            },
        )?;
        machines.create_number_input(
            machine,
            NumberInputSpec {
                name: "Amount".into(),
                default_value: 0.0,
            },
        )?;
        let trigger =
            machines.create_trigger_input(machine, TriggerInputSpec { name: "Go".into() })?;
        let layer = machines.create_layer(machine, MachineLayerSpec { name: None })?;
        machines.create_any_state(layer)?;
        let entry = machines.create_entry_state(layer)?;
        let exit = machines.create_exit_state(layer)?;
        let transition = machines.create_transition(entry, exit)?;

        let foreign_machine = machines.create_machine(
            artboard,
            MachineSpec {
                name: Some("Foreign".into()),
            },
        )?;
        let foreign_number = machines.create_number_input(
            foreign_machine,
            NumberInputSpec {
                name: "Foreign amount".into(),
                default_value: 0.0,
            },
        )?;
        let foreign_layer =
            machines.create_layer(foreign_machine, MachineLayerSpec { name: None })?;
        machines.create_any_state(foreign_layer)?;
        machines.create_entry_state(foreign_layer)?;
        machines.create_exit_state(foreign_layer)?;
        Ok((machine, transition, trigger, foreign_number))
    })?;
    let epoch = scene.epoch();
    let records = scene.export_records();

    for (case, error, expected) in [
        (
            "number reuses trigger name",
            scene
                .edit(|tx| {
                    tx.machines().create_number_input(
                        machine,
                        NumberInputSpec {
                            name: "Go".into(),
                            default_value: 0.0,
                        },
                    )?;
                    Ok(())
                })
                .expect_err("number inputs share the trigger namespace"),
            EditReason::DuplicateMachineInputName,
        ),
        (
            "trigger reuses number name",
            scene
                .edit(|tx| {
                    tx.machines().create_trigger_input(
                        machine,
                        TriggerInputSpec {
                            name: "Amount".into(),
                        },
                    )?;
                    Ok(())
                })
                .expect_err("trigger inputs share the number namespace"),
            EditReason::DuplicateMachineInputName,
        ),
        (
            "boolean reuses number name",
            scene
                .edit(|tx| {
                    tx.machines().create_boolean_input(
                        machine,
                        BooleanInputSpec {
                            name: "Amount".into(),
                            default_value: false,
                        },
                    )?;
                    Ok(())
                })
                .expect_err("boolean inputs share the number namespace"),
            EditReason::DuplicateMachineInputName,
        ),
        (
            "blank number name",
            scene
                .edit(|tx| {
                    tx.machines().create_number_input(
                        machine,
                        NumberInputSpec {
                            name: " \n ".into(),
                            default_value: 0.0,
                        },
                    )?;
                    Ok(())
                })
                .expect_err("number input names are runtime resolution keys"),
            EditReason::EmptyMachineInputName,
        ),
        (
            "nonfinite default",
            scene
                .edit(|tx| {
                    tx.machines().create_number_input(
                        machine,
                        NumberInputSpec {
                            name: "Invalid".into(),
                            default_value: f32::INFINITY,
                        },
                    )?;
                    Ok(())
                })
                .expect_err("number defaults must be finite"),
            EditReason::NonFiniteProperty {
                property: "default_value",
            },
        ),
        (
            "wrong input kind",
            scene
                .edit(|tx| {
                    tx.machines().add_number_condition(
                        transition,
                        trigger,
                        NumberComparator::Eq,
                        0.0,
                    )?;
                    Ok(())
                })
                .expect_err("number conditions require number inputs"),
            EditReason::RecordPropertyOwnerMismatch {
                property: "input",
                actual: nuxie::AuthoredObjectKind::MachineTrigger,
            },
        ),
        (
            "foreign machine input",
            scene
                .edit(|tx| {
                    tx.machines().add_number_condition(
                        transition,
                        foreign_number,
                        NumberComparator::Eq,
                        0.0,
                    )?;
                    Ok(())
                })
                .expect_err("number conditions cannot consume another machine's input"),
            EditReason::InvalidMachineReference,
        ),
        (
            "nonfinite target",
            scene
                .edit(|tx| {
                    tx.machines().add_number_condition(
                        transition,
                        foreign_number,
                        NumberComparator::Eq,
                        f32::NAN,
                    )?;
                    Ok(())
                })
                .expect_err("number condition targets must be finite"),
            EditReason::NonFiniteProperty { property: "target" },
        ),
    ] {
        assert_eq!(error.kind(), EditErrorKind::Aborted, "{case}");
        assert_eq!(error.diagnostic().reason, expected, "{case}");
        assert_eq!(scene.epoch(), epoch, "{case}");
        assert_eq!(scene.export_records(), records, "{case}");
    }
    Ok(())
}

#[test]
fn number_input_remove_restore_preserves_default_and_rejects_duplicate_name() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, machine, number), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Canvas".into(),
            width: 10.0,
            height: 10.0,
        })?;
        let mut machines = tx.machines();
        let machine = machines.create_machine(artboard, MachineSpec { name: None })?;
        let number = machines.create_number_input(
            machine,
            NumberInputSpec {
                name: "Amount".into(),
                default_value: 0.25,
            },
        )?;
        let layer = machines.create_layer(machine, MachineLayerSpec { name: None })?;
        machines.create_any_state(layer)?;
        machines.create_entry_state(layer)?;
        machines.create_exit_state(layer)?;
        Ok((artboard, machine, number))
    })?;
    let records_before = scene.export_records();
    let instance = scene.instantiate(artboard)?;
    let original = scene.machine_number_input(instance, machine, "Amount")?;
    assert_eq!(scene.frame().get_number(original)?, 0.25);

    let (removed, _) = scene.edit(|tx| tx.remove(number.object_id()))?;
    assert_eq!(scene.frame().get_number(original), Err(StaleCursor));
    scene.edit(|tx| tx.restore(removed).map(|_| ()))?;
    assert_eq!(scene.export_records(), records_before);
    let restored = scene.machine_number_input(instance, machine, "Amount")?;
    assert_eq!(scene.frame().get_number(restored)?, 0.25);

    let (removed, _) = scene.edit(|tx| tx.remove(number.object_id()))?;
    scene.edit(|tx| {
        tx.machines().create_trigger_input(
            machine,
            TriggerInputSpec {
                name: "Amount".into(),
            },
        )?;
        Ok(())
    })?;
    let epoch = scene.epoch();
    let records = scene.export_records();
    let error = scene
        .edit(|tx| tx.restore(removed).map(|_| ()))
        .expect_err("restoring a number cannot bypass input-name uniqueness");
    assert_eq!(error.kind(), EditErrorKind::CommitRejected);
    assert_eq!(
        error.diagnostic().reason,
        EditReason::DuplicateMachineInputName
    );
    assert_eq!(scene.epoch(), epoch);
    assert_eq!(scene.export_records(), records);
    assert!(scene.machine_input(instance, machine, "Amount").is_ok());
    assert!(matches!(
        scene.machine_number_input(instance, machine, "Amount"),
        Err(ResolveError::UnsupportedInputKind)
    ));
    Ok(())
}

#[test]
fn boolean_and_trigger_inputs_share_one_closed_name_namespace() -> Result<()> {
    let mut scene = Scene::new();
    let (machine, _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Canvas".into(),
            width: 10.0,
            height: 10.0,
        })?;
        let mut machines = tx.machines();
        let machine = machines.create_machine(artboard, MachineSpec { name: None })?;
        machines.create_trigger_input(machine, TriggerInputSpec { name: "Go".into() })?;
        machines.create_boolean_input(
            machine,
            BooleanInputSpec {
                name: "Armed".into(),
                default_value: false,
            },
        )?;
        let layer = machines.create_layer(machine, MachineLayerSpec { name: None })?;
        machines.create_entry_state(layer)?;
        machines.create_any_state(layer)?;
        machines.create_exit_state(layer)?;
        Ok(machine)
    })?;
    let epoch = scene.epoch();
    let records = scene.export_records();

    for error in [
        scene
            .edit(|tx| {
                tx.machines().create_boolean_input(
                    machine,
                    BooleanInputSpec {
                        name: "Go".into(),
                        default_value: false,
                    },
                )?;
                Ok(())
            })
            .expect_err("a boolean cannot reuse a trigger name"),
        scene
            .edit(|tx| {
                tx.machines().create_trigger_input(
                    machine,
                    TriggerInputSpec {
                        name: "Armed".into(),
                    },
                )?;
                Ok(())
            })
            .expect_err("a trigger cannot reuse a boolean name"),
    ] {
        assert_eq!(error.kind(), EditErrorKind::Aborted);
        assert_eq!(
            error.diagnostic().reason,
            EditReason::DuplicateMachineInputName
        );
    }
    assert_eq!(scene.epoch(), epoch);
    assert_eq!(scene.export_records(), records);

    let error = scene
        .edit(|tx| {
            tx.machines().create_boolean_input(
                machine,
                BooleanInputSpec {
                    name: " \n ".into(),
                    default_value: false,
                },
            )?;
            Ok(())
        })
        .expect_err("boolean input names must be usable runtime keys");
    assert_eq!(error.kind(), EditErrorKind::Aborted);
    assert_eq!(error.diagnostic().reason, EditReason::EmptyMachineInputName);
    assert_eq!(scene.epoch(), epoch);
    assert_eq!(scene.export_records(), records);
    Ok(())
}

#[test]
fn multiple_machines_advance_once_and_report_events_in_authored_order() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, first_machine, first_event, second_machine, second_event), _) =
        scene.edit(|tx| {
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Canvas".into(),
                width: 100.0,
                height: 100.0,
            })?;
            let shape = tx.create(
                Parent::Artboard(artboard),
                NodeSpec::Shape(ShapeSpec {
                    name: "Fader".into(),
                    x: 0.0,
                    y: 0.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                }),
            )?;
            let idle = tx.animations().create_linear(
                artboard,
                LinearAnimationSpec {
                    name: "Idle".into(),
                    fps: 60,
                    duration: 1,
                },
            )?;
            tx.animations()
                .set_key(idle, shape, props::WORLD_OPACITY, 0, 0.2)?;
            let active = tx.animations().create_linear(
                artboard,
                LinearAnimationSpec {
                    name: "Active".into(),
                    fps: 60,
                    duration: 1,
                },
            )?;
            tx.animations()
                .set_key(active, shape, props::WORLD_OPACITY, 0, 0.8)?;
            let (first_machine, _, first_event) = append_authored_trigger_machine(
                tx,
                artboard,
                idle,
                active,
                "First",
                "First go",
                "First event",
            )?;
            let (second_machine, _, second_event) = append_authored_trigger_machine(
                tx,
                artboard,
                idle,
                active,
                "Second",
                "Second go",
                "Second event",
            )?;
            Ok((
                artboard,
                first_machine,
                first_event,
                second_machine,
                second_event,
            ))
        })?;
    let instance = scene.instantiate(artboard)?;
    let mut events = Vec::new();
    scene.frame().advance(instance, 0.0, &mut events);

    let second = scene.machine_input(instance, second_machine, "Second go")?;
    let first = scene.machine_input(instance, first_machine, "First go")?;
    scene.frame().fire(second)?;
    scene.frame().fire(first)?;
    scene.frame().advance(instance, 0.0, &mut events);
    assert_eq!(
        events
            .iter()
            .map(|event| match event {
                SceneEvent::Authored { event, .. } => *event,
                _ => unreachable!("bounded runtime event family"),
            })
            .collect::<Vec<_>>(),
        vec![first_event, second_event],
        "event order follows authored runtime machine order, not trigger call order"
    );
    scene.frame().advance(instance, 0.0, &mut events);
    assert!(events.is_empty());
    Ok(())
}

#[test]
fn factory_advance_executes_retained_machine_and_reports_semantic_event() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, shape, machine, _, event), _) = scene.edit(create_authored_trigger_machine)?;
    let instance = scene.instantiate(artboard)?;
    let opacity = scene.cursor(instance, shape, props::WORLD_OPACITY)?;
    let go = scene.machine_input(instance, machine, "Go")?;
    let mut events = Vec::new();
    let mut factory = RecordingFactory::new();

    assert!(
        scene
            .frame()
            .try_advance_with_factory(instance, 0.0, &mut events, &mut factory,)?
    );
    assert_eq!(scene.frame().get(opacity)?, 0.2);
    assert!(events.is_empty());

    scene.frame().fire(go)?;
    assert!(
        scene
            .frame()
            .try_advance_with_factory(instance, 0.0, &mut events, &mut factory,)?
    );
    assert_eq!(scene.frame().get(opacity)?, 0.8);
    assert_eq!(
        events,
        vec![SceneEvent::Authored {
            event,
            name: Some("Reached active".into()),
            string_properties: Vec::new(),
            seconds_delay: 0.0,
            context: None,
        }]
    );

    scene
        .frame()
        .try_advance_with_factory(instance, 0.0, &mut events, &mut factory)?;
    assert!(
        events.is_empty(),
        "events are scoped to one factory advance"
    );
    Ok(())
}

#[test]
fn machine_input_resolution_and_fire_are_fenced_by_artboard_epoch_and_instance() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, _, machine, _, _), _) = scene.edit(create_authored_trigger_machine)?;
    let (other_artboard, _) = scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Other".into(),
            width: 10.0,
            height: 10.0,
        })
    })?;
    let instance = scene.instantiate(artboard)?;
    let other_instance = scene.instantiate(other_artboard)?;

    assert!(matches!(
        scene.machine_input(instance, machine, "missing"),
        Err(ResolveError::UnknownMachineInput)
    ));
    assert!(matches!(
        scene.machine_input(other_instance, machine, "Go"),
        Err(ResolveError::DifferentArtboard)
    ));

    let stale_after_edit = scene.machine_input(instance, machine, "Go")?;
    scene.edit(|tx| {
        tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Structural edit".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        Ok(())
    })?;
    assert_eq!(scene.frame().fire(stale_after_edit), Err(StaleCursor));

    let stale_after_drop = scene.machine_input(instance, machine, "Go")?;
    scene.drop_instance(instance);
    assert_eq!(scene.frame().fire(stale_after_drop), Err(StaleCursor));
    assert!(matches!(
        scene.machine_input(instance, machine, "Go"),
        Err(ResolveError::UnknownInstance)
    ));

    let replacement = scene.instantiate(artboard)?;
    scene.edit(|tx| tx.remove(machine.object_id()).map(|_| ()))?;
    assert!(matches!(
        scene.machine_input(replacement, machine, "Go"),
        Err(ResolveError::UnknownMachine)
    ));
    Ok(())
}

#[test]
fn machine_authoring_rejects_cross_machine_and_cross_artboard_references_atomically() -> Result<()>
{
    let mut scene = Scene::new();
    let ((artboard, _, _, _, event), _) = scene.edit(create_authored_trigger_machine)?;
    let (other_artboard, _) = scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Other".into(),
            width: 10.0,
            height: 10.0,
        })
    })?;

    let epoch_before = scene.epoch();
    let records_before = scene.export_records();
    let cross_machine = scene
        .edit(|tx| {
            let mut machines = tx.machines();
            let input_machine = machines.create_machine(
                artboard,
                MachineSpec {
                    name: Some("Inputs".into()),
                },
            )?;
            let foreign = machines.create_trigger_input(
                input_machine,
                TriggerInputSpec {
                    name: "Foreign".into(),
                },
            )?;
            let transition_machine = machines.create_machine(
                artboard,
                MachineSpec {
                    name: Some("Transitions".into()),
                },
            )?;
            let layer = machines.create_layer(
                transition_machine,
                MachineLayerSpec {
                    name: Some("Layer".into()),
                },
            )?;
            machines.create_any_state(layer)?;
            let entry = machines.create_entry_state(layer)?;
            let exit = machines.create_exit_state(layer)?;
            let transition = machines.create_transition(entry, exit)?;
            machines.add_trigger_condition(transition, foreign)?;
            Ok(())
        })
        .expect_err("a transition cannot consume another machine's input");
    assert_eq!(cross_machine.kind(), EditErrorKind::Aborted);
    assert_eq!(
        cross_machine.diagnostic().reason,
        EditReason::InvalidMachineReference
    );
    assert_eq!(scene.epoch(), epoch_before);
    assert_eq!(scene.export_records(), records_before);

    let cross_artboard = scene
        .edit(|tx| {
            let mut machines = tx.machines();
            let machine = machines.create_machine(
                other_artboard,
                MachineSpec {
                    name: Some("Other machine".into()),
                },
            )?;
            let layer = machines.create_layer(
                machine,
                MachineLayerSpec {
                    name: Some("Other layer".into()),
                },
            )?;
            let entry = machines.create_entry_state(layer)?;
            machines.add_fire_event(entry, event, FireEventOccurs::AtStart)?;
            Ok(())
        })
        .expect_err("a state cannot fire an event owned by another artboard");
    assert_eq!(cross_artboard.kind(), EditErrorKind::Aborted);
    assert_eq!(
        cross_artboard.diagnostic().reason,
        EditReason::CrossArtboardReference {
            source: other_artboard,
            target: artboard,
        }
    );
    assert_eq!(scene.epoch(), epoch_before);
    assert_eq!(scene.export_records(), records_before);

    let transition_cross_artboard = scene
        .edit(|tx| {
            let mut machines = tx.machines();
            let machine = machines.create_machine(
                other_artboard,
                MachineSpec {
                    name: Some("Transition event owner".into()),
                },
            )?;
            let layer = machines.create_layer(machine, MachineLayerSpec { name: None })?;
            machines.create_any_state(layer)?;
            let entry = machines.create_entry_state(layer)?;
            let exit = machines.create_exit_state(layer)?;
            let transition = machines.create_transition(entry, exit)?;
            machines.add_transition_fire_event(transition, event, FireEventOccurs::AtStart)?;
            Ok(())
        })
        .expect_err("a transition cannot fire an event owned by another artboard");
    assert_eq!(transition_cross_artboard.kind(), EditErrorKind::Aborted);
    assert_eq!(
        transition_cross_artboard.diagnostic().reason,
        EditReason::CrossArtboardReference {
            source: other_artboard,
            target: artboard,
        }
    );
    assert_eq!(scene.epoch(), epoch_before);
    assert_eq!(scene.export_records(), records_before);
    Ok(())
}

#[test]
fn machine_topology_rejects_missing_or_duplicate_required_states_atomically() -> Result<()> {
    let mut scene = Scene::new();
    let ((machine, layer, entry), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Canvas".into(),
            width: 10.0,
            height: 10.0,
        })?;
        let mut machines = tx.machines();
        let machine = machines.create_machine(artboard, MachineSpec { name: None })?;
        let layer = machines.create_layer(machine, MachineLayerSpec { name: None })?;
        let entry = machines.create_entry_state(layer)?;
        machines.create_any_state(layer)?;
        machines.create_exit_state(layer)?;
        Ok((machine, layer, entry))
    })?;
    let epoch = scene.epoch();
    let records = scene.export_records();

    let missing_entry = scene
        .edit(|tx| tx.remove(entry.object_id()).map(|_| ()))
        .expect_err("removing the only Entry state must reject at commit");
    assert_eq!(missing_entry.kind(), EditErrorKind::CommitRejected);
    assert_eq!(
        missing_entry.diagnostic().reason,
        EditReason::InvalidMachineTopology {
            requirement: "exactly one entry state per layer",
            actual: 0,
        }
    );
    assert_eq!(scene.epoch(), epoch);
    assert_eq!(scene.export_records(), records);

    let duplicate_entry = scene
        .edit(|tx| tx.machines().create_entry_state(layer).map(|_| ()))
        .expect_err("a second Entry state must reject at commit");
    assert_eq!(duplicate_entry.kind(), EditErrorKind::CommitRejected);
    assert_eq!(
        duplicate_entry.diagnostic().reason,
        EditReason::InvalidMachineTopology {
            requirement: "exactly one entry state per layer",
            actual: 2,
        }
    );
    assert_eq!(scene.epoch(), epoch);
    assert_eq!(scene.export_records(), records);

    let missing_layer = scene
        .edit(|tx| tx.remove(layer.object_id()).map(|_| ()))
        .expect_err("removing the only layer must reject at commit");
    assert_eq!(missing_layer.kind(), EditErrorKind::CommitRejected);
    assert_eq!(
        missing_layer.diagnostic().reason,
        EditReason::InvalidMachineTopology {
            requirement: "at least one state-machine layer",
            actual: 0,
        }
    );
    assert_eq!(scene.epoch(), epoch);
    assert_eq!(scene.export_records(), records);
    assert!(
        scene
            .export_records()
            .records()
            .iter()
            .any(|record| record.kind == ExportedObjectKind::StateMachine)
    );
    let _ = machine;
    Ok(())
}

#[test]
fn machine_names_match_semantic_lowering_whitespace_rules() -> Result<()> {
    let mut scene = Scene::new();
    let (machine, _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Canvas".into(),
            width: 10.0,
            height: 10.0,
        })?;
        let mut machines = tx.machines();
        let event = machines.create_event(
            artboard,
            EventSpec {
                name: Some("   ".into()),
            },
        )?;
        let machine = machines.create_machine(
            artboard,
            MachineSpec {
                name: Some("\t".into()),
            },
        )?;
        machines.create_trigger_input(machine, TriggerInputSpec { name: "Go".into() })?;
        let layer = machines.create_layer(
            machine,
            MachineLayerSpec {
                name: Some("\n".into()),
            },
        )?;
        machines.create_entry_state(layer)?;
        machines.create_any_state(layer)?;
        machines.create_exit_state(layer)?;
        let _ = event;
        Ok(machine)
    })?;

    let normalized_records = scene.export_records();
    for kind in [
        ExportedObjectKind::Event,
        ExportedObjectKind::StateMachine,
        ExportedObjectKind::StateMachineLayer,
    ] {
        let record = normalized_records
            .records()
            .iter()
            .find(|record| record.kind == kind)
            .expect("normalized machine-family record");
        assert!(
            record.properties.iter().all(|property| !matches!(
                property,
                ExportedProperty::AnimationName(_)
                    | ExportedProperty::StateMachineComponentName(_)
                    | ExportedProperty::ComponentName(_)
            )),
            "whitespace-only optional names normalize to an omitted property"
        );
    }

    let records = scene.export_records();
    let epoch = scene.epoch();
    let error = scene
        .edit(|tx| {
            tx.machines()
                .create_trigger_input(machine, TriggerInputSpec { name: "   ".into() })?;
            Ok(())
        })
        .expect_err("whitespace-only trigger names cannot be runtime resolution keys");
    assert_eq!(error.kind(), EditErrorKind::Aborted);
    assert_eq!(error.diagnostic().reason, EditReason::EmptyMachineInputName);
    assert_eq!(scene.epoch(), epoch);
    assert_eq!(scene.export_records(), records);
    Ok(())
}

#[test]
fn dangling_machine_reference_rejects_at_commit_without_staling_live_input() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, _, machine, _, event), _) = scene.edit(create_authored_trigger_machine)?;
    let instance = scene.instantiate(artboard)?;
    let go = scene.machine_input(instance, machine, "Go")?;
    let epoch_before = scene.epoch();
    let records_before = scene.export_records();

    let error = scene
        .edit(|tx| tx.remove(event.object_id()).map(|_| ()))
        .expect_err("removing a referenced authored event must reject atomically");
    assert_eq!(error.kind(), EditErrorKind::CommitRejected);
    assert_eq!(error.diagnostic().reason, EditReason::UnknownObject);
    assert_eq!(scene.epoch(), epoch_before);
    assert_eq!(scene.export_records(), records_before);

    let mut events = Vec::new();
    scene.frame().advance(instance, 0.0, &mut events);
    scene.frame().fire(go)?;
    scene.frame().advance(instance, 0.0, &mut events);
    assert_eq!(
        events.len(),
        1,
        "failed edits leave retained machines usable"
    );
    Ok(())
}

#[test]
fn duplicate_machine_input_name_rejects_without_ambiguous_resolution() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, _, machine, _, _), _) = scene.edit(create_authored_trigger_machine)?;
    let instance = scene.instantiate(artboard)?;
    let (spare, _) = scene.edit(|tx| {
        tx.machines().create_trigger_input(
            machine,
            TriggerInputSpec {
                name: "Spare".into(),
            },
        )
    })?;
    let epoch_before = scene.epoch();
    let records_before = scene.export_records();

    let error = scene
        .edit(|tx| {
            tx.machines()
                .create_trigger_input(machine, TriggerInputSpec { name: "Go".into() })?;
            Ok(())
        })
        .expect_err("machine input names are the runtime resolution key");
    assert_eq!(error.kind(), EditErrorKind::Aborted);
    assert_eq!(
        error.diagnostic().reason,
        EditReason::DuplicateMachineInputName
    );
    assert_eq!(scene.epoch(), epoch_before);
    assert_eq!(scene.export_records(), records_before);
    assert!(scene.machine_input(instance, machine, "Go").is_ok());

    let (removed, _) = scene.edit(|tx| tx.remove(spare.object_id()))?;
    scene.edit(|tx| {
        tx.machines().create_trigger_input(
            machine,
            TriggerInputSpec {
                name: "Spare".into(),
            },
        )?;
        Ok(())
    })?;
    let epoch_before_restore = scene.epoch();
    let records_before_restore = scene.export_records();
    let error = scene
        .edit(|tx| tx.restore(removed).map(|_| ()))
        .expect_err("restoring an input cannot bypass the uniqueness invariant");
    assert_eq!(error.kind(), EditErrorKind::CommitRejected);
    assert_eq!(
        error.diagnostic().reason,
        EditReason::DuplicateMachineInputName
    );
    assert_eq!(scene.epoch(), epoch_before_restore);
    assert_eq!(scene.export_records(), records_before_restore);
    assert!(scene.machine_input(instance, machine, "Spare").is_ok());
    Ok(())
}

#[test]
fn machine_remove_restore_preserves_uniform_record_identity_and_rebuilds_live_state() -> Result<()>
{
    let mut scene = Scene::new();
    let ((artboard, _, machine, _, _), _) = scene.edit(create_authored_trigger_machine)?;
    let records_before = scene.export_records();
    let instance = scene.instantiate(artboard)?;
    let old_input = scene.machine_input(instance, machine, "Go")?;

    let (removed, _) = scene.edit(|tx| tx.remove(machine.object_id()))?;
    assert_eq!(scene.frame().fire(old_input), Err(StaleCursor));
    assert!(matches!(
        scene.machine_input(instance, machine, "Go"),
        Err(ResolveError::UnknownMachine)
    ));
    assert!(
        !scene
            .export_records()
            .records()
            .iter()
            .any(|record| record.kind == ExportedObjectKind::StateMachine)
    );

    let (restored, _) = scene.edit(|tx| tx.restore(removed))?;
    assert_eq!(restored, machine.object_id());
    assert_eq!(scene.export_records(), records_before);
    let refreshed = scene.machine_input(instance, machine, "Go")?;
    assert!(scene.frame().fire(refreshed).is_ok());
    Ok(())
}

#[test]
fn linear_animation_remove_restore_preserves_object_identity_and_stales_absent_handles()
-> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, shape, animation), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Canvas".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Fader".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let animation = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Fade".into(),
                fps: 60,
                duration: 60,
            },
        )?;
        tx.animations()
            .set_key(animation, shape, props::WORLD_OPACITY, 0, 0.0)?;
        tx.animations()
            .set_key(animation, shape, props::WORLD_OPACITY, 60, 1.0)?;
        Ok((artboard, shape, animation))
    })?;
    let records_before = scene.export_records();
    let instance = scene.instantiate(artboard)?;
    let cursor_before_remove = scene.cursor(instance, shape, props::WORLD_OPACITY)?;

    let (removed, _) = scene.edit(|tx| tx.remove(animation.object_id()))?;
    assert!(
        !scene
            .export_records()
            .records()
            .iter()
            .any(|record| record.kind == ExportedObjectKind::LinearAnimation)
    );
    assert_eq!(
        scene.frame().scrub(instance, animation, 0.5),
        Err(StaleCursor),
        "the typed handle is stale while its ordinary ObjectId is absent"
    );
    assert_eq!(scene.frame().get(cursor_before_remove), Err(StaleCursor));

    let (restored, _) = scene.edit(|tx| tx.restore(removed))?;
    assert_eq!(restored, animation.object_id());
    assert_eq!(scene.export_records(), records_before);
    let cursor_after_restore = scene.cursor(instance, shape, props::WORLD_OPACITY)?;
    scene.frame().scrub(instance, animation, 0.5)?;
    assert_eq!(scene.frame().get(cursor_after_restore)?, 0.5);
    Ok(())
}

#[test]
fn generic_set_mutates_animation_and_keyframe_records_through_typed_props() -> Result<()> {
    assert_eq!(props::ANIMATION_FPS.value_kind(), PropValueKind::Uint);
    assert_eq!(props::ANIMATION_FPS.declared_owner(), "LinearAnimation");
    assert_eq!(props::ANIMATION_DURATION.value_kind(), PropValueKind::Uint);
    assert_eq!(
        props::KEY_FRAME_DOUBLE_VALUE.value_kind(),
        PropValueKind::Double
    );
    assert_eq!(
        props::KEY_FRAME_DOUBLE_VALUE.declared_owner(),
        "KeyFrameDouble"
    );
    assert!(!props::ANIMATION_DURATION.is_available_on(NodeKind::Shape));

    let mut scene = Scene::new();
    let ((animation, key), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Canvas".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Fader".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let animation = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Fade".into(),
                fps: 60,
                duration: 60,
            },
        )?;
        let key = tx
            .animations()
            .set_key(animation, shape, props::WORLD_OPACITY, 60, 1.0)?;
        Ok((animation, key))
    })?;

    scene.edit(|tx| {
        tx.set(animation.object_id(), props::ANIMATION_FPS, 30)?;
        tx.set(animation.object_id(), props::ANIMATION_DURATION, 120)?;
        tx.set(key, props::KEY_FRAME_DOUBLE_VALUE, 0.25)
    })?;

    let records = scene.export_records();
    let animation_record = records
        .records()
        .iter()
        .find(|record| record.kind == ExportedObjectKind::LinearAnimation)
        .expect("linear animation record");
    assert!(
        animation_record
            .properties
            .contains(&ExportedProperty::AnimationFps(30))
    );
    assert!(
        animation_record
            .properties
            .contains(&ExportedProperty::AnimationDuration(120))
    );
    let key_record = records
        .records()
        .iter()
        .find(|record| record.kind == ExportedObjectKind::KeyFrameDouble)
        .expect("keyframe record");
    assert!(
        key_record
            .properties
            .contains(&ExportedProperty::KeyFrameDoubleValue(0.25))
    );
    Ok(())
}

#[test]
fn animation_targeting_another_artboard_reports_precise_cross_artboard_reason() -> Result<()> {
    let mut scene = Scene::new();
    let ((source_artboard, target_artboard, foreign_shape, animation), _) = scene.edit(|tx| {
        let source_artboard = tx.create_artboard(ArtboardSpec {
            name: "Source".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let target_artboard = tx.create_artboard(ArtboardSpec {
            name: "Target".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let foreign_shape = tx.create(
            Parent::Artboard(target_artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Foreign".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let animation = tx.animations().create_linear(
            source_artboard,
            LinearAnimationSpec {
                name: "Fade".into(),
                fps: 60,
                duration: 60,
            },
        )?;
        Ok((source_artboard, target_artboard, foreign_shape, animation))
    })?;

    let error = scene
        .edit(|tx| {
            tx.animations()
                .set_key(animation, foreign_shape, props::WORLD_OPACITY, 0, 0.0)?;
            Ok(())
        })
        .expect_err("cross-artboard animation target must reject");
    assert_eq!(
        error.diagnostic().reason,
        EditReason::CrossArtboardReference {
            source: source_artboard,
            target: target_artboard,
        }
    );
    Ok(())
}

#[test]
fn moving_an_animated_visual_to_another_artboard_rejects_atomically() -> Result<()> {
    let mut scene = Scene::new();
    let ((source_artboard, target_artboard, shape), _) = scene.edit(|tx| {
        let source_artboard = tx.create_artboard(ArtboardSpec {
            name: "Source".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let target_artboard = tx.create_artboard(ArtboardSpec {
            name: "Target".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(source_artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Animated".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let animation = tx.animations().create_linear(
            source_artboard,
            LinearAnimationSpec {
                name: "Fade".into(),
                fps: 60,
                duration: 60,
            },
        )?;
        tx.animations()
            .set_key(animation, shape, props::WORLD_OPACITY, 0, 0.0)?;
        Ok((source_artboard, target_artboard, shape))
    })?;
    let before = scene.export_records();
    let epoch = scene.epoch();

    let error = scene
        .edit(|tx| tx.reparent(shape, Parent::Artboard(target_artboard), ChildIndex::Last))
        .expect_err("an animated target cannot move away from its timeline owner");

    assert_eq!(
        error.diagnostic().reason,
        EditReason::CrossArtboardReference {
            source: source_artboard,
            target: target_artboard,
        }
    );
    assert_eq!(scene.epoch(), epoch);
    assert_eq!(scene.export_records(), before);
    Ok(())
}

#[test]
fn cursor_reports_nonvisual_animation_identity_explicitly() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, animation), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Canvas".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let animation = tx.animations().create_linear(
            artboard,
            LinearAnimationSpec {
                name: "Fade".into(),
                fps: 60,
                duration: 60,
            },
        )?;
        Ok((artboard, animation))
    })?;
    let instance = scene.instantiate(artboard)?;
    assert!(matches!(
        scene.cursor(instance, animation.object_id(), props::WORLD_OPACITY),
        Err(ResolveError::NonVisualObject)
    ));
    Ok(())
}

fn create_card(
    tx: &mut SceneTx<'_>,
    name: &str,
    color: u32,
) -> std::result::Result<(ArtboardId, ObjectId, ObjectId), EditAbort> {
    let artboard = tx.create_artboard(ArtboardSpec {
        name: name.into(),
        width: 100.0,
        height: 100.0,
    })?;
    let shape = tx.create(
        Parent::Artboard(artboard),
        NodeSpec::Shape(ShapeSpec {
            name: format!("{name} Card"),
            x: 50.0,
            y: 50.0,
            opacity: 1.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        }),
    )?;
    let rectangle = tx.create(
        Parent::Object(shape),
        NodeSpec::Rectangle(RectangleSpec::new(format!("{name} Rectangle"), 80.0, 60.0)),
    )?;
    let fill = tx.create(
        Parent::Object(shape),
        NodeSpec::Fill(FillSpec {
            name: format!("{name} Fill"),
        }),
    )?;
    let color = tx.create(
        Parent::Object(fill),
        NodeSpec::SolidColor(SolidColorSpec {
            name: format!("{name} Color"),
            color,
        }),
    )?;
    Ok((artboard, rectangle, color))
}

fn create_hit_shape(
    tx: &mut SceneTx<'_>,
    artboard: ArtboardId,
    name: &str,
    x: f32,
    with_stroke: bool,
) -> std::result::Result<ObjectId, EditAbort> {
    let shape = create_colored_hit_shape(tx, artboard, name, x, 0xff11_2233)?;
    if with_stroke {
        let stroke = tx.create(
            Parent::Object(shape),
            NodeSpec::Stroke(StrokeSpec {
                name: format!("{name} Stroke"),
                thickness: 2.0,
                cap: SceneStrokeCap::Butt,
                join: SceneStrokeJoin::Miter,
                transform_affects_stroke: true,
            }),
        )?;
        tx.create(
            Parent::Object(stroke),
            NodeSpec::SolidColor(SolidColorSpec {
                name: format!("{name} Stroke Color"),
                color: 0xff44_5566,
            }),
        )?;
    }
    Ok(shape)
}

fn create_colored_hit_shape(
    tx: &mut SceneTx<'_>,
    artboard: ArtboardId,
    name: &str,
    x: f32,
    color: u32,
) -> std::result::Result<ObjectId, EditAbort> {
    let shape = tx.create(
        Parent::Artboard(artboard),
        NodeSpec::Shape(ShapeSpec {
            name: name.into(),
            x,
            y: 50.0,
            opacity: 1.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        }),
    )?;
    tx.create(
        Parent::Object(shape),
        NodeSpec::Rectangle(RectangleSpec::new(format!("{name} Rectangle"), 40.0, 40.0)),
    )?;
    let fill = tx.create(
        Parent::Object(shape),
        NodeSpec::Fill(FillSpec {
            name: format!("{name} Fill"),
        }),
    )?;
    tx.create(
        Parent::Object(fill),
        NodeSpec::SolidColor(SolidColorSpec {
            name: format!("{name} Fill Color"),
            color,
        }),
    )?;
    Ok(shape)
}

fn exported_component_names(scene: &Scene) -> Vec<String> {
    scene
        .export_records()
        .records()
        .iter()
        .filter_map(|record| {
            record
                .properties
                .iter()
                .find_map(|property| match property {
                    ExportedProperty::ComponentName(name) => Some(name.clone()),
                    _ => None,
                })
        })
        .collect()
}

#[test]
fn nested_artboard_hit_paths_preserve_child_front_to_back_order() -> Result<()> {
    let mut scene = Scene::new();
    let ((root_artboard, child_artboard, host, front, back), _) = scene.edit(|tx| {
        let root_artboard = tx.create_artboard(ArtboardSpec {
            name: "Root".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let child_artboard = tx.create_artboard(ArtboardSpec {
            name: "Child".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let front = create_colored_hit_shape(tx, child_artboard, "Front", 50.0, 0xff11_2233)?;
        let back = create_colored_hit_shape(tx, child_artboard, "Back", 50.0, 0xff44_5566)?;
        let host = tx.create(
            Parent::Artboard(root_artboard),
            NodeSpec::NestedArtboard(NestedArtboardSpec {
                name: "Child Host".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                artboard: child_artboard,
                view_model_source: None,
                view_model_instance: None,
                state_machines: Vec::new(),
                state_machine_inputs: Vec::new(),
            }),
        )?;
        Ok((root_artboard, child_artboard, host, front, back))
    })?;
    let child_instance = scene.instantiate(child_artboard)?;
    let root_instance = scene.instantiate(root_artboard)?;
    let point = Vec2D::new(50.0, 50.0);

    let child_hits = scene
        .frame()
        .hit_test_paths(child_instance, point)
        .into_iter()
        .map(|path| path.objects().to_vec())
        .collect::<Vec<_>>();
    assert_eq!(
        child_hits,
        vec![vec![front], vec![back]],
        "the child artboard establishes the canonical front-to-back order",
    );
    let nested_hits = scene
        .frame()
        .hit_test_paths(root_instance, point)
        .into_iter()
        .map(|path| path.objects().to_vec())
        .collect::<Vec<_>>();
    assert_eq!(
        nested_hits,
        vec![vec![host, front], vec![host, back]],
        "prefixing a nested-artboard host must not reverse its child hit order",
    );
    Ok(())
}

#[test]
fn component_list_hit_paths_preserve_child_front_to_back_order() -> Result<()> {
    let mut scene = Scene::new();
    let ((root_artboard, item_artboard, host, front, back), _) = scene.edit(|tx| {
        let root_artboard = tx.create_artboard(ArtboardSpec {
            name: "Root".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let item_artboard = tx.create_artboard(ArtboardSpec {
            name: "Item".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let front = create_colored_hit_shape(tx, item_artboard, "Front", 50.0, 0xff11_2233)?;
        let back = create_colored_hit_shape(tx, item_artboard, "Back", 50.0, 0xff44_5566)?;
        let (item_model, items) = {
            let mut view_models = tx.view_models();
            let root_model = view_models.create(ViewModelSpec {
                scope: ViewModelScope::Local,
                name: "Root model".into(),
            })?;
            let item_model = view_models.create(ViewModelSpec {
                scope: ViewModelScope::Local,
                name: "Item model".into(),
            })?;
            let items = view_models.create_list(
                root_model,
                ViewModelListSpec {
                    name: "items".into(),
                },
            )?;
            let root_defaults = view_models.create_instance(
                root_model,
                ViewModelInstanceSpec {
                    name: Some("Root defaults".into()),
                },
            )?;
            let item = view_models.create_instance(
                item_model,
                ViewModelInstanceSpec {
                    name: Some("Item".into()),
                },
            )?;
            view_models.set_list_items(root_defaults, items, &[item])?;
            view_models.set_artboard_default(root_artboard, root_defaults)?;
            view_models.set_artboard_default(item_artboard, item)?;
            (item_model, items)
        };
        let host = tx.create_component_list(
            root_artboard,
            ArtboardComponentListSpec {
                name: "Items".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                flow: None,
                source: ViewModelListSource::direct(items),
                map_rules: vec![ArtboardListMapRuleSpec {
                    view_model: item_model,
                    artboard: item_artboard,
                    state_machines: Vec::new(),
                }],
            },
        )?;
        Ok((root_artboard, item_artboard, host, front, back))
    })?;
    let item_instance = scene.instantiate(item_artboard)?;
    let root_instance = scene.instantiate(root_artboard)?;
    let mut events = Vec::new();
    scene.frame().advance(root_instance, 0.0, &mut events);
    let point = Vec2D::new(50.0, 50.0);

    let item_hits = scene
        .frame()
        .hit_test_paths(item_instance, point)
        .into_iter()
        .map(|path| path.objects().to_vec())
        .collect::<Vec<_>>();
    assert_eq!(
        item_hits,
        vec![vec![front], vec![back]],
        "the item artboard establishes the canonical front-to-back order",
    );
    let list_hits = scene
        .frame()
        .hit_test_paths(root_instance, point)
        .into_iter()
        .map(|path| path.objects().to_vec())
        .collect::<Vec<_>>();
    assert_eq!(
        list_hits,
        vec![vec![host.object_id(), front], vec![host.object_id(), back]],
        "prefixing a component-list host must not reverse its item hit order",
    );
    Ok(())
}

#[test]
fn same_parent_reorder_sets_exact_preorder_and_front_to_back_hit_order() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, first, second), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let first = create_colored_hit_shape(tx, artboard, "First", 50.0, 0xff11_2233)?;
        let second = create_colored_hit_shape(tx, artboard, "Second", 50.0, 0xff44_5566)?;
        Ok((artboard, first, second))
    })?;
    let instance = scene.instantiate(artboard)?;

    assert_eq!(
        scene.frame().hit_test(instance, Vec2D::new(50.0, 50.0)),
        vec![first, second]
    );

    scene.edit(|tx| tx.reorder(second, ChildIndex::First))?;

    assert_eq!(
        exported_component_names(&scene),
        vec![
            "Main",
            "Second",
            "Second Rectangle",
            "Second Fill",
            "Second Fill Color",
            "First",
            "First Rectangle",
            "First Fill",
            "First Fill Color",
        ]
    );
    assert_eq!(
        scene.frame().hit_test(instance, Vec2D::new(50.0, 50.0)),
        vec![second, first]
    );

    scene.edit(|tx| tx.reorder(second, ChildIndex::Last))?;
    assert_eq!(
        scene.frame().hit_test(instance, Vec2D::new(50.0, 50.0)),
        vec![first, second]
    );
    scene.edit(|tx| tx.reorder(second, ChildIndex::At(0)))?;
    assert_eq!(
        scene.frame().hit_test(instance, Vec2D::new(50.0, 50.0)),
        vec![second, first]
    );
    Ok(())
}

#[test]
fn same_artboard_reparent_preserves_subtree_ids_parent_ids_preorder_and_fresh_render() -> Result<()>
{
    let mut scene = Scene::new();
    let ((artboard, shape_b, fill_a, color_a), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape_a = create_colored_hit_shape(tx, artboard, "A", 25.0, 0xff11_2233)?;
        let shape_b = create_colored_hit_shape(tx, artboard, "B", 75.0, 0xff44_5566)?;
        let fill_a = tx.create(
            Parent::Object(shape_a),
            NodeSpec::Fill(FillSpec {
                name: "Moved Fill".into(),
            }),
        )?;
        let color_a = tx.create(
            Parent::Object(fill_a),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Moved Color".into(),
                color: 0xff77_8899,
            }),
        )?;
        Ok((artboard, shape_b, fill_a, color_a))
    })?;
    let instance = scene.instantiate(artboard)?;
    let stale_color = scene.cursor(instance, color_a, props::COLOR_VALUE)?;
    let mut held_factory = RecordingFactory::new();
    let mut held_cache = scene.new_draw_token(instance)?;
    let mut held_renderer = held_factory.make_renderer();
    scene.frame().draw(
        instance,
        &mut held_factory,
        &mut held_renderer,
        &mut held_cache,
    )?;

    scene.edit(|tx| tx.reparent(fill_a, Parent::Object(shape_b), ChildIndex::First))?;

    assert_eq!(scene.frame().get(stale_color), Err(StaleCursor));
    let fresh_color = scene.cursor(instance, color_a, props::COLOR_VALUE)?;
    assert_eq!(scene.frame().get(fresh_color)?, 0xff77_8899);
    let records = scene.export_records();
    let named_records = records
        .records()
        .iter()
        .filter_map(|record| {
            let name = record
                .properties
                .iter()
                .find_map(|property| match property {
                    ExportedProperty::ComponentName(name) => Some(name.as_str()),
                    _ => None,
                })?;
            let parent = record
                .properties
                .iter()
                .find_map(|property| match property {
                    ExportedProperty::ParentId(parent) => Some(*parent),
                    _ => None,
                });
            Some((name, parent))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        named_records,
        vec![
            ("Main", None),
            ("A", None),
            ("A Rectangle", Some(1)),
            ("A Fill", Some(1)),
            ("A Fill Color", Some(3)),
            ("B", None),
            ("Moved Fill", Some(5)),
            ("Moved Color", Some(6)),
            ("B Rectangle", Some(5)),
            ("B Fill", Some(5)),
            ("B Fill Color", Some(9)),
        ]
    );

    let mut refreshed_factory = RecordingFactory::new();
    let mut refreshed_renderer = refreshed_factory.make_renderer();
    scene.frame().draw(
        instance,
        &mut refreshed_factory,
        &mut refreshed_renderer,
        &mut held_cache,
    )?;
    assert_eq!(
        refreshed_factory.stream(),
        draw_stream(&mut scene, instance)?
    );
    Ok(())
}

#[test]
fn cross_artboard_reparent_moves_one_stable_subtree_and_refreshes_both_draws() -> Result<()> {
    let mut scene = Scene::new();
    let ((source, target, moved_shape, moved_color, target_shape), _) = scene.edit(|tx| {
        let source = tx.create_artboard(ArtboardSpec {
            name: "Source".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let moved_shape = create_colored_hit_shape(tx, source, "Moved", 50.0, 0xff11_2233)?;
        let moved_color = tx.create(
            Parent::Object(moved_shape),
            NodeSpec::Fill(FillSpec {
                name: "Stable Fill".into(),
            }),
        )?;
        let moved_color = tx.create(
            Parent::Object(moved_color),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Stable Color".into(),
                color: 0xff77_8899,
            }),
        )?;
        let target = tx.create_artboard(ArtboardSpec {
            name: "Target".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let target_shape = create_colored_hit_shape(tx, target, "Existing", 50.0, 0xff44_5566)?;
        Ok((source, target, moved_shape, moved_color, target_shape))
    })?;
    let source_instance = scene.instantiate(source)?;
    let target_instance = scene.instantiate(target)?;
    let source_before = draw_stream(&mut scene, source_instance)?;
    let target_before = draw_stream(&mut scene, target_instance)?;

    scene.edit(|tx| tx.reparent(moved_shape, Parent::Artboard(target), ChildIndex::At(1)))?;

    assert!(matches!(
        scene.cursor(source_instance, moved_color, props::COLOR_VALUE),
        Err(ResolveError::DifferentArtboard)
    ));
    assert!(
        matches!(
            scene.cursor(source_instance, target_shape, props::WORLD_OPACITY),
            Err(ResolveError::DifferentArtboard)
        ),
        "the target object must remain unavailable through the source instance"
    );
    let moved_cursor = scene.cursor(target_instance, moved_color, props::COLOR_VALUE)?;
    assert_eq!(scene.frame().get(moved_cursor)?, 0xff77_8899);
    assert_eq!(
        exported_component_names(&scene),
        vec![
            "Source",
            "Target",
            "Existing",
            "Existing Rectangle",
            "Existing Fill",
            "Existing Fill Color",
            "Moved",
            "Moved Rectangle",
            "Moved Fill",
            "Moved Fill Color",
            "Stable Fill",
            "Stable Color",
        ]
    );
    assert_ne!(draw_stream(&mut scene, source_instance)?, source_before);
    assert_ne!(draw_stream(&mut scene, target_instance)?, target_before);
    assert!(draw_stream(&mut scene, target_instance)?.contains("ff778899"));
    Ok(())
}

#[test]
fn rejected_structural_moves_are_strict_typed_and_preserve_all_live_state() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, shape_a, rectangle_a, fill_a, color_a, shape_b, removed_id), _) =
        scene.edit(|tx| {
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Main".into(),
                width: 100.0,
                height: 100.0,
            })?;
            let shape_a = create_colored_hit_shape(tx, artboard, "A", 25.0, 0xff11_2233)?;
            let rectangle_a = tx.create(
                Parent::Object(shape_a),
                NodeSpec::Rectangle(RectangleSpec::new("Extra Rectangle", 10.0, 10.0)),
            )?;
            let fill_a = tx.create(
                Parent::Object(shape_a),
                NodeSpec::Fill(FillSpec {
                    name: "Extra Fill".into(),
                }),
            )?;
            let color_a = tx.create(
                Parent::Object(fill_a),
                NodeSpec::SolidColor(SolidColorSpec {
                    name: "Extra Color".into(),
                    color: 0xff44_5566,
                }),
            )?;
            let shape_b = create_colored_hit_shape(tx, artboard, "B", 75.0, 0xff77_8899)?;
            let removed = tx.create(
                Parent::Object(shape_b),
                NodeSpec::Rectangle(RectangleSpec::new("Removed", 5.0, 5.0)),
            )?;
            tx.remove(removed)?;
            let removed_id = removed;
            Ok((
                artboard,
                shape_a,
                rectangle_a,
                fill_a,
                color_a,
                shape_b,
                removed_id,
            ))
        })?;
    let instance = scene.instantiate(artboard)?;
    let color_cursor = scene.cursor(instance, color_a, props::COLOR_VALUE)?;
    assert!(scene.frame().set(color_cursor, 0xffaa_bbcc)?);
    let mut factory = RecordingFactory::new();
    let mut cache = scene.new_draw_token(instance)?;
    let mut renderer = factory.make_renderer();
    scene
        .frame()
        .draw(instance, &mut factory, &mut renderer, &mut cache)?;
    factory.clear();
    scene
        .frame()
        .draw(instance, &mut factory, &mut renderer, &mut cache)?;
    let draw_before = factory.stream();
    let epoch_before = scene.epoch();
    let records_before = scene.export_records();

    let cases = [
        scene
            .edit(|tx| tx.reorder(removed_id, ChildIndex::First))
            .expect_err("a removed object is unknown"),
        scene
            .edit(|tx| tx.reparent(shape_a, Parent::Object(rectangle_a), ChildIndex::First))
            .expect_err("a descendant parent creates a cycle before its kind is considered"),
        scene
            .edit(|tx| tx.reparent(rectangle_a, Parent::Object(fill_a), ChildIndex::First))
            .expect_err("a Fill cannot parent a Rectangle"),
        scene
            .edit(|tx| tx.reorder(shape_b, ChildIndex::At(2)))
            .expect_err("two root siblings have final indices zero and one"),
    ];
    assert_eq!(
        cases.map(|error| error.diagnostic().reason.clone()),
        [
            EditReason::UnknownObject,
            EditReason::CycleDetected,
            EditReason::InvalidParent {
                parent: Some(NodeKind::Fill),
                child: NodeKind::Rectangle,
            },
            EditReason::ChildIndexOutOfRange,
        ]
    );
    assert_eq!(scene.epoch(), epoch_before);
    assert_eq!(scene.export_records(), records_before);
    assert_eq!(scene.frame().get(color_cursor)?, 0xffaa_bbcc);
    factory.clear();
    scene
        .frame()
        .draw(instance, &mut factory, &mut renderer, &mut cache)?;
    assert_eq!(factory.stream(), draw_before);
    Ok(())
}

#[test]
fn a_caught_method_abort_leaks_no_partial_hierarchy_into_a_successful_edit() -> Result<()> {
    let mut scene = Scene::new();
    let ((shape_a, fill_a, shape_b, rectangle_b), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape_a = create_colored_hit_shape(tx, artboard, "A", 25.0, 0xff11_2233)?;
        let fill_a = tx.create(
            Parent::Object(shape_a),
            NodeSpec::Fill(FillSpec {
                name: "Must Stay Under A".into(),
            }),
        )?;
        tx.create(
            Parent::Object(fill_a),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Stable Descendant".into(),
                color: 0xff44_5566,
            }),
        )?;
        let shape_b = create_colored_hit_shape(tx, artboard, "B", 75.0, 0xff77_8899)?;
        let rectangle_b = tx.create(
            Parent::Object(shape_b),
            NodeSpec::Rectangle(RectangleSpec::new("Valid Edit", 10.0, 10.0)),
        )?;
        Ok((shape_a, fill_a, shape_b, rectangle_b))
    })?;
    let names_before = exported_component_names(&scene);

    let (caught_reason, _) = scene.edit(|tx| {
        let caught = tx
            .reparent(fill_a, Parent::Object(shape_b), ChildIndex::At(99))
            .expect_err("the target shape has a finite strict child range");
        tx.set(rectangle_b, props::PATH_WIDTH, 42.0)?;
        Ok(caught.diagnostic().reason.clone())
    })?;

    assert_eq!(caught_reason, EditReason::ChildIndexOutOfRange);
    assert_eq!(exported_component_names(&scene), names_before);
    let records = scene.export_records();
    let stable_fill = records
        .records()
        .iter()
        .find(|record| {
            record
                .properties
                .contains(&ExportedProperty::ComponentName("Must Stay Under A".into()))
        })
        .expect("stable fill record");
    assert!(
        stable_fill
            .properties
            .contains(&ExportedProperty::ParentId(1))
    );
    let edited_rectangle = records
        .records()
        .iter()
        .find(|record| {
            record
                .properties
                .contains(&ExportedProperty::ComponentName("Valid Edit".into()))
        })
        .expect("valid edit record");
    assert!(
        edited_rectangle
            .properties
            .contains(&ExportedProperty::PathWidth(42.0))
    );
    let _ = shape_a;
    Ok(())
}

#[test]
fn set_artboard_remounts_only_the_touched_artboard_and_retains_instance_identity() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard_a, _, color_a, artboard_b, _, color_b), _) = scene.edit(|tx| {
        let a = create_card(tx, "A", 0xff11_2233)?;
        let b = create_card(tx, "B", 0xff44_5566)?;
        Ok((a.0, a.1, a.2, b.0, b.1, b.2))
    })?;
    let instance_a = scene.instantiate(artboard_a)?;
    let instance_b = scene.instantiate(artboard_b)?;
    let cursor_a = scene.cursor(instance_a, color_a, props::COLOR_VALUE)?;
    let cursor_b = scene.cursor(instance_b, color_b, props::COLOR_VALUE)?;
    assert!(scene.frame().set(cursor_a, 0xffaa_bbcc)?);
    assert!(scene.frame().set(cursor_b, 0xff77_8899)?);

    let mut factory_a = RecordingFactory::new();
    let mut cache_a = scene.new_draw_token(instance_a)?;
    let mut renderer_a = factory_a.make_renderer();
    scene
        .frame()
        .draw(instance_a, &mut factory_a, &mut renderer_a, &mut cache_a)?;
    let mut factory_b = RecordingFactory::new();
    let mut cache_b = scene.new_draw_token(instance_b)?;
    let mut renderer_b = factory_b.make_renderer();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    factory_b.clear();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    let b_before = factory_b.stream();
    let epoch_before = scene.epoch();

    scene.edit(|tx| {
        tx.set_artboard(
            artboard_a,
            ArtboardSpec {
                name: "A Resized".into(),
                width: 80.0,
                height: 90.0,
            },
        )
    })?;

    assert_eq!(scene.epoch().get(), epoch_before.get() + 1);
    assert_eq!(scene.frame().get(cursor_a), Err(StaleCursor));
    assert_eq!(scene.frame().get(cursor_b), Err(StaleCursor));
    let fresh_a = scene.cursor(instance_a, color_a, props::COLOR_VALUE)?;
    let fresh_b = scene.cursor(instance_b, color_b, props::COLOR_VALUE)?;
    assert_eq!(scene.frame().get(fresh_a)?, 0xff11_2233);
    assert_eq!(scene.frame().get(fresh_b)?, 0xff77_8899);
    assert_eq!(
        exported_component_names(&scene)
            .into_iter()
            .filter(|name| name == "A Resized")
            .count(),
        1
    );

    factory_b.clear();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    assert_eq!(factory_b.stream(), b_before);

    let mut refreshed_factory = RecordingFactory::new();
    let mut refreshed_renderer = refreshed_factory.make_renderer();
    scene.frame().draw(
        instance_a,
        &mut refreshed_factory,
        &mut refreshed_renderer,
        &mut cache_a,
    )?;
    assert_eq!(
        refreshed_factory.stream(),
        draw_stream(&mut scene, instance_a)?
    );
    Ok(())
}

#[test]
fn reorder_artboards_changes_export_order_without_remounting_instances_or_renderers() -> Result<()>
{
    let mut scene = Scene::new();
    let ((artboard_a, _, color_a, artboard_b, _, color_b), _) = scene.edit(|tx| {
        let a = create_card(tx, "A", 0xff11_2233)?;
        let b = create_card(tx, "B", 0xff44_5566)?;
        Ok((a.0, a.1, a.2, b.0, b.1, b.2))
    })?;
    let instance_a = scene.instantiate(artboard_a)?;
    let instance_b = scene.instantiate(artboard_b)?;
    let cursor_a = scene.cursor(instance_a, color_a, props::COLOR_VALUE)?;
    let cursor_b = scene.cursor(instance_b, color_b, props::COLOR_VALUE)?;
    assert!(scene.frame().set(cursor_a, 0xffaa_bbcc)?);
    assert!(scene.frame().set(cursor_b, 0xff77_8899)?);

    let mut factory_a = RecordingFactory::new();
    let mut cache_a = scene.new_draw_token(instance_a)?;
    let mut renderer_a = factory_a.make_renderer();
    scene
        .frame()
        .draw(instance_a, &mut factory_a, &mut renderer_a, &mut cache_a)?;
    factory_a.clear();
    scene
        .frame()
        .draw(instance_a, &mut factory_a, &mut renderer_a, &mut cache_a)?;
    let a_before = factory_a.stream();
    let mut factory_b = RecordingFactory::new();
    let mut cache_b = scene.new_draw_token(instance_b)?;
    let mut renderer_b = factory_b.make_renderer();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    factory_b.clear();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    let b_before = factory_b.stream();
    let epoch_before = scene.epoch();

    scene.edit(|tx| tx.reorder_artboard(artboard_b, ChildIndex::First))?;

    assert_eq!(scene.epoch().get(), epoch_before.get() + 1);
    assert_eq!(scene.frame().get(cursor_a), Err(StaleCursor));
    assert_eq!(scene.frame().get(cursor_b), Err(StaleCursor));
    assert_eq!(
        exported_component_names(&scene),
        vec![
            "B",
            "B Card",
            "B Rectangle",
            "B Fill",
            "B Color",
            "A",
            "A Card",
            "A Rectangle",
            "A Fill",
            "A Color",
        ]
    );
    let fresh_a = scene.cursor(instance_a, color_a, props::COLOR_VALUE)?;
    let fresh_b = scene.cursor(instance_b, color_b, props::COLOR_VALUE)?;
    assert_eq!(scene.frame().get(fresh_a)?, 0xffaa_bbcc);
    assert_eq!(scene.frame().get(fresh_b)?, 0xff77_8899);
    factory_a.clear();
    scene
        .frame()
        .draw(instance_a, &mut factory_a, &mut renderer_a, &mut cache_a)?;
    assert_eq!(factory_a.stream(), a_before);
    factory_b.clear();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    assert_eq!(factory_b.stream(), b_before);
    Ok(())
}

#[test]
fn remove_artboard_drops_only_its_materialization_and_instances() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard_a, _, color_a, artboard_b, _, color_b), _) = scene.edit(|tx| {
        let a = create_card(tx, "A", 0xff11_2233)?;
        let b = create_card(tx, "B", 0xff44_5566)?;
        Ok((a.0, a.1, a.2, b.0, b.1, b.2))
    })?;
    let instance_a = scene.instantiate(artboard_a)?;
    let instance_b = scene.instantiate(artboard_b)?;
    let cursor_a = scene.cursor(instance_a, color_a, props::COLOR_VALUE)?;
    let cursor_b = scene.cursor(instance_b, color_b, props::COLOR_VALUE)?;
    assert!(scene.frame().set(cursor_b, 0xff77_8899)?);

    let mut factory_a = RecordingFactory::new();
    let mut cache_a = scene.new_draw_token(instance_a)?;
    let mut renderer_a = factory_a.make_renderer();
    scene
        .frame()
        .draw(instance_a, &mut factory_a, &mut renderer_a, &mut cache_a)?;
    let mut factory_b = RecordingFactory::new();
    let mut cache_b = scene.new_draw_token(instance_b)?;
    let mut renderer_b = factory_b.make_renderer();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    factory_b.clear();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    let b_before = factory_b.stream();

    scene.edit(|tx| tx.remove_artboard(artboard_a))?;

    assert_eq!(scene.frame().get(cursor_a), Err(StaleCursor));
    assert_eq!(scene.frame().get(cursor_b), Err(StaleCursor));
    assert!(matches!(
        scene.cursor(instance_a, color_a, props::COLOR_VALUE),
        Err(ResolveError::UnknownInstance)
    ));
    assert!(matches!(
        scene.instantiate(artboard_a),
        Err(nuxie::InstanceError::UnknownArtboard)
    ));
    assert!(matches!(
        scene
            .frame()
            .draw(instance_a, &mut factory_a, &mut renderer_a, &mut cache_a),
        Err(nuxie::DrawError::UnknownInstance)
    ));
    assert_eq!(
        exported_component_names(&scene),
        vec!["B", "B Card", "B Rectangle", "B Fill", "B Color",]
    );
    let fresh_b = scene.cursor(instance_b, color_b, props::COLOR_VALUE)?;
    assert_eq!(scene.frame().get(fresh_b)?, 0xff77_8899);
    factory_b.clear();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    assert_eq!(factory_b.stream(), b_before);
    Ok(())
}

#[test]
fn deleting_the_last_artboard_exports_only_backboard_and_allows_recreation() -> Result<()> {
    let mut scene = Scene::new();
    let ((removed_artboard, _, removed_color), _) =
        scene.edit(|tx| create_card(tx, "Only", 0xff11_2233))?;
    let removed_instance = scene.instantiate(removed_artboard)?;
    let removed_cursor = scene.cursor(removed_instance, removed_color, props::COLOR_VALUE)?;

    scene.edit(|tx| tx.remove_artboard(removed_artboard))?;

    assert_eq!(scene.frame().get(removed_cursor), Err(StaleCursor));
    assert_eq!(
        scene
            .export_records()
            .records()
            .iter()
            .map(|record| record.kind)
            .collect::<Vec<_>>(),
        vec![ExportedObjectKind::Backboard]
    );
    assert!(matches!(
        scene.new_draw_token(removed_instance),
        Err(ResolveError::UnknownInstance)
    ));

    let (recreated, _) = scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Recreated".into(),
            width: 120.0,
            height: 80.0,
        })
    })?;
    assert_ne!(recreated, removed_artboard);
    assert!(scene.instantiate(recreated).is_ok());
    assert_eq!(
        scene
            .export_records()
            .records()
            .iter()
            .map(|record| record.kind)
            .collect::<Vec<_>>(),
        vec![ExportedObjectKind::Backboard, ExportedObjectKind::Artboard,]
    );
    Ok(())
}

#[test]
fn removing_one_artboard_with_an_invalid_retained_candidate_publishes_nothing() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard_a, _, color_a, artboard_b, _, color_b), _) = scene.edit(|tx| {
        let a = create_card(tx, "A", 0xff11_2233)?;
        let b = create_card(tx, "B", 0xff44_5566)?;
        Ok((a.0, a.1, a.2, b.0, b.1, b.2))
    })?;
    let instance_a = scene.instantiate(artboard_a)?;
    let instance_b = scene.instantiate(artboard_b)?;
    let cursor_a = scene.cursor(instance_a, color_a, props::COLOR_VALUE)?;
    let cursor_b = scene.cursor(instance_b, color_b, props::COLOR_VALUE)?;
    assert!(scene.frame().set(cursor_a, 0xffaa_bbcc)?);
    assert!(scene.frame().set(cursor_b, 0xff77_8899)?);
    let mut factory_a = RecordingFactory::new();
    let mut cache_a = scene.new_draw_token(instance_a)?;
    let mut renderer_a = factory_a.make_renderer();
    scene
        .frame()
        .draw(instance_a, &mut factory_a, &mut renderer_a, &mut cache_a)?;
    factory_a.clear();
    scene
        .frame()
        .draw(instance_a, &mut factory_a, &mut renderer_a, &mut cache_a)?;
    let a_before = factory_a.stream();
    let mut factory_b = RecordingFactory::new();
    let mut cache_b = scene.new_draw_token(instance_b)?;
    let mut renderer_b = factory_b.make_renderer();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    factory_b.clear();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    let b_before = factory_b.stream();
    let epoch_before = scene.epoch();
    let records_before = scene.export_records();

    let error = scene
        .edit(|tx| {
            tx.remove_artboard(artboard_a)?;
            tx.set_artboard(
                artboard_b,
                ArtboardSpec {
                    name: "Invalid B".into(),
                    width: f32::NAN,
                    height: 100.0,
                },
            )?;
            Ok(())
        })
        .expect_err("all retained candidates must preflight before deletion publishes");

    assert_eq!(error.kind(), EditErrorKind::CommitRejected);
    assert_eq!(error.diagnostic().operation_index, 1);
    assert_eq!(
        error.diagnostic().reason,
        EditReason::NonFiniteProperty { property: "width" }
    );
    assert_eq!(scene.epoch(), epoch_before);
    assert_eq!(scene.export_records(), records_before);
    assert_eq!(scene.frame().get(cursor_a)?, 0xffaa_bbcc);
    assert_eq!(scene.frame().get(cursor_b)?, 0xff77_8899);
    factory_a.clear();
    scene
        .frame()
        .draw(instance_a, &mut factory_a, &mut renderer_a, &mut cache_a)?;
    assert_eq!(factory_a.stream(), a_before);
    factory_b.clear();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    assert_eq!(factory_b.stream(), b_before);
    Ok(())
}

#[test]
fn shape_export_omits_exact_zero_axes_but_retains_nonzero_axes() -> Result<()> {
    let mut scene = Scene::new();
    scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Axes".into(),
            width: 100.0,
            height: 100.0,
        })?;
        tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Zero".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Nonzero".into(),
                x: 12.0,
                y: -7.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        Ok(())
    })?;
    let records = scene.export_records();
    let zero = records
        .records()
        .iter()
        .find(|record| {
            record
                .properties
                .contains(&ExportedProperty::ComponentName("Zero".into()))
        })
        .expect("zero-axis shape");
    assert_eq!(
        zero.properties,
        vec![ExportedProperty::ComponentName("Zero".into())]
    );
    let nonzero = records
        .records()
        .iter()
        .find(|record| {
            record
                .properties
                .contains(&ExportedProperty::ComponentName("Nonzero".into()))
        })
        .expect("nonzero-axis shape");
    assert!(
        nonzero
            .properties
            .contains(&ExportedProperty::TranslateX(12.0))
    );
    assert!(
        nonzero
            .properties
            .contains(&ExportedProperty::TranslateY(-7.0))
    );
    Ok(())
}

#[test]
fn text_export_retains_exact_zero_axes_required_by_publish() -> Result<()> {
    let mut scene = Scene::new();
    scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Text Axes".into(),
            width: 100.0,
            height: 100.0,
        })?;
        tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Text(TextSpec {
                name: "Zero".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                sizing: SceneTextSizing::Fixed,
                width: 80.0,
                height: 30.0,
                align: SceneTextAlign::Left,
                wrap: SceneTextWrap::Wrap,
                overflow: SceneTextOverflow::Visible,
            }),
        )?;
        Ok(())
    })?;

    let exported = scene.export_records();
    let text = exported
        .records()
        .iter()
        .find(|record| record.kind == ExportedObjectKind::Text)
        .expect("zero-axis Text record");
    assert!(text.properties.contains(&ExportedProperty::TranslateX(0.0)));
    assert!(text.properties.contains(&ExportedProperty::TranslateY(0.0)));
    Ok(())
}

#[test]
fn authored_shape_geometry_queries_follow_the_live_runtime_scene() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, shape, rectangle), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Geometry".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Card".into(),
                x: 50.0,
                y: 40.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let rectangle = tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new("Card Rectangle", 20.0, 10.0)),
        )?;
        let fill = tx.create(
            Parent::Object(shape),
            NodeSpec::Fill(FillSpec {
                name: "Card Fill".into(),
            }),
        )?;
        tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Card Color".into(),
                color: 0xff11_2233,
            }),
        )?;
        Ok((artboard, shape, rectangle))
    })?;
    let instance = scene.instantiate(artboard)?;

    {
        let mut frame = scene.frame();
        assert_eq!(
            frame.world_transform(instance, shape),
            Some(nuxie::Mat2D([1.0, 0.0, 0.0, 1.0, 50.0, 40.0]))
        );
        assert_eq!(
            frame.world_bounds(instance, shape),
            Some(Aabb::new(40.0, 35.0, 60.0, 45.0))
        );
        assert_eq!(
            frame.hit_test(instance, Vec2D::new(50.0, 40.0)),
            vec![shape]
        );
    }

    scene.edit(|tx| {
        tx.set(rectangle, props::PATH_WIDTH, 30.0)?;
        Ok(())
    })?;
    assert_eq!(
        scene.frame().world_bounds(instance, shape),
        Some(Aabb::new(35.0, 35.0, 65.0, 45.0)),
        "occurrence-owned geometry must follow the structurally remounted runtime"
    );
    Ok(())
}

#[test]
fn hit_test_is_front_to_back_deduplicated_and_includes_the_path_boundary() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, front, back), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Hit Order".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let front = create_hit_shape(tx, artboard, "Front", 45.0, true)?;
        let back = create_hit_shape(tx, artboard, "Back", 55.0, false)?;
        Ok((artboard, front, back))
    })?;
    let instance = scene.instantiate(artboard)?;
    let mut frame = scene.frame();

    assert_eq!(
        frame.hit_test(instance, Vec2D::new(50.0, 50.0)),
        vec![front, back]
    );
    assert_eq!(
        frame.hit_test(instance, Vec2D::new(24.0, 50.0)),
        vec![front],
        "the outer edge of the front rectangle's two-pixel stroke is inclusive"
    );
    assert!(
        frame.hit_test(instance, Vec2D::new(23.99, 50.0)).is_empty(),
        "hit testing must not add editor selection slop"
    );
    Ok(())
}

#[test]
fn hit_test_respects_the_artboard_clip() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, shape), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Clipped".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = create_hit_shape(tx, artboard, "Off Artboard", 150.0, false)?;
        Ok((artboard, shape))
    })?;
    let instance = scene.instantiate(artboard)?;
    let mut frame = scene.frame();

    assert!(
        frame.hit_test(instance, Vec2D::new(150.0, 50.0)).is_empty(),
        "the visible shape geometry is outside the clipped 100x100 artboard"
    );
    assert_eq!(
        frame.world_bounds(instance, shape),
        Some(Aabb::new(130.0, 30.0, 170.0, 70.0)),
        "clipping affects visibility, not the shape's logical world bounds"
    );
    Ok(())
}

#[test]
fn hit_test_ignores_alpha_zero_paints_but_keeps_visible_siblings() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, transparent, visible), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Paint Visibility".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let transparent = create_colored_hit_shape(tx, artboard, "Transparent", 25.0, 0x0011_2233)?;
        let visible = create_colored_hit_shape(tx, artboard, "Visible", 75.0, 0xff11_2233)?;
        Ok((artboard, transparent, visible))
    })?;
    let instance = scene.instantiate(artboard)?;
    let mut frame = scene.frame();

    assert!(
        frame.hit_test(instance, Vec2D::new(25.0, 50.0)).is_empty(),
        "an alpha-zero fill does not produce a visible hit region"
    );
    assert_eq!(
        frame.hit_test(instance, Vec2D::new(75.0, 50.0)),
        vec![visible]
    );
    assert_ne!(transparent, visible);
    Ok(())
}

#[test]
fn hit_test_uses_the_visible_stroke_region_instead_of_filling_a_stroke_only_shape() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, shape), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Stroke Hit".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Outline".into(),
                x: 50.0,
                y: 50.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new("Outline Rectangle", 40.0, 40.0)),
        )?;
        let stroke = tx.create(
            Parent::Object(shape),
            NodeSpec::Stroke(StrokeSpec {
                name: "Outline Stroke".into(),
                thickness: 4.0,
                cap: SceneStrokeCap::Butt,
                join: SceneStrokeJoin::Miter,
                transform_affects_stroke: true,
            }),
        )?;
        tx.create(
            Parent::Object(stroke),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Outline Color".into(),
                color: 0xff11_2233,
            }),
        )?;
        Ok((artboard, shape))
    })?;
    let instance = scene.instantiate(artboard)?;
    let mut frame = scene.frame();

    assert_eq!(
        frame.hit_test(instance, Vec2D::new(30.0, 50.0)),
        vec![shape],
        "the stroke centerline is visible"
    );
    assert!(
        frame.hit_test(instance, Vec2D::new(50.0, 50.0)).is_empty(),
        "the unfilled interior of a stroke-only shape is not visible"
    );
    assert!(
        frame.hit_test(instance, Vec2D::new(27.99, 50.0)).is_empty(),
        "hit testing adds no selection slop outside the four-pixel stroke"
    );
    Ok(())
}

#[test]
fn geometry_queries_flush_hot_writes_and_reject_wrong_instance_targets() -> Result<()> {
    let mut scene = Scene::new();
    let ((first_artboard, first_shape, second_artboard, second_shape), _) = scene.edit(|tx| {
        let first_artboard = tx.create_artboard(ArtboardSpec {
            name: "First".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let first_shape = create_hit_shape(tx, first_artboard, "First Shape", 20.0, false)?;
        let second_artboard = tx.create_artboard(ArtboardSpec {
            name: "Second".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let second_shape = create_hit_shape(tx, second_artboard, "Second Shape", 50.0, false)?;
        Ok((first_artboard, first_shape, second_artboard, second_shape))
    })?;
    let first_instance = scene.instantiate(first_artboard)?;
    let _second_instance = scene.instantiate(second_artboard)?;
    let translate_x = scene.cursor(first_instance, first_shape, props::TRANSLATE_X)?;
    let opacity = scene.cursor(first_instance, first_shape, props::WORLD_OPACITY)?;

    {
        let mut frame = scene.frame();
        assert!(frame.set(translate_x, 70.0)?);
        assert_eq!(
            frame.world_transform(first_instance, first_shape),
            Some(nuxie::Mat2D([1.0, 0.0, 0.0, 1.0, 70.0, 50.0]))
        );
        assert_eq!(
            frame.world_bounds(first_instance, first_shape),
            Some(Aabb::new(50.0, 30.0, 90.0, 70.0))
        );
        assert_eq!(
            frame.hit_test(first_instance, Vec2D::new(70.0, 50.0)),
            vec![first_shape]
        );
        assert!(
            frame
                .hit_test(first_instance, Vec2D::new(20.0, 50.0))
                .is_empty()
        );
        assert_eq!(
            frame.world_bounds(first_instance, second_shape),
            None,
            "an object from another artboard cannot be queried through this instance"
        );
        assert_eq!(
            frame.world_transform(first_instance, second_shape),
            None,
            "an object from another artboard cannot resolve a transform through this instance"
        );

        assert!(frame.set(opacity, 0.0)?);
        assert!(
            frame
                .hit_test(first_instance, Vec2D::new(70.0, 50.0))
                .is_empty(),
            "a hidden shape must not participate in hit testing"
        );
    }

    let _ = scene.edit(|tx| tx.remove(first_shape))?;
    {
        let mut frame = scene.frame();
        assert_eq!(frame.world_bounds(first_instance, first_shape), None);
        assert_eq!(frame.world_transform(first_instance, first_shape), None);
        assert!(
            frame
                .hit_test(first_instance, Vec2D::new(70.0, 50.0))
                .is_empty()
        );
    }

    scene.drop_instance(first_instance);
    let mut frame = scene.frame();
    assert!(
        frame
            .hit_test(first_instance, Vec2D::new(70.0, 50.0))
            .is_empty()
    );
    assert_eq!(frame.world_bounds(first_instance, first_shape), None);
    assert_eq!(frame.world_transform(first_instance, first_shape), None);
    Ok(())
}

#[test]
fn a_removed_subtree_restores_the_same_objects_records_and_draw_in_a_later_edit() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, shape, rectangle, _fill, color), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Card".into(),
                x: 50.0,
                y: 50.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let rectangle = tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new("Card Rectangle", 80.0, 60.0)),
        )?;
        let fill = tx.create(
            Parent::Object(shape),
            NodeSpec::Fill(FillSpec {
                name: "Card Fill".into(),
            }),
        )?;
        let color = tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Card Color".into(),
                color: 0xff112233,
            }),
        )?;
        Ok((artboard, shape, rectangle, fill, color))
    })?;
    let instance = scene.instantiate(artboard)?;
    let before_records = scene.export_records();
    let before_draw = draw_stream(&mut scene, instance)?;

    let (removed, remove_receipt) = scene.edit(|tx| tx.remove(shape))?;
    assert!(remove_receipt.created.is_empty());
    assert!(matches!(
        scene.cursor(instance, shape, props::WORLD_OPACITY),
        Err(ResolveError::UnknownObject)
    ));
    assert!(matches!(
        scene.cursor(instance, rectangle, props::PATH_WIDTH),
        Err(ResolveError::UnknownObject)
    ));
    assert!(matches!(
        scene.cursor(instance, color, props::COLOR_VALUE),
        Err(ResolveError::UnknownObject)
    ));
    assert_ne!(draw_stream(&mut scene, instance)?, before_draw);

    let (restored_root, restore_receipt) = scene.edit(|tx| tx.restore(removed))?;
    assert_eq!(restored_root, shape);
    assert!(restore_receipt.created.is_empty());
    assert_eq!(scene.export_records(), before_records);
    assert_eq!(draw_stream(&mut scene, instance)?, before_draw);
    assert!(scene.cursor(instance, shape, props::WORLD_OPACITY).is_ok());
    assert!(scene.cursor(instance, rectangle, props::PATH_WIDTH).is_ok());
    assert!(scene.cursor(instance, color, props::COLOR_VALUE).is_ok());
    Ok(())
}

#[test]
fn restore_preserves_parent_before_child_order_after_interleaved_creation() -> Result<()> {
    let mut scene = Scene::new();
    let ((shape_a, _shape_b, _rectangle_a, _rectangle_b), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let mut create_shape = |name: &str, x: f32| {
            tx.create(
                Parent::Artboard(artboard),
                NodeSpec::Shape(ShapeSpec {
                    name: name.into(),
                    x,
                    y: 50.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                }),
            )
        };
        let shape_a = create_shape("A", 25.0)?;
        let shape_b = create_shape("B", 75.0)?;
        let rectangle_a = tx.create(
            Parent::Object(shape_a),
            NodeSpec::Rectangle(RectangleSpec::new("A Rectangle", 20.0, 20.0)),
        )?;
        let rectangle_b = tx.create(
            Parent::Object(shape_b),
            NodeSpec::Rectangle(RectangleSpec::new("B Rectangle", 30.0, 30.0)),
        )?;
        Ok((shape_a, shape_b, rectangle_a, rectangle_b))
    })?;
    let records_before = scene.export_records();
    assert_eq!(
        exported_component_names(&scene),
        vec!["Main", "A", "B", "A Rectangle", "B Rectangle"],
        "valid parent-before-child creation order is retained instead of rewritten as preorder"
    );

    let (removed, _) = scene.edit(|tx| tx.remove(shape_a))?;
    scene.edit(|tx| tx.restore(removed))?;

    assert_eq!(scene.export_records(), records_before);
    Ok(())
}

#[test]
fn restoring_an_existing_identity_aborts_the_entire_edit_with_a_collision_diagnostic() -> Result<()>
{
    let mut scene = Scene::new();
    let ((artboard, rectangle, color), _) = scene.edit(|tx| create_card(tx, "Main", 0xff112233))?;
    let instance = scene.instantiate(artboard)?;
    let (removed, _) = scene.edit(|tx| tx.remove(rectangle))?;
    scene.edit(|tx| tx.restore(removed.clone()))?;
    let color_cursor = scene.cursor(instance, color, props::COLOR_VALUE)?;
    let epoch_before = scene.epoch();
    let records_before = scene.export_records();
    let draw_before = draw_stream(&mut scene, instance)?;

    let error = scene
        .edit(|tx| {
            tx.set(color, props::COLOR_VALUE, 0xff445566)?;
            tx.restore(removed)?;
            Ok(())
        })
        .expect_err("the restored rectangle identity already exists");

    assert_eq!(error.kind(), EditErrorKind::Aborted);
    assert_eq!(error.diagnostic().operation_index, 1);
    assert_eq!(
        error.diagnostic().involved_ids,
        vec![EditId::Object(rectangle)]
    );
    assert_eq!(error.diagnostic().reason, EditReason::IdentityCollision);
    assert_eq!(scene.epoch(), epoch_before);
    assert_eq!(scene.export_records(), records_before);
    assert_eq!(draw_stream(&mut scene, instance)?, draw_before);
    assert_eq!(scene.frame().get(color_cursor)?, 0xff112233);
    Ok(())
}

#[test]
fn remove_then_restore_in_one_edit_is_an_exact_structural_commit_that_stales_all_cursors()
-> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, rectangle, color), _) = scene.edit(|tx| create_card(tx, "Main", 0xff112233))?;
    let instance = scene.instantiate(artboard)?;
    let old_color_cursor = scene.cursor(instance, color, props::COLOR_VALUE)?;
    let epoch_before = scene.epoch();
    let records_before = scene.export_records();
    let draw_before = draw_stream(&mut scene, instance)?;

    let (restored, receipt) = scene.edit(|tx| {
        let removed = tx.remove(rectangle)?;
        tx.restore(removed)
    })?;

    assert_eq!(restored, rectangle);
    assert_eq!(receipt.epoch, scene.epoch());
    assert_eq!(scene.epoch().get(), epoch_before.get() + 1);
    assert!(receipt.created.is_empty());
    assert_eq!(scene.export_records(), records_before);
    assert_eq!(draw_stream(&mut scene, instance)?, draw_before);
    assert_eq!(scene.frame().get(old_color_cursor), Err(StaleCursor));
    let fresh_color_cursor = scene.cursor(instance, color, props::COLOR_VALUE)?;
    assert_eq!(scene.frame().get(fresh_color_cursor)?, 0xff112233);
    Ok(())
}

#[test]
fn edit_receipts_exclude_objects_created_and_removed_before_commit() -> Result<()> {
    let mut scene = Scene::new();
    let (artboard, _) = scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })
    })?;
    let epoch_before = scene.epoch();

    let ((shape, removed), receipt) = scene.edit(|tx| {
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Transient".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let removed = tx.remove(shape)?;
        Ok((shape, removed))
    })?;

    assert!(receipt.created.is_empty());
    assert_eq!(scene.epoch().get(), epoch_before.get() + 1);
    let (restored, restore_receipt) = scene.edit(|tx| tx.restore(removed))?;
    assert_eq!(restored, shape);
    assert!(
        restore_receipt.created.is_empty(),
        "restoring an existing identity never reports a new allocation"
    );
    Ok(())
}

#[test]
fn restore_rejects_a_missing_original_parent_with_structured_diagnostics() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, shape, rectangle), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Card".into(),
                x: 50.0,
                y: 50.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let rectangle = tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new("Card Rectangle", 80.0, 60.0)),
        )?;
        Ok((artboard, shape, rectangle))
    })?;
    let (removed_rectangle, _) = scene.edit(|tx| tx.remove(rectangle))?;
    let (_removed_shape, _) = scene.edit(|tx| tx.remove(shape))?;
    let epoch_before = scene.epoch();
    let records_before = scene.export_records();

    let error = scene
        .edit(|tx| tx.restore(removed_rectangle))
        .expect_err("the rectangle's original shape parent no longer exists");

    assert_eq!(error.kind(), EditErrorKind::Aborted);
    assert_eq!(error.diagnostic().operation_index, 0);
    assert_eq!(
        error.diagnostic().involved_ids,
        vec![EditId::Object(rectangle), EditId::Object(shape)]
    );
    assert_eq!(error.diagnostic().reason, EditReason::UnknownObject);
    assert_eq!(scene.epoch(), epoch_before);
    assert_eq!(scene.export_records(), records_before);
    assert!(scene.instantiate(artboard).is_ok());
    Ok(())
}

#[test]
fn restoring_a_subtree_into_another_scene_rejects_its_owner_and_rolls_back_the_edit() -> Result<()>
{
    let mut source = Scene::new();
    let ((source_artboard, source_rectangle, _), _) =
        source.edit(|tx| create_card(tx, "Source", 0xff112233))?;
    let (removed, _) = source.edit(|tx| tx.remove(source_rectangle))?;

    let mut target = Scene::new();
    let ((target_artboard, _, target_color), _) =
        target.edit(|tx| create_card(tx, "Target", 0xff223344))?;
    let target_instance = target.instantiate(target_artboard)?;
    let target_cursor = target.cursor(target_instance, target_color, props::COLOR_VALUE)?;
    let epoch_before = target.epoch();
    let records_before = target.export_records();
    let draw_before = draw_stream(&mut target, target_instance)?;

    let error = target
        .edit(|tx| {
            tx.set(target_color, props::COLOR_VALUE, 0xff556677)?;
            tx.restore(removed)?;
            Ok(())
        })
        .expect_err("a removed subtree remains owned by its original scene artboard");

    assert_eq!(error.kind(), EditErrorKind::Aborted);
    assert_eq!(error.diagnostic().operation_index, 1);
    assert_eq!(
        error.diagnostic().involved_ids,
        vec![
            EditId::Artboard(source_artboard),
            EditId::Object(source_rectangle)
        ]
    );
    assert_eq!(error.diagnostic().reason, EditReason::UnknownArtboard);
    assert_eq!(target.epoch(), epoch_before);
    assert_eq!(target.export_records(), records_before);
    assert_eq!(draw_stream(&mut target, target_instance)?, draw_before);
    assert_eq!(target.frame().get(target_cursor)?, 0xff223344);
    Ok(())
}

#[test]
fn removing_from_one_artboard_preserves_another_artboards_hot_state_and_renderer() -> Result<()> {
    let mut scene = Scene::new();
    let ((_, rectangle_a, _, artboard_b, _, color_b), _) = scene.edit(|tx| {
        let (artboard_a, rectangle_a, color_a) = create_card(tx, "A", 0xff112233)?;
        let (artboard_b, rectangle_b, color_b) = create_card(tx, "B", 0xff223344)?;
        Ok((
            artboard_a,
            rectangle_a,
            color_a,
            artboard_b,
            rectangle_b,
            color_b,
        ))
    })?;
    let instance_b = scene.instantiate(artboard_b)?;
    let old_cursor_b = scene.cursor(instance_b, color_b, props::COLOR_VALUE)?;
    assert!(scene.frame().set(old_cursor_b, 0xff556677)?);

    let mut factory_b = RecordingFactory::new();
    let mut cache_b = scene.new_draw_token(instance_b)?;
    let mut renderer_b = factory_b.make_renderer();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    factory_b.clear();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    let draw_before = factory_b.stream();
    assert!(draw_before.contains("ff556677"));

    scene.edit(|tx| tx.remove(rectangle_a))?;

    assert_eq!(scene.frame().get(old_cursor_b), Err(StaleCursor));
    let fresh_cursor_b = scene.cursor(instance_b, color_b, props::COLOR_VALUE)?;
    assert_eq!(scene.frame().get(fresh_cursor_b)?, 0xff556677);
    factory_b.clear();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    assert_eq!(
        factory_b.stream(),
        draw_before,
        "the untouched Artboard occurrence must retain its live renderer members"
    );
    Ok(())
}

#[test]
fn authored_scene_uses_typed_cursor_writes_and_stales_them_after_structure_changes() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, shape, rectangle, color), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Card".into(),
                x: 50.0,
                y: 50.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let rectangle = tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new("Card Rectangle", 80.0, 60.0)),
        )?;
        let fill = tx.create(
            Parent::Object(shape),
            NodeSpec::Fill(FillSpec {
                name: "Card Fill".into(),
            }),
        )?;
        let color = tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Card Color".into(),
                color: 0xff112233,
            }),
        )?;
        Ok((artboard, shape, rectangle, color))
    })?;

    let instance = scene.instantiate(artboard)?;
    let color_cursor = scene.cursor(instance, color, props::COLOR_VALUE)?;
    let opacity_cursor = scene.cursor(instance, shape, props::WORLD_OPACITY)?;
    let rotation_cursor = scene.cursor(instance, shape, props::ROTATION)?;
    let before = draw_stream(&mut scene, instance)?;

    assert!(scene.frame().set(opacity_cursor, 0.5)?);
    assert!(scene.frame().set(rotation_cursor, 0.25)?);
    assert!(scene.frame().set(color_cursor, 0xff445566)?);
    let after = draw_stream(&mut scene, instance)?;
    assert_ne!(
        after, before,
        "the cursor write must change rendered output"
    );
    assert!(
        !scene.frame().set(opacity_cursor, f32::NAN)?,
        "invalid hot values are rejected as unchanged"
    );
    assert_eq!(
        draw_stream(&mut scene, instance)?,
        after,
        "a rejected hot write must leave the live graph valid"
    );

    scene.edit(|tx| {
        tx.set(rectangle, props::PATH_WIDTH, 72.0)?;
        Ok(())
    })?;
    assert_eq!(
        scene.frame().set(color_cursor, 0xff778899),
        Err(StaleCursor)
    );
    Ok(())
}

#[test]
fn rectangle_height_cursor_resolves_sets_gets_and_stales_after_structure_changes() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, rectangle, _), _) =
        scene.edit(|tx| create_card(tx, "Height cursor", 0xff112233))?;
    let instance = scene.instantiate(artboard)?;
    let height = scene.cursor(instance, rectangle, props::PATH_HEIGHT)?;

    assert_eq!(scene.frame().get(height)?, 60.0);
    assert!(scene.frame().set(height, 44.0)?);
    assert_eq!(scene.frame().get(height)?, 44.0);

    scene.edit(|tx| tx.set(rectangle, props::PATH_WIDTH, 72.0))?;
    assert_eq!(scene.frame().get(height), Err(StaleCursor));
    assert_eq!(scene.frame().set(height, 48.0), Err(StaleCursor));
    Ok(())
}

#[test]
fn stroke_thickness_cursor_resolves_sets_gets_and_stales_after_structure_changes() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, rectangle, stroke), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Stroke cursor".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Card".into(),
                x: 50.0,
                y: 50.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let rectangle = tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new("Card rectangle", 80.0, 60.0)),
        )?;
        let stroke = tx.create(
            Parent::Object(shape),
            NodeSpec::Stroke(StrokeSpec {
                name: "Card stroke".into(),
                thickness: 2.0,
                cap: SceneStrokeCap::Butt,
                join: SceneStrokeJoin::Miter,
                transform_affects_stroke: true,
            }),
        )?;
        Ok((artboard, rectangle, stroke))
    })?;
    let instance = scene.instantiate(artboard)?;
    let thickness = scene.cursor(instance, stroke, props::STROKE_THICKNESS)?;

    assert_eq!(scene.frame().get(thickness)?, 2.0);
    assert!(scene.frame().set(thickness, 6.0)?);
    assert_eq!(scene.frame().get(thickness)?, 6.0);

    scene.edit(|tx| tx.set(rectangle, props::PATH_WIDTH, 72.0))?;
    assert_eq!(scene.frame().get(thickness), Err(StaleCursor));
    assert_eq!(scene.frame().set(thickness, 8.0), Err(StaleCursor));
    Ok(())
}

#[test]
fn dash_length_cursor_resolves_sets_gets_and_stales_after_structure_changes() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, rectangle, dash), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Dash cursor".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Card".into(),
                x: 50.0,
                y: 50.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let rectangle = tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new("Card rectangle", 80.0, 60.0)),
        )?;
        let stroke = tx.create(
            Parent::Object(shape),
            NodeSpec::Stroke(StrokeSpec {
                name: "Card stroke".into(),
                thickness: 2.0,
                cap: SceneStrokeCap::Butt,
                join: SceneStrokeJoin::Miter,
                transform_affects_stroke: true,
            }),
        )?;
        let dash_path = tx.create(
            Parent::Object(stroke),
            NodeSpec::DashPath(DashPathSpec {
                name: "Dash path".into(),
                offset: 0.0,
                offset_is_percentage: false,
            }),
        )?;
        let dash = tx.create(
            Parent::Object(dash_path),
            NodeSpec::Dash(DashSpec {
                name: "Dash on".into(),
                length: 4.0,
                length_is_percentage: false,
            }),
        )?;
        Ok((artboard, rectangle, dash))
    })?;
    let instance = scene.instantiate(artboard)?;
    let length = scene.cursor(instance, dash, props::DASH_LENGTH)?;

    assert_eq!(scene.frame().get(length)?, 4.0);
    assert!(scene.frame().set(length, 9.0)?);
    assert_eq!(scene.frame().get(length)?, 9.0);

    scene.edit(|tx| tx.set(rectangle, props::PATH_WIDTH, 72.0))?;
    assert_eq!(scene.frame().get(length), Err(StaleCursor));
    assert_eq!(scene.frame().set(length, 11.0), Err(StaleCursor));
    Ok(())
}

#[test]
fn a_cursor_can_never_write_to_another_scene_with_matching_slots_and_epoch() -> Result<()> {
    let mut left = Scene::new();
    let ((left_artboard, _, left_color), _) =
        left.edit(|tx| create_card(tx, "Left", 0xff112233))?;
    let left_instance = left.instantiate(left_artboard)?;
    let left_cursor = left.cursor(left_instance, left_color, props::COLOR_VALUE)?;

    let mut right = Scene::new();
    let ((right_artboard, _, _), _) = right.edit(|tx| create_card(tx, "Right", 0xff445566))?;
    let right_instance = right.instantiate(right_artboard)?;
    assert_eq!(left.epoch(), right.epoch());

    let before = draw_stream(&mut right, right_instance)?;
    assert_eq!(right.frame().set(left_cursor, 0xffaabbcc), Err(StaleCursor));
    assert_eq!(draw_stream(&mut right, right_instance)?, before);
    Ok(())
}

#[test]
fn public_ids_from_one_scene_never_alias_same_shaped_objects_or_instances_in_another() -> Result<()>
{
    let mut left = Scene::new();
    let ((left_artboard, _, left_color), _) =
        left.edit(|tx| create_card(tx, "Left", 0xff112233))?;
    let left_instance = left.instantiate(left_artboard)?;

    let mut right = Scene::new();
    let ((right_artboard, _, right_color), _) =
        right.edit(|tx| create_card(tx, "Right", 0xff445566))?;
    let right_instance = right.instantiate(right_artboard)?;
    let right_cursor = right.cursor(right_instance, right_color, props::COLOR_VALUE)?;

    assert!(right.instantiate(left_artboard).is_err());
    assert!(matches!(
        right.cursor(right_instance, left_color, props::COLOR_VALUE),
        Err(ResolveError::UnknownObject)
    ));
    let error = right
        .edit(|tx| {
            tx.set(left_color, props::COLOR_VALUE, 0xffaabbcc)?;
            Ok(())
        })
        .expect_err("an object id from another scene must not target a same-shaped object");
    assert_eq!(error.kind(), EditErrorKind::Aborted);
    assert_eq!(error.diagnostic().reason, EditReason::UnknownObject);

    right.drop_instance(left_instance);
    assert_eq!(right.frame().get(right_cursor)?, 0xff445566);
    Ok(())
}

#[test]
fn frame_get_reads_schema_defaults_and_current_hot_values() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, shape, color), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Card".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let fill = tx.create(
            Parent::Object(shape),
            NodeSpec::Fill(FillSpec {
                name: "Card Fill".into(),
            }),
        )?;
        let color = tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Card Color".into(),
                color: 0xff112233,
            }),
        )?;
        Ok((artboard, shape, color))
    })?;
    let instance = scene.instantiate(artboard)?;
    let opacity = scene.cursor(instance, shape, props::WORLD_OPACITY)?;
    let rotation = scene.cursor(instance, shape, props::ROTATION)?;
    let scale_x = scene.cursor(instance, shape, props::SCALE_X)?;
    let color = scene.cursor(instance, color, props::COLOR_VALUE)?;

    {
        let frame = scene.frame();
        assert_eq!(frame.get(opacity)?, 1.0);
        assert_eq!(frame.get(rotation)?, 0.0);
        assert_eq!(frame.get(scale_x)?, 1.0);
        assert_eq!(frame.get(color)?, 0xff112233);
    }

    {
        let mut frame = scene.frame();
        assert!(frame.set(opacity, 0.25)?);
        assert!(frame.set(color, 0xff445566)?);
        assert_eq!(frame.get(opacity)?, 0.25);
        assert_eq!(frame.get(color)?, 0xff445566);
    }
    Ok(())
}

#[test]
fn frame_get_rejects_stale_and_foreign_scene_cursors() -> Result<()> {
    let mut left = Scene::new();
    let ((left_artboard, left_rectangle, left_color), _) =
        left.edit(|tx| create_card(tx, "Left", 0xff112233))?;
    let left_instance = left.instantiate(left_artboard)?;
    let left_cursor = left.cursor(left_instance, left_color, props::COLOR_VALUE)?;

    let mut right = Scene::new();
    let ((right_artboard, _, _), _) = right.edit(|tx| create_card(tx, "Right", 0xff445566))?;
    right.instantiate(right_artboard)?;
    assert_eq!(left.epoch(), right.epoch());
    assert_eq!(right.frame().get(left_cursor), Err(StaleCursor));

    left.edit(|tx| {
        tx.set(left_rectangle, props::PATH_WIDTH, 72.0)?;
        Ok(())
    })?;
    assert_eq!(left.frame().get(left_cursor), Err(StaleCursor));
    Ok(())
}

#[test]
fn dropped_instance_slots_never_alias_old_cursors_and_do_not_disturb_other_instances() -> Result<()>
{
    let mut scene = Scene::new();
    let ((artboard, rectangle, color), _) = scene.edit(|tx| create_card(tx, "Main", 0xff112233))?;
    let dropped = scene.instantiate(artboard)?;
    let survivor = scene.instantiate(artboard)?;
    let dropped_cursor = scene.cursor(dropped, color, props::COLOR_VALUE)?;
    let survivor_cursor = scene.cursor(survivor, color, props::COLOR_VALUE)?;
    let epoch = scene.epoch();

    scene.drop_instance(dropped);
    assert_eq!(
        scene.epoch(),
        epoch,
        "instance lifecycle is not a definition edit"
    );
    assert_eq!(scene.frame().get(dropped_cursor), Err(StaleCursor));
    assert_eq!(
        scene.frame().set(dropped_cursor, 0xff445566),
        Err(StaleCursor)
    );
    assert!(scene.frame().set(survivor_cursor, 0xff556677)?);
    assert_eq!(scene.frame().get(survivor_cursor)?, 0xff556677);

    let replacement = scene.instantiate(artboard)?;
    let replacement_cursor = scene.cursor(replacement, color, props::COLOR_VALUE)?;
    assert_ne!(replacement, dropped, "instance ids are never reused");
    assert_eq!(scene.frame().get(dropped_cursor), Err(StaleCursor));
    assert_eq!(scene.frame().get(replacement_cursor)?, 0xff112233);
    assert!(scene.frame().set(replacement_cursor, 0xff667788)?);
    assert_eq!(scene.frame().get(replacement_cursor)?, 0xff667788);

    scene.drop_instance(dropped);
    assert_eq!(scene.frame().get(survivor_cursor)?, 0xff556677);

    scene.edit(|tx| {
        tx.set(rectangle, props::PATH_WIDTH, 72.0)?;
        Ok(())
    })?;
    assert_eq!(scene.frame().get(survivor_cursor), Err(StaleCursor));
    assert_eq!(scene.frame().get(replacement_cursor), Err(StaleCursor));
    let survivor_cursor = scene.cursor(survivor, color, props::COLOR_VALUE)?;
    let replacement_cursor = scene.cursor(replacement, color, props::COLOR_VALUE)?;
    assert_eq!(scene.frame().get(survivor_cursor)?, 0xff112233);
    assert_eq!(scene.frame().get(replacement_cursor)?, 0xff112233);
    Ok(())
}

#[test]
fn aborted_edits_burn_allocated_identity_without_publishing_definitions() -> Result<()> {
    let mut scene = Scene::new();
    let mut leaked_artboard = None;

    let aborted = scene.edit::<()>(|tx| {
        leaked_artboard = Some(tx.create_artboard(ArtboardSpec {
            name: "Aborted".into(),
            width: 100.0,
            height: 100.0,
        })?);
        Err(tx.abort("abort the transaction"))
    });
    assert!(aborted.is_err());

    let leaked_artboard = leaked_artboard.expect("the closure allocated an identity");
    let (committed_artboard, _) = scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Committed".into(),
            width: 100.0,
            height: 100.0,
        })
    })?;

    assert_ne!(committed_artboard, leaked_artboard);
    assert!(scene.instantiate(leaked_artboard).is_err());
    assert!(scene.instantiate(committed_artboard).is_ok());
    Ok(())
}

#[test]
fn edit_receipts_report_structure_epoch_and_only_created_object_ids() -> Result<()> {
    let mut scene = Scene::new();
    assert_eq!(scene.epoch(), StructureEpoch::INITIAL);

    let ((artboard, shape), receipt) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Card".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        Ok((artboard, shape))
    })?;

    assert_eq!(receipt.epoch, scene.epoch());
    assert_eq!(receipt.created, vec![shape]);
    assert_eq!(scene.epoch().get(), 1);

    // Artboard identity is deliberately not exposed as an ObjectId in the receipt.
    assert!(scene.instantiate(artboard).is_ok());
    Ok(())
}

#[test]
fn local_validation_returns_a_structured_abort_diagnostic() -> Result<()> {
    let mut scene = Scene::new();
    let (artboard, _) = scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })
    })?;
    let epoch_before = scene.epoch();

    let error = scene
        .edit(|tx| {
            tx.create(
                Parent::Artboard(artboard),
                NodeSpec::Rectangle(RectangleSpec::new("Invalid child", 10.0, 10.0)),
            )?;
            Ok(())
        })
        .expect_err("a Rectangle cannot be parented directly to an Artboard");

    assert_eq!(error.kind(), EditErrorKind::Aborted);
    assert_eq!(error.diagnostic().operation_index, 0);
    assert_eq!(
        error.diagnostic().involved_ids,
        vec![EditId::Artboard(artboard)]
    );
    assert_eq!(
        error.diagnostic().reason,
        EditReason::InvalidParent {
            parent: None,
            child: NodeKind::Rectangle,
        }
    );
    assert_eq!(scene.epoch(), epoch_before, "an abort is atomic");
    Ok(())
}

#[test]
fn materialization_failure_is_a_structured_edit_error_without_internal_details() -> Result<()> {
    let mut scene = Scene::new();
    let mut invalid_artboard = None;
    let error = scene
        .edit(|tx| {
            invalid_artboard = Some(tx.create_artboard(ArtboardSpec {
                name: "Invalid".into(),
                width: f32::NAN,
                height: 100.0,
            })?);
            Ok(())
        })
        .expect_err("non-finite geometry must be rejected at commit");

    assert_eq!(error.kind(), EditErrorKind::CommitRejected);
    assert_eq!(
        error.diagnostic().operation_index,
        0,
        "commit validation must point to the operation that introduced the invalid spec"
    );
    assert_eq!(
        error.diagnostic().involved_ids,
        vec![EditId::Artboard(
            invalid_artboard.expect("the transaction allocated the artboard")
        )]
    );
    assert_eq!(
        error.diagnostic().reason,
        EditReason::NonFiniteProperty { property: "width" }
    );
    assert_eq!(error.to_string(), "scene edit was rejected during commit");
    assert_eq!(scene.epoch(), StructureEpoch::INITIAL);
    Ok(())
}

#[test]
fn generated_authoring_vocabulary_tracks_schema_owners_value_kinds_and_surface_availability() {
    assert_eq!(NodeKind::Rectangle.schema_name(), "Rectangle");
    assert_eq!(NodeKind::Text.schema_name(), "Text");

    assert_eq!(props::PATH_WIDTH.schema_name(), "width");
    assert!(props::PATH_WIDTH.is_available_on(NodeKind::Rectangle));
    assert_eq!(props::PATH_WIDTH.value_kind(), PropValueKind::Double);
    assert_eq!(props::PATH_WIDTH.declared_owner(), "ParametricPath");

    assert_eq!(props::PATH_HEIGHT.schema_name(), "height");
    assert!(props::PATH_HEIGHT.is_available_on(NodeKind::Rectangle));
    assert_eq!(props::PATH_HEIGHT.value_kind(), PropValueKind::Double);
    assert_eq!(props::PATH_HEIGHT.declared_owner(), "ParametricPath");

    assert_eq!(props::STROKE_THICKNESS.schema_name(), "thickness");
    assert!(props::STROKE_THICKNESS.is_available_on(NodeKind::Stroke));
    assert_eq!(props::STROKE_THICKNESS.value_kind(), PropValueKind::Double);
    assert_eq!(props::STROKE_THICKNESS.declared_owner(), "Stroke");

    assert_eq!(props::DASH_LENGTH.schema_name(), "length");
    assert!(props::DASH_LENGTH.is_available_on(NodeKind::Dash));
    assert_eq!(props::DASH_LENGTH.value_kind(), PropValueKind::Double);
    assert_eq!(props::DASH_LENGTH.declared_owner(), "Dash");

    assert_eq!(props::COLOR_VALUE.schema_name(), "colorValue");
    assert!(props::COLOR_VALUE.is_available_on(NodeKind::SolidColor));
    assert_eq!(props::COLOR_VALUE.value_kind(), PropValueKind::Color);
    assert_eq!(props::COLOR_VALUE.declared_owner(), "SolidColor");

    assert_eq!(props::WORLD_OPACITY.schema_name(), "opacity");
    assert!(props::WORLD_OPACITY.is_available_on(NodeKind::Shape));
    assert!(props::WORLD_OPACITY.is_available_on(NodeKind::Text));
    assert_eq!(props::WORLD_OPACITY.value_kind(), PropValueKind::Double);
    assert_eq!(
        props::WORLD_OPACITY.declared_owner(),
        "WorldTransformComponent"
    );

    for property in [props::TRANSLATE_X, props::TRANSLATE_Y] {
        assert!(property.is_available_on(NodeKind::Shape));
        assert!(property.is_available_on(NodeKind::Text));
        assert_eq!(property.value_kind(), PropValueKind::Double);
        assert_eq!(property.declared_owner(), "Node");
    }

    for property in [props::ROTATION, props::SCALE_X, props::SCALE_Y] {
        assert!(property.is_available_on(NodeKind::Shape));
        assert!(property.is_available_on(NodeKind::Text));
        assert_eq!(property.value_kind(), PropValueKind::Double);
        assert_eq!(property.declared_owner(), "TransformComponent");
    }
}

#[test]
fn export_records_are_sparse_canonical_and_compose_one_backboard() -> Result<()> {
    let mut scene = Scene::new();
    scene.edit(|tx| {
        let first = tx.create_artboard(ArtboardSpec {
            name: "First".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let default_shape = tx.create(
            Parent::Artboard(first),
            NodeSpec::Shape(ShapeSpec {
                name: "Default".into(),
                x: 10.0,
                y: 20.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Object(default_shape),
            NodeSpec::Rectangle(RectangleSpec::new("Default Rectangle", 40.0, 30.0)),
        )?;
        tx.create(
            Parent::Artboard(first),
            NodeSpec::Shape(ShapeSpec {
                name: "Complex".into(),
                x: 30.0,
                y: 40.0,
                opacity: 0.4,
                rotation: 0.25,
                scale_x: 1.5,
                scale_y: 0.5,
            }),
        )?;
        tx.create_artboard(ArtboardSpec {
            name: "Second".into(),
            width: 200.0,
            height: 150.0,
        })?;
        Ok(())
    })?;

    let exported = scene.export_records();
    let records = exported.records();
    assert_eq!(
        records.iter().map(|record| record.kind).collect::<Vec<_>>(),
        vec![
            ExportedObjectKind::Backboard,
            ExportedObjectKind::Artboard,
            ExportedObjectKind::Shape,
            ExportedObjectKind::Rectangle,
            ExportedObjectKind::Shape,
            ExportedObjectKind::Artboard,
        ]
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record.kind == ExportedObjectKind::Backboard)
            .count(),
        1
    );
    let properties = |index: usize| {
        records
            .get(index)
            .map(|record| record.properties.clone())
            .unwrap_or_default()
    };
    assert_eq!(
        properties(2),
        vec![
            ExportedProperty::ComponentName("Default".into()),
            ExportedProperty::TranslateX(10.0),
            ExportedProperty::TranslateY(20.0),
        ],
        "root parent and identity transform defaults are omitted"
    );
    assert_eq!(
        properties(3),
        vec![
            ExportedProperty::ComponentName("Default Rectangle".into()),
            ExportedProperty::ParentId(1),
            ExportedProperty::PathWidth(40.0),
            ExportedProperty::PathHeight(30.0),
        ],
        "non-root parent is present and properties are canonical without exposing schema keys"
    );
    assert_eq!(
        properties(4),
        vec![
            ExportedProperty::ComponentName("Complex".into()),
            ExportedProperty::TranslateX(30.0),
            ExportedProperty::TranslateY(40.0),
            ExportedProperty::Rotation(0.25),
            ExportedProperty::ScaleX(1.5),
            ExportedProperty::ScaleY(0.5),
            ExportedProperty::WorldOpacity(0.4),
        ]
    );
    Ok(())
}

#[test]
fn typed_scripted_drawable_exports_raw_script_and_shader_payloads_with_one_envelope() -> Result<()>
{
    let mut scene = Scene::new();
    scene.edit(|tx| {
        let script = tx.create_script_asset(ScriptAssetSpec {
            name: "node".into(),
            is_module: false,
            bytes: vec![0x1b, b'L', b'u', b'a', b'u'],
        })?;
        tx.create_shader_asset(ShaderAssetSpec {
            name: "fill".into(),
            bytes: b"RSTB".to_vec(),
        })?;
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Scripted".into(),
            width: 100.0,
            height: 100.0,
        })?;
        tx.create(
            Parent::Artboard(artboard),
            NodeSpec::ScriptedDrawable(ScriptedDrawableSpec {
                name: "Canvas".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                script,
            }),
        )?;
        Ok(())
    })?;

    let records = scene.export_records();
    assert_eq!(
        records
            .records()
            .iter()
            .map(|record| record.kind)
            .collect::<Vec<_>>(),
        vec![
            ExportedObjectKind::Backboard,
            ExportedObjectKind::ScriptAsset,
            ExportedObjectKind::FileAssetContents,
            ExportedObjectKind::ShaderAsset,
            ExportedObjectKind::FileAssetContents,
            ExportedObjectKind::Artboard,
            ExportedObjectKind::ScriptedDrawable,
        ]
    );
    assert_eq!(
        records.records()[1].properties,
        vec![
            ExportedProperty::AssetName("node".into()),
            ExportedProperty::FileAssetId(0),
        ],
        "the protocol-script default isModule=false remains sparse"
    );
    assert_eq!(
        records.records()[2].properties,
        vec![ExportedProperty::FileAssetContentsBytes(vec![
            0x00, 0x1b, b'L', b'u', b'a', b'u',
        ])],
        "Scene adds exactly one unsigned text-asset envelope to raw Luau bytecode"
    );
    assert_eq!(
        records.records()[3].properties,
        vec![
            ExportedProperty::AssetName("fill".into()),
            ExportedProperty::FileAssetId(1),
        ]
    );
    assert_eq!(
        records.records()[4].properties,
        vec![ExportedProperty::FileAssetContentsBytes(b"\0RSTB".to_vec())],
        "Scene adds exactly one unsigned text-asset envelope to raw RSTB"
    );
    assert_eq!(
        records.records()[6].properties,
        vec![
            ExportedProperty::ComponentName("Canvas".into()),
            ExportedProperty::ScriptedDrawableScriptAssetId(0),
        ]
    );
    Ok(())
}

#[test]
fn empty_script_and_shader_payloads_are_structurally_preserved_as_unsigned_envelopes() -> Result<()>
{
    let mut scene = Scene::new();
    scene.edit(|tx| {
        tx.create_script_asset(ScriptAssetSpec {
            name: String::new(),
            is_module: false,
            bytes: Vec::new(),
        })?;
        tx.create_shader_asset(ShaderAssetSpec {
            name: String::new(),
            bytes: Vec::new(),
        })?;
        Ok(())
    })?;

    assert_eq!(
        scene.export_records().records(),
        &[
            ExportedRecord {
                kind: ExportedObjectKind::Backboard,
                properties: vec![],
            },
            ExportedRecord {
                kind: ExportedObjectKind::ScriptAsset,
                properties: vec![
                    ExportedProperty::AssetName(String::new()),
                    ExportedProperty::FileAssetId(0),
                ],
            },
            ExportedRecord {
                kind: ExportedObjectKind::FileAssetContents,
                properties: vec![ExportedProperty::FileAssetContentsBytes(vec![0x00])],
            },
            ExportedRecord {
                kind: ExportedObjectKind::ShaderAsset,
                properties: vec![
                    ExportedProperty::AssetName(String::new()),
                    ExportedProperty::FileAssetId(1),
                ],
            },
            ExportedRecord {
                kind: ExportedObjectKind::FileAssetContents,
                properties: vec![ExportedProperty::FileAssetContentsBytes(vec![0x00])],
            },
        ],
        "Scene owns the unsigned text-asset envelope; payload/name validity belongs to the scripting and renderer consumers"
    );
    Ok(())
}

#[test]
fn scripted_drawable_rejects_a_script_identity_owned_by_another_scene_atomically() -> Result<()> {
    let mut source = Scene::new();
    let (foreign_script, _) = source.edit(|tx| {
        tx.create_script_asset(ScriptAssetSpec {
            name: "foreign".into(),
            is_module: false,
            bytes: b"foreign bytecode".to_vec(),
        })
    })?;
    let mut target = Scene::new();
    let records_before = target.export_records();
    let mut drawable = None;

    let error = target
        .edit(|tx| {
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Target".into(),
                width: 100.0,
                height: 100.0,
            })?;
            drawable = Some(tx.create(
                Parent::Artboard(artboard),
                NodeSpec::ScriptedDrawable(ScriptedDrawableSpec {
                    name: "Foreign Script".into(),
                    x: 0.0,
                    y: 0.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    script: foreign_script,
                }),
            )?);
            Ok(())
        })
        .expect_err("a script identity is meaningful only in its owning scene");

    assert_eq!(error.kind(), EditErrorKind::CommitRejected);
    assert_eq!(error.diagnostic().reason, EditReason::UnknownScriptAsset);
    assert_eq!(
        error.diagnostic().involved_ids,
        vec![
            EditId::Object(drawable.expect("the transaction allocated the drawable")),
            EditId::ScriptAsset(foreign_script),
        ]
    );
    assert_eq!(target.epoch(), StructureEpoch::INITIAL);
    assert_eq!(target.export_records(), records_before);
    Ok(())
}

#[test]
fn sparse_export_omits_only_exact_schema_defaults_across_remounts() -> Result<()> {
    let next_below_one = f32::from_bits(1.0f32.to_bits() - 1);
    let tiny_rotation = f32::EPSILON / 2.0;
    let mut scene = Scene::new();
    let ((artboard, shape, rectangle), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Near defaults".into(),
                x: 0.0,
                y: 0.0,
                opacity: next_below_one,
                rotation: tiny_rotation,
                scale_x: next_below_one,
                scale_y: next_below_one,
            }),
        )?;
        let rectangle = tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new("Rectangle", 80.0, 60.0)),
        )?;
        Ok((artboard, shape, rectangle))
    })?;

    let records = scene.export_records();
    assert_eq!(
        records.records()[2].properties,
        vec![
            ExportedProperty::ComponentName("Near defaults".into()),
            ExportedProperty::Rotation(tiny_rotation),
            ExportedProperty::ScaleX(next_below_one),
            ExportedProperty::ScaleY(next_below_one),
            ExportedProperty::WorldOpacity(next_below_one),
        ],
        "exact defaults are omitted while adjacent representable values remain"
    );

    let instance = scene.instantiate(artboard)?;
    let assert_near_defaults = |scene: &mut Scene| -> Result<()> {
        let opacity = scene.cursor(instance, shape, props::WORLD_OPACITY)?;
        let rotation = scene.cursor(instance, shape, props::ROTATION)?;
        let scale_x = scene.cursor(instance, shape, props::SCALE_X)?;
        let scale_y = scene.cursor(instance, shape, props::SCALE_Y)?;
        let frame = scene.frame();
        assert_eq!(frame.get(opacity)?, next_below_one);
        assert_eq!(frame.get(rotation)?, tiny_rotation);
        assert_eq!(frame.get(scale_x)?, next_below_one);
        assert_eq!(frame.get(scale_y)?, next_below_one);
        Ok(())
    };
    assert_near_defaults(&mut scene)?;

    scene.edit(|tx| {
        tx.set(rectangle, props::PATH_WIDTH, 72.0)?;
        Ok(())
    })?;
    assert_near_defaults(&mut scene)?;
    Ok(())
}

#[test]
fn typed_feather_materializes_draws_and_exports_without_raw_schema_keys() -> Result<()> {
    let mut scene = Scene::new();
    let (artboard, _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Shadow".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Shadow Shape".into(),
                x: 25.0,
                y: 25.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new("Shadow Rectangle", 40.0, 30.0)),
        )?;
        let fill = tx.create(
            Parent::Object(shape),
            NodeSpec::Fill(FillSpec {
                name: "Shadow Fill".into(),
            }),
        )?;
        tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Shadow Color".into(),
                color: 0xff11_2233,
            }),
        )?;
        tx.create(
            Parent::Object(fill),
            NodeSpec::Feather(FeatherSpec {
                name: "Shadow Feather".into(),
                space: SceneFeatherSpace::Local,
                strength: 7.5,
                offset_x: 3.0,
                offset_y: -4.0,
                inner: false,
            }),
        )?;
        Ok(artboard)
    })?;

    let records = scene.export_records();
    let feather = records
        .records()
        .iter()
        .find(|record| record.kind == ExportedObjectKind::Feather)
        .expect("typed scene exports its feather child");
    assert_eq!(
        feather.properties,
        vec![
            ExportedProperty::ComponentName("Shadow Feather".into()),
            ExportedProperty::ParentId(3),
            ExportedProperty::FeatherSpace(SceneFeatherSpace::Local),
            ExportedProperty::FeatherStrength(7.5),
            ExportedProperty::FeatherOffsetX(3.0),
            ExportedProperty::FeatherOffsetY(-4.0),
            ExportedProperty::FeatherInner(false),
        ]
    );

    let instance = scene.instantiate(artboard)?;
    let stream = draw_stream(&mut scene, instance)?;
    assert!(stream.contains("feather=7.5"), "{stream}");
    assert!(stream.contains("color=0xff112233"), "{stream}");
    Ok(())
}

#[test]
fn typed_scene_materializes_rectangle_radii_and_a_dashed_stroke_without_raw_schema_keys()
-> Result<()> {
    let mut scene = Scene::new();
    let (artboard, _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Border".into(),
            width: 10.0,
            height: 10.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Border Shape".into(),
                x: 5.0,
                y: 5.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec {
                name: "Border Rectangle".into(),
                width: 8.0,
                height: 8.0,
                corner_radii: Some(RectangleCornerRadii {
                    top_left: 1.0,
                    top_right: 2.0,
                    bottom_right: 3.0,
                    bottom_left: 4.0,
                    linked: false,
                }),
            }),
        )?;
        let fill = tx.create(
            Parent::Object(shape),
            NodeSpec::Fill(FillSpec {
                name: "Border Fill".into(),
            }),
        )?;
        tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Fill Color".into(),
                color: 0xff112233,
            }),
        )?;
        let stroke = tx.create(
            Parent::Object(shape),
            NodeSpec::Stroke(StrokeSpec {
                name: "Border Stroke".into(),
                thickness: 2.0,
                cap: SceneStrokeCap::Butt,
                join: SceneStrokeJoin::Miter,
                transform_affects_stroke: true,
            }),
        )?;
        tx.create(
            Parent::Object(stroke),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Stroke Color".into(),
                color: 0xff445566,
            }),
        )?;
        let dash_path = tx.create(
            Parent::Object(stroke),
            NodeSpec::DashPath(DashPathSpec {
                name: "Dash Path".into(),
                offset: 0.0,
                offset_is_percentage: false,
            }),
        )?;
        for (index, length) in [0.5, 0.5].into_iter().enumerate() {
            tx.create(
                Parent::Object(dash_path),
                NodeSpec::Dash(DashSpec {
                    name: format!("Dash {index}"),
                    length,
                    length_is_percentage: true,
                }),
            )?;
        }
        Ok(artboard)
    })?;

    let records = scene.export_records();
    let [
        _,
        _,
        _,
        rectangle,
        _,
        _,
        stroke,
        stroke_color,
        dash_path,
        dash_on,
        dash_off,
    ] = records.records()
    else {
        panic!("border scene must export exactly eleven records");
    };
    assert_eq!(
        records
            .records()
            .iter()
            .map(|record| record.kind)
            .collect::<Vec<_>>(),
        vec![
            ExportedObjectKind::Backboard,
            ExportedObjectKind::Artboard,
            ExportedObjectKind::Shape,
            ExportedObjectKind::Rectangle,
            ExportedObjectKind::Fill,
            ExportedObjectKind::SolidColor,
            ExportedObjectKind::Stroke,
            ExportedObjectKind::SolidColor,
            ExportedObjectKind::DashPath,
            ExportedObjectKind::Dash,
            ExportedObjectKind::Dash,
        ]
    );
    assert_eq!(
        rectangle.properties,
        vec![
            ExportedProperty::ComponentName("Border Rectangle".into()),
            ExportedProperty::ParentId(1),
            ExportedProperty::PathWidth(8.0),
            ExportedProperty::PathHeight(8.0),
            ExportedProperty::RectangleCornerRadiusTopLeft(1.0),
            ExportedProperty::RectangleCornerRadiusTopRight(2.0),
            ExportedProperty::RectangleCornerRadiusBottomLeft(4.0),
            ExportedProperty::RectangleCornerRadiusBottomRight(3.0),
            ExportedProperty::RectangleLinkCornerRadius(false),
        ]
    );
    assert_eq!(
        stroke.properties,
        vec![
            ExportedProperty::ComponentName("Border Stroke".into()),
            ExportedProperty::ParentId(1),
            ExportedProperty::StrokeThickness(2.0),
            ExportedProperty::StrokeCap(SceneStrokeCap::Butt),
            ExportedProperty::StrokeJoin(SceneStrokeJoin::Miter),
            ExportedProperty::StrokeTransformAffectsStroke(true),
        ]
    );
    assert_eq!(
        stroke_color.properties,
        vec![
            ExportedProperty::ComponentName("Stroke Color".into()),
            ExportedProperty::ParentId(5),
            ExportedProperty::ColorValue(0xff445566),
        ]
    );
    assert_eq!(
        dash_path.properties,
        vec![
            ExportedProperty::ComponentName("Dash Path".into()),
            ExportedProperty::ParentId(5),
            ExportedProperty::DashOffset(0.0),
            ExportedProperty::DashOffsetIsPercentage(false),
        ]
    );
    assert_eq!(
        dash_on.properties,
        vec![
            ExportedProperty::ComponentName("Dash 0".into()),
            ExportedProperty::ParentId(7),
            ExportedProperty::DashLength(0.5),
            ExportedProperty::DashLengthIsPercentage(true),
        ]
    );
    assert_eq!(
        dash_off.properties,
        vec![
            ExportedProperty::ComponentName("Dash 1".into()),
            ExportedProperty::ParentId(7),
            ExportedProperty::DashLength(0.5),
            ExportedProperty::DashLengthIsPercentage(true),
        ]
    );

    let instance = scene.instantiate(artboard)?;
    let stream = draw_stream(&mut scene, instance)?;
    assert!(stream.contains("style=stroke"));
    assert!(stream.contains("color=0xff445566"));
    Ok(())
}

#[test]
fn typed_points_path_exports_exact_cubic_records_and_draws_after_runtime_import() -> Result<()> {
    let mut scene = Scene::new();
    let (artboard, _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Cubic path".into(),
            width: 40.0,
            height: 40.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Cubic shape".into(),
                x: 20.0,
                y: 20.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let path = tx.create(
            Parent::Object(shape),
            NodeSpec::PointsPath(PointsPathSpec {
                name: "Closed cubic path".into(),
                is_closed: true,
            }),
        )?;
        for (index, spec) in [
            CubicDetachedVertexSpec {
                name: "Vertex 0".into(),
                x: -10.0,
                y: -10.0,
                in_rotation: 0.0,
                in_distance: 0.0,
                out_rotation: 0.0,
                out_distance: 5.0,
            },
            CubicDetachedVertexSpec {
                name: "Vertex 1".into(),
                x: 10.0,
                y: -10.0,
                in_rotation: core::f32::consts::PI,
                in_distance: 5.0,
                out_rotation: core::f32::consts::FRAC_PI_2,
                out_distance: 5.0,
            },
            CubicDetachedVertexSpec {
                name: "Vertex 2".into(),
                x: 0.0,
                y: 10.0,
                in_rotation: -core::f32::consts::FRAC_PI_2,
                in_distance: 5.0,
                out_rotation: core::f32::consts::PI,
                out_distance: 5.0,
            },
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(index.to_string(), spec.name.trim_start_matches("Vertex "));
            tx.create(Parent::Object(path), NodeSpec::CubicDetachedVertex(spec))?;
        }
        let fill = tx.create(
            Parent::Object(shape),
            NodeSpec::Fill(FillSpec {
                name: "Cubic fill".into(),
            }),
        )?;
        tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Cubic color".into(),
                color: 0xff12_3456,
            }),
        )?;
        Ok(artboard)
    })?;

    let records = scene.export_records();
    let path = records
        .records()
        .iter()
        .find(|record| record.kind == ExportedObjectKind::PointsPath)
        .expect("typed scene exports the points path");
    assert_eq!(
        path.properties,
        vec![
            ExportedProperty::ComponentName("Closed cubic path".into()),
            ExportedProperty::ParentId(1),
            ExportedProperty::PointsPathIsClosed(true),
        ]
    );
    let vertices = records
        .records()
        .iter()
        .filter(|record| record.kind == ExportedObjectKind::CubicDetachedVertex)
        .collect::<Vec<_>>();
    assert_eq!(vertices.len(), 3);
    assert_eq!(
        vertices[0].properties,
        vec![
            ExportedProperty::ComponentName("Vertex 0".into()),
            ExportedProperty::ParentId(2),
            ExportedProperty::VertexX(-10.0),
            ExportedProperty::VertexY(-10.0),
            ExportedProperty::CubicDetachedVertexInRotation(0.0),
            ExportedProperty::CubicDetachedVertexInDistance(0.0),
            ExportedProperty::CubicDetachedVertexOutRotation(0.0),
            ExportedProperty::CubicDetachedVertexOutDistance(5.0),
        ]
    );

    let instance = scene.instantiate(artboard)?;
    let stream = draw_stream(&mut scene, instance)?;
    assert!(stream.contains("style=fill"), "{stream}");
    assert!(stream.contains("color=0xff123456"), "{stream}");
    assert!(stream.contains("cubic"), "{stream}");
    Ok(())
}

#[test]
fn cubic_detached_vertex_rejects_non_finite_geometry_atomically() -> Result<()> {
    let mut scene = Scene::new();
    let error = scene
        .edit(|tx| {
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Invalid cubic path".into(),
                width: 40.0,
                height: 40.0,
            })?;
            let shape = tx.create(
                Parent::Artboard(artboard),
                NodeSpec::Shape(ShapeSpec {
                    name: "Shape".into(),
                    x: 0.0,
                    y: 0.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                }),
            )?;
            let path = tx.create(
                Parent::Object(shape),
                NodeSpec::PointsPath(PointsPathSpec {
                    name: "Path".into(),
                    is_closed: false,
                }),
            )?;
            tx.create(
                Parent::Object(path),
                NodeSpec::CubicDetachedVertex(CubicDetachedVertexSpec {
                    name: "Invalid vertex".into(),
                    x: 0.0,
                    y: 0.0,
                    in_rotation: 0.0,
                    in_distance: 0.0,
                    out_rotation: 0.0,
                    out_distance: f32::NAN,
                }),
            )?;
            Ok(())
        })
        .expect_err("non-finite cubic handles must reject the transaction");
    assert_eq!(error.kind(), EditErrorKind::CommitRejected);
    assert_eq!(
        error.diagnostic().reason,
        EditReason::NonFiniteProperty {
            property: "out_distance",
        }
    );
    assert_eq!(scene.epoch(), StructureEpoch::INITIAL);
    assert_eq!(scene.export_records().records().len(), 1);
    Ok(())
}

#[test]
fn typed_scene_materializes_and_draws_a_real_linear_gradient_subtree() -> Result<()> {
    let mut scene = Scene::new();
    let (artboard, _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Gradient".into(),
            width: 20.0,
            height: 20.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Gradient Shape".into(),
                x: 10.0,
                y: 10.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new("Gradient Rectangle", 20.0, 20.0)),
        )?;
        let fill = tx.create(
            Parent::Object(shape),
            NodeSpec::Fill(FillSpec {
                name: "Gradient Fill".into(),
            }),
        )?;
        let gradient = tx.create(
            Parent::Object(fill),
            NodeSpec::LinearGradient(LinearGradientSpec {
                name: "Background Gradient".into(),
                start_x: -10.0,
                start_y: 0.0,
                end_x: 10.0,
                end_y: 0.0,
                opacity: 0.75,
            }),
        )?;
        for (index, (color, position)) in [(0xff09_0e38, 0.0), (0xff05_0410, 1.0)]
            .into_iter()
            .enumerate()
        {
            tx.create(
                Parent::Object(gradient),
                NodeSpec::GradientStop(GradientStopSpec {
                    name: format!("Gradient Stop {index}"),
                    color,
                    position,
                }),
            )?;
        }
        Ok(artboard)
    })?;

    let records = scene.export_records();
    assert_eq!(
        records
            .records()
            .iter()
            .map(|record| record.kind)
            .collect::<Vec<_>>(),
        vec![
            ExportedObjectKind::Backboard,
            ExportedObjectKind::Artboard,
            ExportedObjectKind::Shape,
            ExportedObjectKind::Rectangle,
            ExportedObjectKind::Fill,
            ExportedObjectKind::LinearGradient,
            ExportedObjectKind::GradientStop,
            ExportedObjectKind::GradientStop,
        ]
    );
    assert_eq!(
        records.records()[5].properties,
        vec![
            ExportedProperty::ComponentName("Background Gradient".into()),
            ExportedProperty::ParentId(3),
            ExportedProperty::LinearGradientStartY(0.0),
            ExportedProperty::LinearGradientEndX(10.0),
            ExportedProperty::LinearGradientEndY(0.0),
            ExportedProperty::LinearGradientStartX(-10.0),
            ExportedProperty::LinearGradientOpacity(0.75),
        ]
    );
    assert_eq!(
        records.records()[6].properties,
        vec![
            ExportedProperty::ComponentName("Gradient Stop 0".into()),
            ExportedProperty::ParentId(4),
            ExportedProperty::GradientStopColorValue(0xff09_0e38),
            ExportedProperty::GradientStopPosition(0.0),
        ]
    );

    let instance = scene.instantiate(artboard)?;
    let stream = draw_stream(&mut scene, instance)?;
    assert!(
        stream.contains(
            "start=(-10,0) end=(10,0) stops=[{color=0xbf090e38,stop=0},{color=0xbf050410,stop=1}]"
        ),
        "{stream}"
    );
    Ok(())
}

#[test]
fn explicit_all_zero_rectangle_radii_remain_present_in_typed_export() -> Result<()> {
    let mut scene = Scene::new();
    scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Zero Radius".into(),
            width: 10.0,
            height: 10.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Shape".into(),
                x: 5.0,
                y: 5.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec {
                name: "Rectangle".into(),
                width: 8.0,
                height: 8.0,
                corner_radii: Some(RectangleCornerRadii {
                    top_left: 0.0,
                    top_right: 0.0,
                    bottom_right: 0.0,
                    bottom_left: 0.0,
                    linked: false,
                }),
            }),
        )?;
        Ok(())
    })?;

    let records = scene.export_records();
    let [_, _, _, rectangle] = records.records() else {
        panic!("zero-radius scene must export exactly four records");
    };
    assert_eq!(
        rectangle.properties,
        vec![
            ExportedProperty::ComponentName("Rectangle".into()),
            ExportedProperty::ParentId(1),
            ExportedProperty::PathWidth(8.0),
            ExportedProperty::PathHeight(8.0),
            ExportedProperty::RectangleCornerRadiusTopLeft(0.0),
            ExportedProperty::RectangleCornerRadiusTopRight(0.0),
            ExportedProperty::RectangleCornerRadiusBottomLeft(0.0),
            ExportedProperty::RectangleCornerRadiusBottomRight(0.0),
            ExportedProperty::RectangleLinkCornerRadius(false),
        ]
    );
    Ok(())
}

#[test]
fn invalid_create_keeps_its_diagnostic_origin_across_later_hierarchy_and_value_edits() -> Result<()>
{
    let mut scene = Scene::new();

    let error = scene
        .edit(|tx| {
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Main".into(),
                width: 100.0,
                height: 100.0,
            })?;
            tx.create(
                Parent::Artboard(artboard),
                NodeSpec::Shape(ShapeSpec {
                    name: "Stable sibling".into(),
                    x: 0.0,
                    y: 0.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                }),
            )?;
            let invalid = tx.create(
                Parent::Artboard(artboard),
                NodeSpec::Shape(ShapeSpec {
                    name: "Invalid at creation".into(),
                    x: f32::NAN,
                    y: 0.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                }),
            )?;
            tx.reorder(invalid, ChildIndex::First)?;
            tx.reparent(invalid, Parent::Artboard(artboard), ChildIndex::Last)?;
            tx.set(invalid, props::ROTATION, 0.25)?;
            Ok(())
        })
        .expect_err("the non-finite create must fail commit");

    assert_eq!(error.kind(), EditErrorKind::CommitRejected);
    assert_eq!(
        error.diagnostic().operation_index,
        2,
        "hierarchy touches and a later valid property write must not steal the invalid spec's origin"
    );
    assert_eq!(
        error.diagnostic().reason,
        EditReason::NonFiniteProperty { property: "x" }
    );
    Ok(())
}

#[test]
fn restoring_an_uncommitted_invalid_subtree_attributes_the_spec_to_restore() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, removed), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let invalid = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Invalid token".into(),
                x: f32::NAN,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let removed = tx.remove(invalid)?;
        Ok((artboard, removed))
    })?;

    let error = scene
        .edit(|tx| {
            tx.set_artboard(
                artboard,
                ArtboardSpec {
                    name: "Still Main".into(),
                    width: 100.0,
                    height: 100.0,
                },
            )?;
            tx.restore(removed)?;
            Ok(())
        })
        .expect_err("restoring the invalid opaque token must reject commit");

    assert_eq!(error.kind(), EditErrorKind::CommitRejected);
    assert_eq!(error.diagnostic().operation_index, 1);
    assert_eq!(
        error.diagnostic().reason,
        EditReason::NonFiniteProperty { property: "x" }
    );
    Ok(())
}

#[test]
fn clear_artboard_is_one_atomic_typed_replacement_operation() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, removed), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let mut roots = Vec::new();
        for name in ["Old A", "Old B", "Old C"] {
            roots.push(tx.create(
                Parent::Artboard(artboard),
                NodeSpec::Shape(ShapeSpec {
                    name: name.into(),
                    x: 0.0,
                    y: 0.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                }),
            )?);
        }
        Ok((artboard, roots))
    })?;
    let instance = scene.instantiate(artboard)?;

    let (replacement, receipt) = scene.edit(|tx| {
        tx.clear_artboard(artboard)?;
        tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Replacement".into(),
                x: 10.0,
                y: 20.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )
    })?;

    assert_eq!(receipt.created, vec![replacement]);
    assert_eq!(
        exported_component_names(&scene),
        vec!["Main", "Replacement"]
    );
    for object in removed {
        assert!(matches!(
            scene.cursor(instance, object, props::WORLD_OPACITY),
            Err(ResolveError::UnknownObject)
        ));
    }
    assert!(
        scene
            .cursor(instance, replacement, props::WORLD_OPACITY)
            .is_ok()
    );
    Ok(())
}

#[test]
fn generated_transform_props_keep_local_validation_typed_and_atomic() -> Result<()> {
    let mut scene = Scene::new();
    let ((_, shape), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Card".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        Ok((artboard, shape))
    })?;
    let epoch_before = scene.epoch();

    let error = scene
        .edit(|tx| {
            tx.set(shape, props::SCALE_X, f32::NAN)?;
            Ok(())
        })
        .expect_err("non-finite generated property writes must abort locally");

    assert_eq!(error.kind(), EditErrorKind::Aborted);
    assert_eq!(error.diagnostic().operation_index, 0);
    assert_eq!(error.diagnostic().involved_ids, vec![EditId::Object(shape)]);
    assert_eq!(
        error.diagnostic().reason,
        EditReason::NonFiniteProperty {
            property: "scale_x"
        }
    );
    assert_eq!(scene.epoch(), epoch_before);
    Ok(())
}

#[test]
fn failed_materialization_burns_allocated_identity_without_changing_the_scene() -> Result<()> {
    let mut scene = Scene::new();
    let (initial_artboard, _) = scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Initial".into(),
            width: 100.0,
            height: 100.0,
        })
    })?;
    let initial_instance = scene.instantiate(initial_artboard)?;
    let before = draw_stream(&mut scene, initial_instance)?;

    let mut leaked_artboard = None;
    let failed = scene.edit(|tx| {
        leaked_artboard = Some(tx.create_artboard(ArtboardSpec {
            name: "Invalid".into(),
            width: f32::NAN,
            height: 100.0,
        })?);
        Ok(())
    });
    assert!(failed.is_err(), "non-finite geometry must not materialize");

    let leaked_artboard = leaked_artboard.expect("the closure allocated an identity");
    assert_eq!(draw_stream(&mut scene, initial_instance)?, before);
    assert!(scene.instantiate(leaked_artboard).is_err());

    let (committed_artboard, _) = scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Committed".into(),
            width: 100.0,
            height: 100.0,
        })
    })?;
    assert_ne!(committed_artboard, leaked_artboard);
    Ok(())
}

#[test]
fn occurrence_renderer_held_across_a_structural_remount_matches_a_fresh_draw() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, rectangle), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Card".into(),
                x: 50.0,
                y: 50.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let rectangle = tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new("Card Rectangle", 80.0, 60.0)),
        )?;
        let fill = tx.create(
            Parent::Object(shape),
            NodeSpec::Fill(FillSpec {
                name: "Card Fill".into(),
            }),
        )?;
        tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Card Color".into(),
                color: 0xff112233,
            }),
        )?;
        Ok((artboard, rectangle))
    })?;
    let instance = scene.instantiate(artboard)?;

    let mut original_factory = RecordingFactory::new();
    let mut held_token = scene.new_draw_token(instance)?;
    let mut original_renderer = original_factory.make_renderer();
    scene.frame().draw(
        instance,
        &mut original_factory,
        &mut original_renderer,
        &mut held_token,
    )?;

    scene.edit(|tx| {
        tx.set(rectangle, props::PATH_WIDTH, 72.0)?;
        Ok(())
    })?;

    let mut refreshed_factory = RecordingFactory::new();
    let mut refreshed_renderer = refreshed_factory.make_renderer();
    scene.frame().draw(
        instance,
        &mut refreshed_factory,
        &mut refreshed_renderer,
        &mut held_token,
    )?;

    let mut fresh_factory = RecordingFactory::new();
    let mut fresh_token = scene.new_draw_token(instance)?;
    let mut fresh_renderer = fresh_factory.make_renderer();
    scene.frame().draw(
        instance,
        &mut fresh_factory,
        &mut fresh_renderer,
        &mut fresh_token,
    )?;

    // A held occurrence draws warm while the fresh backend must construct its
    // resources first. Canonical recordings compare the complete retained
    // path/paint state at the draw instead of incidental construction commands.
    assert_eq!(
        refreshed_factory.canonical_recording(),
        fresh_factory.canonical_recording()
    );
    Ok(())
}

#[test]
fn editing_one_artboard_preserves_another_artboards_hot_state_and_renderer() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard_a, rectangle_a, color_a, artboard_b, color_b), _) = scene.edit(|tx| {
        let artboard_a = tx.create_artboard(ArtboardSpec {
            name: "A".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape_a = tx.create(
            Parent::Artboard(artboard_a),
            NodeSpec::Shape(ShapeSpec {
                name: "A Card".into(),
                x: 50.0,
                y: 50.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let rectangle_a = tx.create(
            Parent::Object(shape_a),
            NodeSpec::Rectangle(RectangleSpec::new("A Rectangle", 80.0, 60.0)),
        )?;
        let fill_a = tx.create(
            Parent::Object(shape_a),
            NodeSpec::Fill(FillSpec {
                name: "A Fill".into(),
            }),
        )?;
        let color_a = tx.create(
            Parent::Object(fill_a),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "A Color".into(),
                color: 0xff112233,
            }),
        )?;

        let artboard_b = tx.create_artboard(ArtboardSpec {
            name: "B".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape_b = tx.create(
            Parent::Artboard(artboard_b),
            NodeSpec::Shape(ShapeSpec {
                name: "B Card".into(),
                x: 50.0,
                y: 50.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Object(shape_b),
            NodeSpec::Rectangle(RectangleSpec::new("B Rectangle", 80.0, 60.0)),
        )?;
        let fill_b = tx.create(
            Parent::Object(shape_b),
            NodeSpec::Fill(FillSpec {
                name: "B Fill".into(),
            }),
        )?;
        let color_b = tx.create(
            Parent::Object(fill_b),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "B Color".into(),
                color: 0xff112233,
            }),
        )?;
        Ok((artboard_a, rectangle_a, color_a, artboard_b, color_b))
    })?;

    let instance_a = scene.instantiate(artboard_a)?;
    let instance_b = scene.instantiate(artboard_b)?;
    let cursor_a = scene.cursor(instance_a, color_a, props::COLOR_VALUE)?;
    let cursor_b = scene.cursor(instance_b, color_b, props::COLOR_VALUE)?;
    assert!(scene.frame().set(cursor_b, 0xff445566)?);

    let mut factory_b = RecordingFactory::new();
    let mut cache_b = scene.new_draw_token(instance_b)?;
    let mut renderer_b = factory_b.make_renderer();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    factory_b.clear();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    let b_before = factory_b.stream();
    assert!(b_before.contains("ff445566"), "the hot color must render");

    let a_before = draw_stream(&mut scene, instance_a)?;
    scene.edit(|tx| {
        tx.set(rectangle_a, props::PATH_WIDTH, 72.0)?;
        Ok(())
    })?;

    assert_eq!(scene.frame().set(cursor_a, 0xff778899), Err(StaleCursor));
    assert_eq!(scene.frame().set(cursor_b, 0xff778899), Err(StaleCursor));
    assert_ne!(draw_stream(&mut scene, instance_a)?, a_before);

    factory_b.clear();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    assert_eq!(
        factory_b.stream(),
        b_before,
        "an unrelated artboard edit must preserve the live occurrence and its renderer members"
    );
    Ok(())
}

#[test]
fn failed_multi_artboard_materialization_publishes_nothing_before_a_valid_commit() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard_a, rectangle_a, color_a, artboard_b, rectangle_b, color_b), _) =
        scene.edit(|tx| {
            let (artboard_a, rectangle_a, color_a) = create_card(tx, "A", 0xff112233)?;
            let (artboard_b, rectangle_b, color_b) = create_card(tx, "B", 0xff223344)?;
            Ok((
                artboard_a,
                rectangle_a,
                color_a,
                artboard_b,
                rectangle_b,
                color_b,
            ))
        })?;
    let instance_a = scene.instantiate(artboard_a)?;
    let instance_b = scene.instantiate(artboard_b)?;
    let cursor_a = scene.cursor(instance_a, color_a, props::COLOR_VALUE)?;
    let cursor_b = scene.cursor(instance_b, color_b, props::COLOR_VALUE)?;
    assert!(scene.frame().set(cursor_a, 0xff556677)?);
    assert!(scene.frame().set(cursor_b, 0xff667788)?);

    let mut factory_a = RecordingFactory::new();
    let mut cache_a = scene.new_draw_token(instance_a)?;
    let mut renderer_a = factory_a.make_renderer();
    scene
        .frame()
        .draw(instance_a, &mut factory_a, &mut renderer_a, &mut cache_a)?;
    factory_a.clear();
    scene
        .frame()
        .draw(instance_a, &mut factory_a, &mut renderer_a, &mut cache_a)?;
    let a_before = factory_a.stream();

    let mut factory_b = RecordingFactory::new();
    let mut cache_b = scene.new_draw_token(instance_b)?;
    let mut renderer_b = factory_b.make_renderer();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    factory_b.clear();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    let b_before = factory_b.stream();
    let epoch_before = scene.epoch();

    let failed = scene.edit(|tx| {
        tx.set(rectangle_a, props::PATH_WIDTH, 72.0)?;
        tx.set(rectangle_b, props::PATH_WIDTH, 68.0)?;
        tx.create_artboard(ArtboardSpec {
            name: "Invalid".into(),
            width: f32::NAN,
            height: 100.0,
        })?;
        Ok(())
    });
    let failed = failed.expect_err("the third candidate must reject the scope");
    assert_eq!(
        failed.diagnostic().operation_index,
        2,
        "commit failure must identify the operation that introduced the invalid candidate"
    );
    assert_eq!(scene.epoch(), epoch_before);
    assert!(scene.frame().set(cursor_a, 0xff556677).is_ok());
    assert!(scene.frame().set(cursor_b, 0xff667788).is_ok());

    factory_a.clear();
    scene
        .frame()
        .draw(instance_a, &mut factory_a, &mut renderer_a, &mut cache_a)?;
    assert_eq!(factory_a.stream(), a_before);
    factory_b.clear();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    assert_eq!(factory_b.stream(), b_before);

    let (_, receipt) = scene.edit(|tx| {
        tx.set(color_a, props::COLOR_VALUE, 0xffaabbcc)?;
        tx.set(color_b, props::COLOR_VALUE, 0xffbbccdd)?;
        Ok(())
    })?;
    assert_eq!(receipt.epoch, scene.epoch());
    assert_ne!(scene.epoch(), epoch_before);
    assert_eq!(scene.frame().set(cursor_a, 0xff8899aa), Err(StaleCursor));
    assert_eq!(scene.frame().set(cursor_b, 0xff99aabb), Err(StaleCursor));

    factory_a.clear();
    scene
        .frame()
        .draw(instance_a, &mut factory_a, &mut renderer_a, &mut cache_a)?;
    assert_ne!(factory_a.stream(), a_before);
    factory_b.clear();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    assert_ne!(factory_b.stream(), b_before);

    // Rebuild an independent oracle with only the valid color edit. Exact stream equality proves
    // the failed width definitions never leaked into the next successful materialization.
    let mut expected = Scene::new();
    let ((expected_a, _, expected_color_a, expected_b, _, expected_color_b), _) =
        expected.edit(|tx| {
            let (artboard_a, rectangle_a, color_a) = create_card(tx, "A", 0xff112233)?;
            let (artboard_b, rectangle_b, color_b) = create_card(tx, "B", 0xff223344)?;
            Ok((
                artboard_a,
                rectangle_a,
                color_a,
                artboard_b,
                rectangle_b,
                color_b,
            ))
        })?;
    expected.edit(|tx| {
        tx.set(expected_color_a, props::COLOR_VALUE, 0xffaabbcc)?;
        tx.set(expected_color_b, props::COLOR_VALUE, 0xffbbccdd)?;
        Ok(())
    })?;
    let expected_instance_a = expected.instantiate(expected_a)?;
    let expected_instance_b = expected.instantiate(expected_b)?;
    assert_eq!(
        draw_stream(&mut scene, instance_a)?,
        draw_stream(&mut expected, expected_instance_a)?
    );
    assert_eq!(
        draw_stream(&mut scene, instance_b)?,
        draw_stream(&mut expected, expected_instance_b)?
    );
    Ok(())
}

#[test]
fn typed_clipping_shape_exports_and_draws_the_source_shape_path() -> Result<()> {
    let mut scene = Scene::new();
    let (artboard, _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Scroll viewport".into(),
            width: 200.0,
            height: 100.0,
        })?;
        let clip_source = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Viewport clip source".into(),
                x: 10.0,
                y: 20.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Object(clip_source),
            NodeSpec::Rectangle(RectangleSpec::new("Viewport path", 40.0, 30.0)),
        )?;

        let content = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Scroll content".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Object(content),
            NodeSpec::Rectangle(RectangleSpec::new("Content path", 100.0, 80.0)),
        )?;
        let fill = tx.create(
            Parent::Object(content),
            NodeSpec::Fill(FillSpec {
                name: "Content fill".into(),
            }),
        )?;
        tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Content color".into(),
                color: 0xff11_2233,
            }),
        )?;
        tx.create(
            Parent::Object(content),
            NodeSpec::ClippingShape(ClippingShapeSpec {
                name: "Viewport clip".into(),
                source: clip_source,
                fill_rule: SceneClippingFillRule::EvenOdd,
                is_visible: true,
            }),
        )?;
        Ok(artboard)
    })?;

    let records = scene.export_records();
    let clipping = records
        .records()
        .iter()
        .find(|record| record.kind == ExportedObjectKind::ClippingShape)
        .expect("typed scene exports its clipping-shape child");
    assert_eq!(
        clipping.properties,
        vec![
            ExportedProperty::ComponentName("Viewport clip".into()),
            ExportedProperty::ParentId(3),
            ExportedProperty::ClippingShapeSourceId(1),
            ExportedProperty::ClippingShapeFillRule(SceneClippingFillRule::EvenOdd),
        ],
        "the exact Rive record keeps stable local source/parent paths and omits isVisible=true"
    );

    let instance = scene.instantiate(artboard)?;
    let stream = draw_stream(&mut scene, instance)?;
    assert!(
        stream.lines().any(|line| {
            line.starts_with("clipPath ")
                && line.contains("fillRule=1")
                && line.contains("points=[(-10,5),(30,5),(30,35),(-10,35),(-10,5)]")
        }),
        "the authored source Shape must become the runtime clipping path: {stream}"
    );
    Ok(())
}

#[test]
fn clipping_shape_rejects_a_source_that_is_not_a_shape() -> Result<()> {
    let mut scene = Scene::new();
    let mut wrong_source = None;
    let mut clipping = None;
    let error = scene
        .edit(|tx| {
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Invalid clip".into(),
                width: 100.0,
                height: 100.0,
            })?;
            let content = tx.create(
                Parent::Artboard(artboard),
                NodeSpec::Shape(ShapeSpec {
                    name: "Content".into(),
                    x: 0.0,
                    y: 0.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                }),
            )?;
            let rectangle = tx.create(
                Parent::Object(content),
                NodeSpec::Rectangle(RectangleSpec::new("Not a source Shape", 40.0, 30.0)),
            )?;
            wrong_source = Some(rectangle);
            clipping = Some(tx.create(
                Parent::Object(content),
                NodeSpec::ClippingShape(ClippingShapeSpec {
                    name: "Clip".into(),
                    source: rectangle,
                    fill_rule: SceneClippingFillRule::NonZero,
                    is_visible: true,
                }),
            )?);
            Ok(())
        })
        .expect_err("a clipping source must be a Shape whose paths can be composed");

    assert_eq!(error.kind(), EditErrorKind::CommitRejected);
    assert_eq!(
        error.diagnostic().involved_ids,
        vec![
            EditId::Object(clipping.expect("the clipping shape was allocated")),
            EditId::Object(wrong_source.expect("the wrong source was allocated")),
        ]
    );
    assert_eq!(
        error.diagnostic().reason,
        EditReason::InvalidReference {
            expected: NodeKind::Shape,
            actual: Some(NodeKind::Rectangle),
        }
    );
    assert_eq!(scene.epoch(), StructureEpoch::INITIAL);
    assert_eq!(scene.export_records().records().len(), 1);
    Ok(())
}

#[test]
fn clipping_shape_requires_an_existing_node_parent() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, source), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Viewport".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let source = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Clip source".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        Ok((artboard, source))
    })?;
    let epoch_before = scene.epoch();
    let records_before = scene.export_records();

    let error = scene
        .edit(|tx| {
            tx.create(
                Parent::Artboard(artboard),
                NodeSpec::ClippingShape(ClippingShapeSpec {
                    name: "Unparented clip".into(),
                    source,
                    fill_rule: SceneClippingFillRule::NonZero,
                    is_visible: true,
                }),
            )?;
            Ok(())
        })
        .expect_err("a ClippingShape is a component of the node subtree it clips");

    assert_eq!(error.kind(), EditErrorKind::Aborted);
    assert_eq!(
        error.diagnostic().involved_ids,
        vec![EditId::Artboard(artboard)]
    );
    assert_eq!(
        error.diagnostic().reason,
        EditReason::InvalidParent {
            parent: None,
            child: NodeKind::ClippingShape,
        }
    );
    assert_eq!(scene.epoch(), epoch_before);
    assert_eq!(scene.export_records(), records_before);
    Ok(())
}

#[test]
fn clipping_shape_rejects_a_source_from_another_artboard() -> Result<()> {
    let mut scene = Scene::new();
    let mut clipping = None;
    let mut foreign_source = None;
    let mut source_artboard = None;
    let mut target_artboard = None;
    let error = scene
        .edit(|tx| {
            let target = tx.create_artboard(ArtboardSpec {
                name: "Target".into(),
                width: 100.0,
                height: 100.0,
            })?;
            target_artboard = Some(target);
            let source = tx.create_artboard(ArtboardSpec {
                name: "Source".into(),
                width: 100.0,
                height: 100.0,
            })?;
            source_artboard = Some(source);
            let content = tx.create(
                Parent::Artboard(target),
                NodeSpec::Shape(ShapeSpec {
                    name: "Content".into(),
                    x: 0.0,
                    y: 0.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                }),
            )?;
            let source_shape = tx.create(
                Parent::Artboard(source),
                NodeSpec::Shape(ShapeSpec {
                    name: "Foreign source".into(),
                    x: 0.0,
                    y: 0.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                }),
            )?;
            foreign_source = Some(source_shape);
            clipping = Some(tx.create(
                Parent::Object(content),
                NodeSpec::ClippingShape(ClippingShapeSpec {
                    name: "Clip".into(),
                    source: source_shape,
                    fill_rule: SceneClippingFillRule::NonZero,
                    is_visible: true,
                }),
            )?);
            Ok(())
        })
        .expect_err("runtime-local sourceId values cannot cross an artboard boundary");

    assert_eq!(error.kind(), EditErrorKind::CommitRejected);
    assert_eq!(
        error.diagnostic().involved_ids,
        vec![
            EditId::Object(clipping.expect("the clipping shape was allocated")),
            EditId::Object(foreign_source.expect("the foreign source was allocated")),
        ]
    );
    assert_eq!(
        error.diagnostic().reason,
        EditReason::CrossArtboardReference {
            source: target_artboard.expect("the target artboard was allocated"),
            target: source_artboard.expect("the source artboard was allocated"),
        }
    );
    assert_eq!(scene.epoch(), StructureEpoch::INITIAL);
    Ok(())
}

#[test]
fn clipping_shape_remove_and_restore_preserves_identity_and_protects_its_source() -> Result<()> {
    let mut scene = Scene::new();
    let ((source, clipping), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Scroll viewport".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let source = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Clip source".into(),
                x: 50.0,
                y: 50.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Object(source),
            NodeSpec::Rectangle(RectangleSpec::new("Clip path", 100.0, 100.0)),
        )?;
        let content = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Content".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let clipping = tx.create(
            Parent::Object(content),
            NodeSpec::ClippingShape(ClippingShapeSpec {
                name: "Clip".into(),
                source,
                fill_rule: SceneClippingFillRule::EvenOdd,
                is_visible: false,
            }),
        )?;
        Ok((source, clipping))
    })?;
    let records_before = scene.export_records();
    let clipping_record = records_before
        .records()
        .iter()
        .find(|record| record.kind == ExportedObjectKind::ClippingShape)
        .expect("the clipping shape is exported before removal");
    assert_eq!(
        clipping_record.properties,
        vec![
            ExportedProperty::ComponentName("Clip".into()),
            ExportedProperty::ParentId(3),
            ExportedProperty::ClippingShapeSourceId(1),
            ExportedProperty::ClippingShapeFillRule(SceneClippingFillRule::EvenOdd),
            ExportedProperty::ClippingShapeIsVisible(false),
        ]
    );

    let (removed, _) = scene.edit(|tx| tx.remove(clipping))?;
    assert!(
        scene
            .export_records()
            .records()
            .iter()
            .all(|record| record.kind != ExportedObjectKind::ClippingShape)
    );
    let (restored, _) = scene.edit(|tx| tx.restore(removed))?;

    assert_eq!(restored, clipping);
    assert_eq!(scene.export_records(), records_before);

    let epoch_before_rejected_remove = scene.epoch();
    let error = scene
        .edit(|tx| {
            tx.remove(source)?;
            Ok(())
        })
        .expect_err("removing a referenced clip source must reject atomically");
    assert_eq!(error.kind(), EditErrorKind::CommitRejected);
    assert_eq!(
        error.diagnostic().involved_ids,
        vec![EditId::Object(clipping), EditId::Object(source)]
    );
    assert_eq!(error.diagnostic().reason, EditReason::UnknownObject);
    assert_eq!(scene.epoch(), epoch_before_rejected_remove);
    assert_eq!(scene.export_records(), records_before);
    Ok(())
}
