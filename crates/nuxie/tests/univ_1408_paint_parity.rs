use anyhow::{Context, Result};
use nuxie::{
    ArtboardSpec, DashPathSpec, DashSpec, FillSpec, FontAssetSpec, LayoutComponentSpec,
    LayoutComponentStyleSpec, NodeSpec, Parent, RecordingFactory, RectangleSpec, Scene,
    SceneLayoutPosition, SceneLayoutUnit, SceneStrokeCap, SceneStrokeJoin, SceneTextAlign,
    SceneTextOverflow, SceneTextSizing, SceneTextWrap, ShapeSpec, SolidColorSpec, StrokeSpec,
    TextSpec, TextStylePaintSpec, TextValueRunSpec,
};
use nuxie_render_api::{PathVerb, RenderPaintStyle, StrokeCap, StrokeJoin};
use nuxie_render_stream::{Command as RenderCommand, RenderStream};
use std::path::{Path, PathBuf};

const BORDER_COLOR: u32 = 0xff0f_172a;
const BADGE_COLOR: u32 = 0xffc8_a896;
const BADGE_TEXT_COLOR: u32 = 0xff2a_2a2a;

#[derive(Debug)]
struct BorderDash {
    cap: SceneStrokeCap,
    offset: f32,
    runs: Vec<f32>,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture_font_bytes() -> Result<Vec<u8>> {
    std::fs::read(repo_root().join("fixtures/command_queue/OpenSans-Italic.ttf"))
        .context("read fixtures/command_queue/OpenSans-Italic.ttf")
}

fn render_scene(scene: &mut Scene, artboard: nuxie::ArtboardId) -> Result<RenderStream> {
    let instance = scene.instantiate(artboard)?;
    let mut factory = RecordingFactory::new();
    let mut cache = scene.new_draw_token(instance)?;
    let mut renderer = factory.make_renderer();
    scene
        .frame()
        .draw(instance, &mut factory, &mut renderer, &mut cache)?;
    RenderStream::parse(&factory.stream()).context("parse recording render stream")
}

fn create_border_scene(
    width: f32,
    height: f32,
    border_width: f32,
    dash: Option<BorderDash>,
) -> Result<(Scene, nuxie::ArtboardId)> {
    let mut scene = Scene::new();
    let (artboard, _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "UNIV-1408 border".into(),
            width,
            height,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Border Shape".into(),
                x: width / 2.0,
                y: height / 2.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new(
                "Border centerline",
                (width - border_width).max(0.0),
                (height - border_width).max(0.0),
            )),
        )?;
        let stroke = tx.create(
            Parent::Object(shape),
            NodeSpec::Stroke(StrokeSpec {
                name: "Border Stroke".into(),
                thickness: border_width,
                cap: dash
                    .as_ref()
                    .map_or(SceneStrokeCap::Butt, |value| value.cap),
                join: SceneStrokeJoin::Miter,
                transform_affects_stroke: true,
            }),
        )?;
        tx.create(
            Parent::Object(stroke),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Border Color".into(),
                color: BORDER_COLOR,
            }),
        )?;
        if let Some(dash) = dash {
            let dash_path = tx.create(
                Parent::Object(stroke),
                NodeSpec::DashPath(DashPathSpec {
                    name: "Border Dash Path".into(),
                    offset: dash.offset,
                    offset_is_percentage: false,
                }),
            )?;
            for (index, length) in dash.runs.into_iter().enumerate() {
                tx.create(
                    Parent::Object(dash_path),
                    NodeSpec::Dash(DashSpec {
                        name: format!("Border Dash {index}"),
                        length,
                        length_is_percentage: true,
                    }),
                )?;
            }
        }
        Ok(artboard)
    })?;
    Ok((scene, artboard))
}

/// Mirrors the zero-radius branch of the product compiler's
/// `fitted_border_dash_runs`. The compiler emits each fitted run as a
/// percentage of the complete centerline perimeter.
fn fitted_border_dash(
    target_dash: f32,
    target_gap: f32,
    width: f32,
    height: f32,
    overhang: f32,
    round_cap: bool,
) -> BorderDash {
    let sides = [width, height, width, height];
    let mut runs = Vec::new();
    if target_dash <= overhang {
        let step = target_dash + target_gap;
        let intervals = sides.map(|side| ((side / step).round() as usize).max(1));
        let gaps = std::array::from_fn::<_, 4, _>(|index| {
            sides[index] / intervals[index] as f32 - target_dash
        });
        runs.push(target_dash);
        let interval_count = intervals.iter().sum::<usize>();
        let mut emitted_intervals = 0usize;
        for side in 0..4 {
            for _ in 0..intervals[side] {
                runs.push(gaps[side]);
                emitted_intervals += 1;
                if emitted_intervals < interval_count {
                    runs.push(target_dash);
                }
            }
        }
    } else {
        let corner_reach = target_dash - overhang;
        let mut items = Vec::new();
        for side in sides {
            let fit_length = side + 2.0 * overhang;
            let count = (((fit_length - target_dash) / (target_dash + target_gap)).round() as i64
                + 1)
            .max(1);
            let gap = (fit_length - count as f32 * target_dash) / (count - 1) as f32;
            items.push((true, corner_reach));
            items.push((false, gap));
            for _ in 0..(count - 2) {
                items.push((true, target_dash));
                items.push((false, gap));
            }
            items.push((true, corner_reach));
        }
        let mut on_run = true;
        for (on, length) in items {
            match runs.last_mut() {
                Some(last) if on == on_run => *last += length,
                _ => {
                    if !runs.is_empty() {
                        on_run = !on_run;
                    }
                    assert_eq!(on, on_run);
                    runs.push(length);
                }
            }
        }
    }

    let perimeter = runs.iter().map(|run| f64::from(*run)).sum::<f64>();
    let mut emitted = 0.0f64;
    for index in 0..runs.len() {
        runs[index] = if index == runs.len() - 1 {
            (1.0 - emitted).max(0.0) as f32
        } else {
            let normalized = (f64::from(runs[index]) / perimeter).max(0.0);
            emitted += normalized;
            normalized as f32
        };
    }
    BorderDash {
        cap: if round_cap {
            SceneStrokeCap::Round
        } else {
            SceneStrokeCap::Butt
        },
        offset: if round_cap { -target_dash / 2.0 } else { 0.0 },
        runs,
    }
}

fn create_text_scene(line_height: f32) -> Result<(Scene, nuxie::ArtboardId)> {
    let mut scene = Scene::new();
    let font_bytes = fixture_font_bytes()?;
    let (artboard, _) = scene.edit(|tx| {
        let font = tx.create_font_asset(FontAssetSpec {
            name: "Open Sans Italic fixture".into(),
            bytes: font_bytes,
        })?;
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "UNIV-1408 tight line height".into(),
            width: 180.0,
            height: 40.0,
        })?;
        let text = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Text(TextSpec {
                name: "Tight Text".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                sizing: SceneTextSizing::Fixed,
                width: 180.0,
                height: 40.0,
                align: SceneTextAlign::Left,
                wrap: SceneTextWrap::NoWrap,
                overflow: SceneTextOverflow::Visible,
            }),
        )?;
        let style = tx.create(
            Parent::Object(text),
            NodeSpec::TextStylePaint(TextStylePaintSpec {
                name: "Tight Style".into(),
                font_size: 30.0,
                line_height,
                letter_spacing: 0.0,
                font,
            }),
        )?;
        let fill = tx.create(
            Parent::Object(style),
            NodeSpec::Fill(FillSpec {
                name: "Tight Fill".into(),
            }),
        )?;
        tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Tight Color".into(),
                color: BORDER_COLOR,
            }),
        )?;
        tx.create(
            Parent::Object(text),
            NodeSpec::TextValueRun(TextValueRunSpec {
                name: "Tight Run".into(),
                text: "Tight".into(),
                style,
            }),
        )?;
        Ok(artboard)
    })?;
    Ok((scene, artboard))
}

fn create_absolute_badge_scene() -> Result<(Scene, nuxie::ArtboardId)> {
    let mut scene = Scene::new();
    let font_bytes = fixture_font_bytes()?;
    let (artboard, _) = scene.edit(|tx| {
        let font = tx.create_font_asset(FontAssetSpec {
            name: "Open Sans Italic fixture".into(),
            bytes: font_bytes,
        })?;
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "UNIV-1408 absolute badge".into(),
            width: 390.0,
            height: 188.0,
        })?;
        let parent = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::LayoutComponent(LayoutComponentSpec {
                name: "Plan Option".into(),
                x: 20.0,
                y: 20.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                clip: false,
                width: 350.0,
                height: 80.0,
                fractional_width: 1.0,
                fractional_height: 1.0,
                style: LayoutComponentStyleSpec::default(),
            }),
        )?;
        let badge = tx.create(
            Parent::Object(parent),
            NodeSpec::LayoutComponent(LayoutComponentSpec {
                name: "Badge".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                clip: false,
                width: 140.0,
                height: 22.0,
                fractional_width: 1.0,
                fractional_height: 1.0,
                style: LayoutComponentStyleSpec {
                    position_type: SceneLayoutPosition::Absolute,
                    position_right: 16.0,
                    position_top: -11.0,
                    position_right_units: SceneLayoutUnit::Point,
                    position_top_units: SceneLayoutUnit::Point,
                    ..LayoutComponentStyleSpec::default()
                },
            }),
        )?;
        let shape = tx.create(
            Parent::Object(badge),
            NodeSpec::Shape(ShapeSpec {
                name: "Badge Shape".into(),
                x: 70.0,
                y: 11.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new("Badge Rectangle", 140.0, 22.0)),
        )?;
        let shape_fill = tx.create(
            Parent::Object(shape),
            NodeSpec::Fill(FillSpec {
                name: "Badge Fill".into(),
            }),
        )?;
        tx.create(
            Parent::Object(shape_fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Badge Color".into(),
                color: BADGE_COLOR,
            }),
        )?;
        let text = tx.create(
            Parent::Object(badge),
            NodeSpec::Text(TextSpec {
                name: "Badge Label".into(),
                x: 10.0,
                y: 3.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                sizing: SceneTextSizing::Fixed,
                width: 120.0,
                height: 16.0,
                align: SceneTextAlign::Left,
                wrap: SceneTextWrap::NoWrap,
                overflow: SceneTextOverflow::Visible,
            }),
        )?;
        let style = tx.create(
            Parent::Object(text),
            NodeSpec::TextStylePaint(TextStylePaintSpec {
                name: "Badge Label Style".into(),
                font_size: 11.0,
                line_height: 16.0,
                letter_spacing: 0.2,
                font,
            }),
        )?;
        let text_fill = tx.create(
            Parent::Object(style),
            NodeSpec::Fill(FillSpec {
                name: "Badge Label Fill".into(),
            }),
        )?;
        tx.create(
            Parent::Object(text_fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Badge Label Color".into(),
                color: BADGE_TEXT_COLOR,
            }),
        )?;
        tx.create(
            Parent::Object(text),
            NodeSpec::TextValueRun(TextValueRunSpec {
                name: "Badge Label Run".into(),
                text: "BEST VALUE".into(),
                style,
            }),
        )?;
        Ok(artboard)
    })?;
    Ok((scene, artboard))
}

fn recorded_draw(
    stream: &RenderStream,
    color: u32,
) -> Result<(
    [f32; 6],
    &nuxie_render_stream::Path,
    &nuxie_render_stream::Paint,
)> {
    let mut transform = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    for command in stream.frames.iter().flat_map(|frame| &frame.commands) {
        match command {
            RenderCommand::Transform(matrix) => transform = matrix.0,
            RenderCommand::DrawPath { path, paint } if paint.color == color => {
                return Ok((transform, path, paint));
            }
            _ => {}
        }
    }
    anyhow::bail!("render stream has no DrawPath with color {color:#010x}")
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-4,
        "expected {expected}, got {actual}"
    );
}

fn assert_solid_border(border_width: f32) -> Result<()> {
    let (mut scene, artboard) = create_border_scene(96.0, 64.0, border_width, None)?;
    let stream = render_scene(&mut scene, artboard)?;
    let (transform, path, paint) = recorded_draw(&stream, BORDER_COLOR)?;
    let bounds = path.raw_path.precise_bounds().context("border bounds")?;
    println!(
        "thickness={} transform={transform:?} path_bounds={bounds:?}",
        paint.thickness
    );

    assert_eq!(paint.style, RenderPaintStyle::Stroke);
    assert_close(paint.thickness, border_width);
    assert_eq!(paint.cap, StrokeCap::Butt);
    assert_eq!(paint.join, StrokeJoin::Miter);
    assert_eq!(transform, [1.0, 0.0, 0.0, 1.0, 48.0, 32.0]);
    assert_close(bounds.max_x - bounds.min_x, 96.0 - border_width);
    assert_close(bounds.max_y - bounds.min_y, 64.0 - border_width);
    assert_eq!(
        path.raw_path.verbs(),
        &[
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Close,
        ]
    );
    Ok(())
}

#[test]
#[ignore = "UNIV-1408 border: reported 8px stroke paints as about 4px"]
fn border_transparent() -> Result<()> {
    // The current stream refutes both proposed causes: the full thickness is
    // present and the path is the compiler-authored centerline rectangle.
    assert_solid_border(8.0)
}

#[test]
#[ignore = "UNIV-1408 border: reported 4px stroke paints as 2px"]
fn border_basic() -> Result<()> {
    assert_solid_border(4.0)
}

#[test]
#[ignore = "UNIV-1408 border: reported dashed run differs from DOM"]
fn border_dashed() -> Result<()> {
    let dash = fitted_border_dash(8.0, 4.0, 92.0, 60.0, 2.0, false);
    let (mut scene, artboard) = create_border_scene(96.0, 64.0, 4.0, Some(dash))?;
    let stream = render_scene(&mut scene, artboard)?;
    let (transform, path, paint) = recorded_draw(&stream, BORDER_COLOR)?;
    let bounds = path.raw_path.precise_bounds().context("dashed bounds")?;
    let moves = path
        .raw_path
        .verbs()
        .iter()
        .filter(|verb| **verb == PathVerb::Move)
        .count();
    println!(
        "thickness={} transform={transform:?} path_bounds={bounds:?} on_contours={moves} first_points={:?}",
        paint.thickness,
        &path.raw_path.points()[..2]
    );

    // Pinned C++ `src/shapes/paint/dash_path.cpp:39-98` consumes the
    // alternating percentage runs in order and walks the 304px centerline.
    assert_eq!(paint.style, RenderPaintStyle::Stroke);
    assert_close(paint.thickness, 4.0);
    assert_eq!(paint.cap, StrokeCap::Butt);
    assert_eq!(paint.join, StrokeJoin::Miter);
    assert_eq!(transform, [1.0, 0.0, 0.0, 1.0, 48.0, 32.0]);
    assert_eq!(moves, 25);
    assert_close(path.raw_path.points()[0].x, -46.0);
    assert_close(path.raw_path.points()[1].x, -40.0);
    Ok(())
}

#[test]
#[ignore = "UNIV-1408 border: reported dotted run differs from DOM"]
fn border_dotted() -> Result<()> {
    // The product uses a non-zero 0.25px ON run. A zero run is not faithful:
    // it is omitted by binary authoring and would also make C++ return no path.
    let dash = fitted_border_dash(0.25, 7.75, 92.0, 60.0, 2.0, true);
    assert_eq!(dash.runs.len(), 80);
    let (mut scene, artboard) = create_border_scene(96.0, 64.0, 4.0, Some(dash))?;
    let stream = render_scene(&mut scene, artboard)?;
    let (transform, path, paint) = recorded_draw(&stream, BORDER_COLOR)?;
    let bounds = path.raw_path.precise_bounds().context("dotted bounds")?;
    let moves = path
        .raw_path
        .verbs()
        .iter()
        .filter(|verb| **verb == PathVerb::Move)
        .count();
    println!(
        "thickness={} transform={transform:?} path_bounds={bounds:?} on_contours={moves} first_points={:?}",
        paint.thickness,
        &path.raw_path.points()[..2]
    );

    assert_eq!(paint.style, RenderPaintStyle::Stroke);
    assert_close(paint.thickness, 4.0);
    assert_eq!(paint.cap, StrokeCap::Round);
    assert_eq!(paint.join, StrokeJoin::Miter);
    assert_eq!(transform, [1.0, 0.0, 0.0, 1.0, 48.0, 32.0]);
    assert_eq!(moves, 40);
    assert!(!path.raw_path.points().is_empty());
    Ok(())
}

#[test]
#[ignore = "UNIV-1408 text: reported 1px line-height glyphs shift vertically"]
fn text_tight_line_height() -> Result<()> {
    let (mut tight_scene, tight_artboard) = create_text_scene(1.0)?;
    let tight_stream = render_scene(&mut tight_scene, tight_artboard)?;
    let (tight_transform, tight_path, tight_paint) = recorded_draw(&tight_stream, BORDER_COLOR)?;
    let tight = tight_path
        .raw_path
        .precise_bounds()
        .context("tight text bounds")?;

    let (mut reference_scene, reference_artboard) = create_text_scene(40.0)?;
    let reference_stream = render_scene(&mut reference_scene, reference_artboard)?;
    let (_, reference_path, _) = recorded_draw(&reference_stream, BORDER_COLOR)?;
    let reference = reference_path
        .raw_path
        .precise_bounds()
        .context("reference text bounds")?;
    println!("transform={tight_transform:?} glyph_bounds={tight:?}");

    // Pinned C++ `src/text/line_breaker.cpp:77-88` uses realAscent for the
    // first line, not the explicit lineHeight-adjusted ascent. Accordingly,
    // changing 1px to 40px does not move this single first line in either C++
    // or the Rust port.
    assert_eq!(tight_paint.style, RenderPaintStyle::Fill);
    assert_eq!(tight_transform, [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    assert_close(tight.min_y, 9.272461);
    assert_close(tight.max_y, 39.27246);
    assert_close(tight.min_y, reference.min_y);
    assert_close(tight.max_y, reference.max_y);
    Ok(())
}

#[test]
#[ignore = "UNIV-1408 layout paint: reported badge fill shifts below its label"]
fn absolute_badge() -> Result<()> {
    let (mut scene, artboard) = create_absolute_badge_scene()?;
    let stream = render_scene(&mut scene, artboard)?;
    let (fill_transform, fill_path, _) = recorded_draw(&stream, BADGE_COLOR)?;
    let (label_transform, label_path, _) = recorded_draw(&stream, BADGE_TEXT_COLOR)?;
    let fill_bounds = fill_path
        .raw_path
        .precise_bounds()
        .context("badge bounds")?;
    let label_bounds = label_path
        .raw_path
        .precise_bounds()
        .context("badge label bounds")?;
    let fill_origin = [
        fill_transform[4] + fill_bounds.min_x,
        fill_transform[5] + fill_bounds.min_y,
    ];
    let label_owner_origin = [label_transform[4] - 10.0, label_transform[5] - 3.0];
    println!(
        "fill_transform={fill_transform:?} fill_bounds={fill_bounds:?} label_transform={label_transform:?} label_bounds={label_bounds:?}"
    );

    // The shape starts at the badge's local (0, 0); the label is authored at
    // local (10, 3). Removing those local offsets yields the same owner origin.
    assert_eq!(fill_origin, [194.0, -11.0]);
    assert_eq!(label_owner_origin, fill_origin);
    Ok(())
}
