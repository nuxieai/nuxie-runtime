use anyhow::Result;
use nuxie::{
    Aabb, ArtboardId, ArtboardSpec, ChildIndex, DashPathSpec, DashSpec, EditAbort, EditErrorKind,
    EditId, EditReason, ExportedObjectKind, ExportedProperty, FillSpec, FontAssetId, FontAssetSpec,
    NodeKind, NodeSpec, ObjectId, Parent, PropValueKind, RecordingFactory, RectangleCornerRadii,
    RectangleSpec, ResolveError, Scene, SceneEvent, SceneStrokeCap, SceneStrokeJoin,
    SceneTextAlign, SceneTextOverflow, SceneTextSizing, SceneTextWrap, SceneTx, ShapeSpec,
    SolidColorSpec, StaleCursor, StrokeSpec, StructureEpoch, TextSpec, TextStylePaintSpec,
    TextValueRunSpec, Vec2D, props,
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
    let mut factory = RecordingFactory::new();
    let mut cache = scene.new_render_cache(instance, &mut factory)?;
    let mut renderer = factory.make_renderer();
    scene
        .frame()
        .draw(instance, &mut factory, &mut renderer, &mut cache)?;
    Ok(factory.stream())
}

fn canonical_draw_stream(scene: &mut Scene, instance: nuxie::InstanceId) -> Result<String> {
    let mut factory = RecordingFactory::new();
    let mut cache = scene.new_render_cache(instance, &mut factory)?;
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
    const LOCAL_CARET_HEIGHT: f32 = 22.721_35;
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
    let mut held_cache = scene.new_render_cache(instance, &mut held_factory)?;
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
    let mut cache = scene.new_render_cache(instance, &mut factory)?;
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
    let mut cache_a = scene.new_render_cache(instance_a, &mut factory_a)?;
    let mut renderer_a = factory_a.make_renderer();
    scene
        .frame()
        .draw(instance_a, &mut factory_a, &mut renderer_a, &mut cache_a)?;
    let mut factory_b = RecordingFactory::new();
    let mut cache_b = scene.new_render_cache(instance_b, &mut factory_b)?;
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
fn reorder_artboards_changes_export_order_without_remounting_instances_or_caches() -> Result<()> {
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
    let mut cache_a = scene.new_render_cache(instance_a, &mut factory_a)?;
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
    let mut cache_b = scene.new_render_cache(instance_b, &mut factory_b)?;
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
    let mut cache_a = scene.new_render_cache(instance_a, &mut factory_a)?;
    let mut renderer_a = factory_a.make_renderer();
    scene
        .frame()
        .draw(instance_a, &mut factory_a, &mut renderer_a, &mut cache_a)?;
    let mut factory_b = RecordingFactory::new();
    let mut cache_b = scene.new_render_cache(instance_b, &mut factory_b)?;
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
        scene.new_render_cache(removed_instance, &mut RecordingFactory::new()),
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
    let mut cache_a = scene.new_render_cache(instance_a, &mut factory_a)?;
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
    let mut cache_b = scene.new_render_cache(instance_b, &mut factory_b)?;
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
        "geometry caches must follow the structurally remounted runtime"
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
fn removing_from_one_artboard_preserves_another_artboards_hot_state_and_held_cache() -> Result<()> {
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
    let mut cache_b = scene.new_render_cache(instance_b, &mut factory_b)?;
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
        "the untouched artboard must retain both its live instance and held cache"
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
fn render_cache_held_across_a_structural_remount_matches_a_fresh_cache() -> Result<()> {
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
    let mut held_cache = scene.new_render_cache(instance, &mut original_factory)?;
    let mut original_renderer = original_factory.make_renderer();
    scene.frame().draw(
        instance,
        &mut original_factory,
        &mut original_renderer,
        &mut held_cache,
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
        &mut held_cache,
    )?;

    let mut fresh_factory = RecordingFactory::new();
    let mut fresh_cache = scene.new_render_cache(instance, &mut fresh_factory)?;
    let mut fresh_renderer = fresh_factory.make_renderer();
    scene.frame().draw(
        instance,
        &mut fresh_factory,
        &mut fresh_renderer,
        &mut fresh_cache,
    )?;

    assert_eq!(refreshed_factory.stream(), fresh_factory.stream());
    Ok(())
}

#[test]
fn editing_one_artboard_preserves_another_artboards_hot_state_and_held_cache() -> Result<()> {
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
    let mut cache_b = scene.new_render_cache(instance_b, &mut factory_b)?;
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
        "an unrelated artboard edit must preserve the live instance and its held cache"
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
    let mut cache_a = scene.new_render_cache(instance_a, &mut factory_a)?;
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
    let mut cache_b = scene.new_render_cache(instance_b, &mut factory_b)?;
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
