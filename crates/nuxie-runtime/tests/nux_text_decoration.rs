//! TextStyleDecoration (Nuxie type 60002) draws underlines and line-throughs
//! into its style's glyph path, one rectangle per decorated run per drawn
//! line. The expected geometry comes from the fixture font's own `head`,
//! `post` and `OS/2` tables, read here without the runtime.
use nuxie_binary::{
    FixtureProperty, FixtureRecord, FixtureValue, RuntimeFile, encode_runtime_file,
};
use nuxie_render_api::{NullRenderer, PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    math::{path_types::PathVerb, vec2d::Vec2D},
    text::{text::Text, text_style_paint::TextStylePaint},
};
use nuxie_runtime::text_decoration::{LINE_THROUGH, TextStyleDecoration, UNDERLINE};
use nuxie_runtime::{Artboard, CoreHandle, File, RuntimeFactoryHandle};

const FONT: &[u8] = include_bytes!("../../../fixtures/fonts/roboto-input.ttf");
const FONT_SIZE: f32 = 20.0;

fn property(owner: &str, name: &str, value: FixtureValue) -> FixtureProperty {
    let def = nuxie_schema::definition_by_name(owner).unwrap();
    let key = std::iter::once(def.name)
        .chain(def.ancestors.iter().copied())
        .filter_map(nuxie_schema::definition_by_name)
        .flat_map(|d| d.properties)
        .find(|p| p.name == name)
        .unwrap_or_else(|| panic!("{owner}.{name}"))
        .key
        .int;
    FixtureProperty { key, value }
}

fn record(owner: &str, properties: Vec<(&str, FixtureValue)>) -> FixtureRecord {
    FixtureRecord {
        type_key: nuxie_schema::definition_by_name(owner)
            .unwrap()
            .type_key
            .int,
        properties: properties
            .into_iter()
            .map(|(name, value)| property(owner, name, value))
            .collect(),
    }
}

fn style(parent: u64) -> [FixtureRecord; 3] {
    [
        record(
            "TextStylePaint",
            vec![
                ("parentId", FixtureValue::Uint(parent)),
                ("fontSize", FixtureValue::Double(FONT_SIZE)),
                ("fontAssetId", FixtureValue::Uint(0)),
                ("lineHeight", FixtureValue::Double(24.0)),
            ],
        ),
        record("Fill", vec![]),
        record("SolidColor", vec![]),
    ]
}

/// A Text with a plain run and a second run whose style may carry a
/// decoration. Artboard-local ids: Text 1, plain style 2-4, decorated style
/// 5-7, decoration 8.
fn scene(line: Option<u32>, width: f32, plain: &str, decorated: &str) -> Vec<u8> {
    let mut records = vec![
        record("Backboard", vec![]),
        record("FontAsset", vec![("assetId", FixtureValue::Uint(0))]),
        record("FileAssetContents", vec![("bytes", FixtureValue::Bytes(FONT.to_vec()))]),
        record(
            "Artboard",
            vec![
                ("width", FixtureValue::Double(400.0)),
                ("height", FixtureValue::Double(400.0)),
            ],
        ),
        record(
            "Text",
            vec![
                ("parentId", FixtureValue::Uint(0)),
                ("sizingValue", FixtureValue::Uint(2)),
                ("width", FixtureValue::Double(width)),
                ("height", FixtureValue::Double(400.0)),
                ("wrapValue", FixtureValue::Uint(0)),
            ],
        ),
    ];
    for (offset, parent) in [(2, 1), (5, 1)] {
        let [paint, mut fill, mut color] = style(parent);
        fill.properties
            .push(property("Fill", "parentId", FixtureValue::Uint(offset)));
        color
            .properties
            .push(property("SolidColor", "parentId", FixtureValue::Uint(offset + 1)));
        records.extend([paint, fill, color]);
    }
    if let Some(line) = line {
        records.push(record(
            "TextStyleDecoration",
            vec![
                ("parentId", FixtureValue::Uint(5)),
                ("line", FixtureValue::Uint(u64::from(line))),
            ],
        ));
    }
    for (text, style) in [(plain, 2), (decorated, 5)] {
        records.push(record(
            "TextValueRun",
            vec![
                ("parentId", FixtureValue::Uint(1)),
                ("text", FixtureValue::String(text.into())),
                ("styleId", FixtureValue::Uint(style)),
            ],
        ));
    }
    encode_runtime_file(&RuntimeFile::from_fixture_records(records).unwrap()).unwrap()
}

struct Built {
    plain: (Vec<PathVerb>, Vec<Vec2D>),
    decorated: (Vec<PathVerb>, Vec<Vec2D>),
    baselines: Vec<f32>,
    decorations: usize,
}

fn build(bytes: &[u8]) -> Built {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("decorated text imports");
    // An instance clones every object, so this also proves the clone.
    let board = file.with_file(File::artboard_default).unwrap();
    let root = board.core_handle();
    Artboard::advance_handle(
        &root,
        0.0,
        AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
    );
    let styles = board.with_artboard(|b| b.find_all_handles::<TextStylePaint>());
    assert_eq!(styles.len(), 2);
    let path = |style: &CoreHandle| {
        style
            .with_downcast_mut::<TextStylePaint, _>(|style| {
                let raw = style.local_path().raw_path();
                (raw.verbs().to_vec(), raw.points().to_vec())
            })
            .unwrap()
    };
    let text = board.with_artboard(|b| b.find_all_handles::<Text>())[0].clone();
    let baselines = text
        .with_downcast::<Text, _>(|t| t.ordered_lines().iter().map(|l| l.y()).collect())
        .unwrap();
    let decorations = board
        .with_artboard(|b| b.find_all_handles::<TextStyleDecoration>())
        .len();
    Artboard::draw_handle(&root, &mut NullRenderer::new());
    Built {
        plain: path(&styles[0]),
        decorated: path(&styles[1]),
        baselines,
        decorations,
    }
}

/// Axis-aligned rectangle contours of a path as (left, top, right, bottom).
fn rects(path: &(Vec<PathVerb>, Vec<Vec2D>)) -> Vec<[f32; 4]> {
    let (verbs, points) = path;
    let mut out = Vec::new();
    let mut point = 0;
    let mut contour: Vec<Vec2D> = Vec::new();
    let mut lines_only = true;
    let mut flush = |contour: &mut Vec<Vec2D>, lines_only: bool| {
        if lines_only && contour.len() == 4 {
            let xs: Vec<f32> = contour.iter().map(|p| p.x).collect();
            let ys: Vec<f32> = contour.iter().map(|p| p.y).collect();
            let (l, r) = (xs.iter().copied().fold(f32::MAX, f32::min), xs.iter().copied().fold(f32::MIN, f32::max));
            let (t, b) = (ys.iter().copied().fold(f32::MAX, f32::min), ys.iter().copied().fold(f32::MIN, f32::max));
            if contour.iter().all(|p| (p.x == l || p.x == r) && (p.y == t || p.y == b)) {
                out.push([l, t, r, b]);
            }
        }
        contour.clear();
    };
    for verb in verbs {
        match verb {
            PathVerb::Move => {
                flush(&mut contour, lines_only);
                lines_only = true;
                contour.push(points[point]);
                point += 1;
            }
            PathVerb::Line => {
                contour.push(points[point]);
                point += 1;
            }
            PathVerb::Quad => {
                lines_only = false;
                point += 2;
            }
            PathVerb::Cubic => {
                lines_only = false;
                point += 3;
            }
            PathVerb::Close => {}
        }
    }
    flush(&mut contour, lines_only);
    out
}

fn table(name: &[u8; 4]) -> &'static [u8] {
    let count = u16::from_be_bytes([FONT[4], FONT[5]]) as usize;
    for index in 0..count {
        let record = &FONT[12 + index * 16..28 + index * 16];
        if &record[..4] == name {
            let offset = u32::from_be_bytes(record[8..12].try_into().unwrap()) as usize;
            let length = u32::from_be_bytes(record[12..16].try_into().unwrap()) as usize;
            return &FONT[offset..offset + length];
        }
    }
    panic!("fixture font has no {name:?} table");
}

fn i16_at(table: &[u8], offset: usize) -> f32 {
    f32::from(i16::from_be_bytes([table[offset], table[offset + 1]]))
}

/// (top, bottom) of a decoration line below a baseline, from font units.
fn expected(baseline: f32, offset: f32, thickness: f32) -> (f32, f32) {
    let upem = f32::from(u16::from_be_bytes([table(b"head")[18], table(b"head")[19]]));
    let top = baseline - offset / upem * FONT_SIZE;
    (top, top + thickness / upem * FONT_SIZE)
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

#[test]
fn undecorated_styles_build_the_upstream_path() {
    let plain = build(&scene(None, 400.0, "aa ", "ab"));
    let underlined = build(&scene(Some(UNDERLINE), 400.0, "aa ", "ab"));
    assert_eq!(plain.decorations, 0);
    assert_eq!(underlined.decorations, 1);
    // The plain style's path is untouched by its neighbour's decoration, and
    // no glyph path carries a rectangle of its own.
    assert_eq!(plain.plain, underlined.plain);
    assert!(rects(&plain.decorated).is_empty());
    // The decoration adds a contour; every glyph contour is unchanged.
    let (verbs, points) = &underlined.decorated;
    assert!(verbs.len() > plain.decorated.0.len());
    assert_eq!(&verbs[..plain.decorated.0.len()], plain.decorated.0.as_slice());
    assert_eq!(&points[..plain.decorated.1.len()], plain.decorated.1.as_slice());
}

#[test]
fn underline_takes_the_font_post_metrics_under_the_run_only() {
    let built = build(&scene(Some(UNDERLINE), 400.0, "aa ", "ab"));
    let post = table(b"post");
    let (top, bottom) = expected(built.baselines[0], i16_at(post, 8), i16_at(post, 10));
    let found = rects(&built.decorated);
    assert_eq!(found.len(), 1, "{found:?}");
    let [left, t, right, b] = found[0];
    assert!(close(t, top) && close(b, bottom), "{found:?} vs {top}..{bottom}");
    // Under the decorated run, not the plain run before it.
    let plain_right = built
        .plain
        .1
        .iter()
        .map(|p| p.x)
        .fold(f32::MIN, f32::max);
    assert!(left > plain_right, "{left} starts after the plain run ({plain_right})");
    let decorated_right = built
        .decorated
        .1
        .iter()
        .filter(|p| p.y < top)
        .map(|p| p.x)
        .fold(f32::MIN, f32::max);
    assert!(right >= decorated_right - 1e-3);
}

#[test]
fn line_through_takes_the_font_os2_strikeout() {
    let built = build(&scene(Some(LINE_THROUGH), 400.0, "aa ", "ab"));
    let os2 = table(b"OS/2");
    let (top, bottom) = expected(built.baselines[0], i16_at(os2, 28), i16_at(os2, 26));
    let found = rects(&built.decorated);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(close(found[0][1], top) && close(found[0][3], bottom), "{found:?}");
    assert!(top < built.baselines[0], "a line-through sits above the baseline");

    let both = build(&scene(Some(UNDERLINE | LINE_THROUGH), 400.0, "aa ", "ab"));
    assert_eq!(rects(&both.decorated).len(), 2);
}

#[test]
fn a_wrapped_run_is_underlined_on_every_line_it_reaches() {
    let built = build(&scene(Some(UNDERLINE), 70.0, "a ", "ab ab ab ab ab ab"));
    assert!(built.baselines.len() >= 3, "{:?}", built.baselines);
    let post = table(b"post");
    let mut found = rects(&built.decorated);
    found.sort_by(|a, b| a[1].total_cmp(&b[1]));
    assert_eq!(found.len(), built.baselines.len(), "{found:?}");
    for (rect, baseline) in found.iter().zip(&built.baselines) {
        let (top, bottom) = expected(*baseline, i16_at(post, 8), i16_at(post, 10));
        assert!(close(rect[1], top) && close(rect[3], bottom), "{rect:?}");
        assert!(rect[2] <= 70.0 + 1e-3, "{rect:?} stays inside the text's width");
    }
}
