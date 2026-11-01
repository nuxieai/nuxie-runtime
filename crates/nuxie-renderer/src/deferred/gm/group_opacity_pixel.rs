//! A hosted CSS element fades as one picture, read from live Metal pixels.
//!
//! Each frame is recorded into a `DeferredSession` and replayed through
//! `GmHost`, as `bitmap_cache_pixel` does. The oracle is CSS's group opacity:
//! at fade f a pixel reads f·V + (1 - f)·K, for V the same frame at full
//! opacity and K the backdrop. At a partly covered pixel both sides are
//! K + f·(c - a·K), for the stack's premultiplied color c and alpha a, so the
//! read needs no interior mask.
//!
//! "Design" names the tests of `design-2`, "night" the earlier design's. The
//! two number their tests apart, and both sets apply to this picture.
use super::ore_gm_helper::*;
use crate::deferred::cmd::{
    command_stream::CommandReader,
    deferred_replayer::{snapshot_frame, DeferredFrame, DeferredReplayer},
    deferred_session::{DeferredSession, SegmentTarget},
    render_commands::{payload_size_of, DrawImagePod, RenderCmd},
    render_handle::{CANVAS_HANDLE_FLAG, CANVAS_HANDLE_MASK},
};
use nuxie_runtime::group_opacity_fixture::{Block, BoxShadow, Host, Item, Overflow, Scene, Unit};
use nuxie_runtime::{File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle};

const WHITE: u32 = 0xffff_ffff;
const BLACK: u32 = 0xff00_0000;
const RED: u32 = 0xffff_0000;
const GREEN: u32 = 0xff00_ff00;
const BLUE: u32 = 0xff00_00ff;
/// The backdrop of night's scenes, a gray nothing here draws.
const K: u32 = 0xff80_8080;
/// The design's colors: T1, T2 and U.
const T1: u32 = 0xfff0_f0f0;
const T2: u32 = 0xffc8_c8c8;
const U: u32 = 0xff78_7878;

const AT: (f32, f32) = (40.0, 40.0);
const FRAME: (u32, u32) = (300, 300);
/// Left, top, right and bottom, the last two exclusive.
type Area = (usize, usize, usize, usize);
const ALL: Area = (0, 0, FRAME.0 as usize, FRAME.1 as usize);
/// The base unit's box inset one pixel.
const INTERIOR: Area = (41, 41, 199, 199);
/// The base unit's photo and title, inset one pixel.
const PHOTO: Area = (41, 41, 199, 119);
const TITLE: Area = (41, 121, 199, 199);

fn each(channel: impl Fn(usize) -> f32) -> [f32; 3] {
    [channel(0), channel(1), channel(2)]
}

fn channels(color: u32) -> [f32; 3] {
    [
        ((color >> 16) & 0xff) as f32,
        ((color >> 8) & 0xff) as f32,
        (color & 0xff) as f32,
    ]
}

#[derive(Clone, Copy)]
struct View {
    transform: Mat2D,
    scope: Option<f32>,
    /// Replays on an atomics frame, whose images draw as `ImageRectDraw`.
    atomics: bool,
}
const PLAIN: View = View {
    transform: Mat2D([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
    scope: None,
    atomics: false,
};
fn under_scope(opacity: f32) -> View {
    View {
        scope: Some(opacity),
        ..PLAIN
    }
}

struct Drawn {
    pixels: Vec<u8>,
    width: usize,
    /// The recorded frame's content canvases, in device pixels.
    canvases: Vec<(u32, u32)>,
    /// `DrawImage`s in screen segments that name a content canvas.
    composites: usize,
}
impl Drawn {
    fn at(&self, x: usize, y: usize) -> [f32; 3] {
        let i = (y * self.width + x) * 4;
        [
            f32::from(self.pixels[i]),
            f32::from(self.pixels[i + 1]),
            f32::from(self.pixels[i + 2]),
        ]
    }
}

/// `DrawImage`s in screen segments that name one of the frame's canvases. A
/// count of every `DrawImage` would also find the photo's.
fn composites(frame: &DeferredFrame) -> usize {
    let mut named = 0;
    for segment in &frame.segments {
        if segment.target != SegmentTarget::Screen {
            continue;
        }
        let commands = &frame.commands[segment.begin as usize..segment.end as usize];
        let mut reader = CommandReader::new(commands, &frame.blobs);
        while let Some(command) = reader.next_u8().and_then(RenderCmd::from_byte) {
            if matches!(command, RenderCmd::DrawImage) {
                let draw: DrawImagePod = reader.read();
                let image = draw.image;
                if image & CANVAS_HANDLE_FLAG != 0
                    && frame
                        .content_canvases
                        .contains_key(&(image & CANVAS_HANDLE_MASK))
                {
                    named += 1;
                }
            } else {
                reader.skip(payload_size_of(command));
            }
        }
    }
    named
}

fn settle(artboard: &RuntimeArtboardInstanceHandle) {
    for _ in 0..3 {
        artboard.advance_default(1.0 / 60.0);
    }
}

fn render(bytes: &[u8], clear: u32, size: (u32, u32), view: View) -> Drawn {
    let mut host = GmHost::with_size(clear, false, size.0, size.1);
    if view.atomics {
        host.set_frame_mode(RenderCanvasFrameMode {
            disable_raster_ordering: true,
            ..Default::default()
        });
    }
    let mut session = PersistentFactory::new(DeferredSession::with_caps(Default::default()));
    session
        .borrow_mut()
        .bind_render_context(host.factory.persistent_context());
    let file = File::import(
        bytes,
        RuntimeFactoryHandle::from_factory(&mut session).unwrap(),
        None,
        None,
        None,
    )
    .expect("the scene imports");
    let artboard = file.with_file(File::artboard_default).expect("root");
    host.factory.borrow_mut().resize(size.0, size.1).unwrap();
    settle(&artboard);
    let screen = session.borrow().screen_renderer(0);
    {
        let mut screen = screen.borrow_mut();
        screen.save();
        screen.transform(view.transform);
        if let Some(opacity) = view.scope {
            screen.modulate_opacity(opacity);
        }
        artboard.draw(screen.as_mut());
        screen.restore();
    }
    let frame = snapshot_frame(&mut session.borrow_mut());
    session.borrow_mut().reset_frame();
    let canvases = frame
        .content_canvases
        .values()
        .map(|canvas| (canvas.borrow().width(), canvas.borrow().height()))
        .collect();
    let composites = composites(&frame);
    DeferredReplayer::default().replay_frame(&frame, &mut host);
    Drawn {
        pixels: host.finish(),
        width: size.0 as usize,
        canvases,
        composites,
    }
}

/// The same frame through `GmHost`'s immediate factory, which has no canvas
/// host, so nothing can take a picture.
fn render_immediate(bytes: &[u8], clear: u32, size: (u32, u32)) -> Drawn {
    let mut host = GmHost::with_size(clear, true, size.0, size.1);
    let file = File::import(
        bytes,
        RuntimeFactoryHandle::from_factory(&mut host.factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("the scene imports");
    let artboard = file.with_file(File::artboard_default).expect("root");
    settle(&artboard);
    let screen = host.screen();
    artboard.draw(screen.borrow_mut().as_mut());
    drop(screen);
    Drawn {
        pixels: host.finish(),
        width: size.0 as usize,
        canvases: Vec::new(),
        composites: 0,
    }
}

fn assert_reads(actual: [f32; 3], expected: [f32; 3], levels: f32, what: &str) {
    assert!(
        (0..3).all(|c| (actual[c] - expected[c]).abs() <= levels),
        "{what}: reads {actual:?}, expected {expected:?} within {levels}"
    );
}

/// Every pixel of `area` reads `want(x, y)` within `levels`.
fn assert_area(
    drawn: &Drawn,
    area: Area,
    levels: f32,
    what: &str,
    want: impl Fn(usize, usize) -> Option<[f32; 3]>,
) {
    let (mut wrong, mut read, mut first) = (0, 0, None);
    for y in area.1..area.3 {
        for x in area.0..area.2 {
            let Some(want) = want(x, y) else {
                continue;
            };
            read += 1;
            let actual = drawn.at(x, y);
            if (0..3).any(|c| (actual[c] - want[c]).abs() > levels) {
                wrong += 1;
                first.get_or_insert((x, y, actual, want));
            }
        }
    }
    assert!(read > 0, "{what}: nothing to read");
    assert!(
        first.is_none(),
        "{what}: {wrong} of {read} pixels off by more than {levels}. First at {first:?} (x, y, reads, expected)"
    );
}

/// Night's sum: `fading` is one source-over of `full` at 0.5 over `backdrop`.
fn assert_half(fading: &Drawn, full: &Drawn, backdrop: u32, area: Area, what: &str) {
    let k = channels(backdrop);
    assert_area(fading, area, 2.0, what, |x, y| {
        let v = full.at(x, y);
        Some(each(|c| 0.5 * v[c] + 0.5 * k[c]))
    });
}

/// Night's T3 arithmetic: one white paint at 0.5 over `under`, then the photo
/// or the title at 0.5 over that. An edge of a title is left out, since its
/// coverage is not read from `full`.
fn assert_per_paint(
    half: &Drawn,
    full: &Drawn,
    under: impl Fn(usize, usize) -> [f32; 3],
    photo: Option<Area>,
    title: Area,
    what: &str,
) {
    let over_white = |v: f32, under: f32| 0.5 * v + 0.25 * 255.0 + 0.25 * under;
    if let Some(photo) = photo {
        assert_area(half, photo, 2.0, &format!("{what}, photo"), |x, y| {
            let (v, u) = (full.at(x, y), under(x, y));
            Some(each(|c| over_white(v[c], u[c])))
        });
    }
    assert_area(half, title, 2.0, &format!("{what}, title"), |x, y| {
        let v = full.at(x, y);
        v.iter().all(|c| *c <= 2.0).then(|| {
            let u = under(x, y);
            each(|c| over_white(0.0, u[c]))
        })
    });
    assert_area(half, title, 2.0, &format!("{what}, white"), |x, y| {
        let v = full.at(x, y);
        v.iter().all(|c| *c >= 253.0).then(|| {
            let u = under(x, y);
            each(|c| 0.5 * 255.0 + 0.5 * u[c])
        })
    });
}

fn no_picture(drawn: &Drawn, what: &str) {
    assert!(drawn.canvases.is_empty(), "{what}: {:?}", drawn.canvases);
    assert_eq!(drawn.composites, 0, "{what}");
}

fn one_picture(drawn: &Drawn, what: &str) {
    assert_eq!(drawn.canvases.len(), 1, "{what}: {:?}", drawn.canvases);
    assert_eq!(drawn.composites, 1, "{what}");
}

fn png(width: u32, height: u32, pixel: impl Fn(u32, u32) -> [u8; 3]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            data.extend(pixel(x, y));
            data.push(255);
        }
    }
    writer.write_image_data(&data).unwrap();
    drop(writer);
    bytes
}

/// The opaque photo that fills the top of night's base unit.
fn photo() -> Vec<u8> {
    png(160, 80, |x, y| {
        [(x * 255 / 159) as u8, (y * 255 / 79) as u8, 96]
    })
}

fn photo_item() -> Item {
    Item::Photo { at: (0.0, 0.0) }
}
fn title_item(at: (f32, f32), width: f32) -> Item {
    Item::Title {
        at,
        size: (width, 40.0),
        text: "Title",
        color: BLACK,
    }
}
fn block(at: (f32, f32), size: (f32, f32), color: u32) -> Item {
    Item::Block(Block {
        at,
        size,
        color: Some(color),
        ..Block::default()
    })
}

/// Night's base unit in the compiled shape: `clip: false`, a record that clips
/// both axes, a white background as an own paint, and directly in its draw
/// list a photo over its top and a black title on the white below it.
fn base_unit() -> Unit {
    Unit {
        background: Some(WHITE),
        items: vec![photo_item(), title_item((8.0, 96.0), 144.0)],
        ..Unit::compiled(160.0, 160.0)
    }
}

fn scene_of(unit: Unit, opacity: f32) -> Vec<u8> {
    Scene {
        photo: Some(photo()),
        ..Scene::hosted((300.0, 300.0), unit, AT, opacity)
    }
    .bytes()
}

/// Two hosts of `unit` at the same place, the top at `top`.
fn two_copies(unit: Unit, top: f32, root: Unit) -> Vec<u8> {
    Scene {
        root: Unit {
            items: vec![
                Item::Host(Host::plain("bottom", 0, AT, 1.0)),
                Item::Host(Host::plain("top", 0, AT, top)),
            ],
            ..root
        },
        units: vec![unit],
        photo: Some(photo()),
    }
    .bytes()
}

/// Two opaque boxes that overlap, in a unit that clips by its record. Red is
/// on top of blue.
fn red_over_blue() -> Unit {
    Unit {
        items: vec![
            block((20.0, 20.0), (90.0, 90.0), BLUE),
            block((60.0, 60.0), (90.0, 90.0), RED),
        ],
        ..Unit::compiled(160.0, 160.0)
    }
}
/// Device pixels of `red_over_blue` at `AT`.
const OVERLAP: (usize, usize) = (125, 125);
const RED_ALONE: (usize, usize) = (170, 170);
const BLUE_ALONE: (usize, usize) = (80, 80);

fn assert_one_picture_over_green(drawn: &Drawn, f: f32, what: &str) {
    let green = (1.0 - f) * 255.0;
    let (red, blue) = (f * 255.0, f * 255.0);
    assert_reads(
        drawn.at(OVERLAP.0, OVERLAP.1),
        [red, green, 0.0],
        1.0,
        &format!("{what}, the overlap, where a fade by paints lets blue show"),
    );
    assert_reads(
        drawn.at(RED_ALONE.0, RED_ALONE.1),
        [red, green, 0.0],
        1.0,
        &format!("{what}, red alone"),
    );
    assert_reads(
        drawn.at(BLUE_ALONE.0, BLUE_ALONE.1),
        [0.0, green, blue],
        1.0,
        &format!("{what}, blue alone"),
    );
    assert_reads(
        drawn.at(10, 10),
        [0.0, 255.0, 0.0],
        1.0,
        &format!("{what}, outside"),
    );
}

/// Design T2.
#[test]
fn a_hosted_element_at_half_reads_as_one_picture() {
    let full = render(&scene_of(red_over_blue(), 1.0), GREEN, FRAME, PLAIN);
    assert_reads(
        full.at(OVERLAP.0, OVERLAP.1),
        [255.0, 0.0, 0.0],
        1.0,
        "red is on top at rest",
    );
    let half = render(&scene_of(red_over_blue(), 0.5), GREEN, FRAME, PLAIN);
    assert_one_picture_over_green(&half, 0.5, "own fade 0.5");
    one_picture(&half, "own fade 0.5");
}

/// Rule 2: the fade that counts is own fade times the renderer's scope, so an
/// element at own 1 under a scope takes a picture too.
#[test]
fn a_hosted_element_at_own_one_under_a_scope_reads_as_one_picture() {
    let drawn = render(
        &scene_of(red_over_blue(), 1.0),
        GREEN,
        FRAME,
        under_scope(0.5),
    );
    assert_one_picture_over_green(&drawn, 0.5, "own 1 under a scope at 0.5");
    one_picture(&drawn, "own 1 under a scope at 0.5");
}

/// Rule 2 again: own fade and scope multiply, once each.
#[test]
fn own_fade_and_scope_multiply_into_one_composite() {
    let drawn = render(
        &scene_of(red_over_blue(), 0.5),
        GREEN,
        FRAME,
        under_scope(0.5),
    );
    assert_one_picture_over_green(&drawn, 0.25, "own 0.5 under a scope at 0.5");
    one_picture(&drawn, "own 0.5 under a scope at 0.5");
}

/// Design section 6, PR 2: an enabled bitmap cache keeps its raster, drawn at
/// full opacity and composited under the fade's scope, so it needs no second
/// canvas and still reads as one picture.
#[test]
fn a_cached_unit_fades_its_raster_under_the_scope() {
    let unit = Unit {
        cached: true,
        ..red_over_blue()
    };
    let drawn = render(&scene_of(unit, 0.5), GREEN, FRAME, PLAIN);
    assert_one_picture_over_green(&drawn, 0.5, "a cached unit");
    assert_eq!(drawn.canvases.len(), 1, "the cache's raster, and no other");
}

/// Design T5, a guard: an unclipped component whose children have no box of
/// their own draws today's pixels. 240 at 0.5 over 120 reads 180, then 200 at
/// 0.5 over that reads 190.
#[test]
fn an_unclipped_component_with_no_box_keeps_its_paints() {
    let unit = Unit {
        items: vec![
            block((20.0, 20.0), (90.0, 90.0), T1),
            block((60.0, 60.0), (90.0, 90.0), T2),
        ],
        ..Unit::css(160.0, 160.0)
    };
    let drawn = render(&scene_of(unit, 0.5), U, FRAME, PLAIN);
    assert_reads(
        drawn.at(OVERLAP.0, OVERLAP.1),
        [190.0; 3],
        1.0,
        "overlap, paint by paint",
    );
    assert_reads(
        drawn.at(BLUE_ALONE.0, BLUE_ALONE.1),
        [180.0; 3],
        1.0,
        "the 240 box alone",
    );
    assert_reads(
        drawn.at(RED_ALONE.0, RED_ALONE.1),
        [160.0; 3],
        1.0,
        "the 200 box alone",
    );
    no_picture(&drawn, "no box");
}

/// Design T6, a guard: with no CSS record each paint fades on its own,
/// `transform_component.cpp:115`, hosted in a Rive artboard and in a CSS
/// Screen. The frame clip gives it a box, so only rule 1 keeps it from one.
#[test]
fn a_rive_made_unit_draws_paint_by_paint_at_half() {
    let unit = || Unit {
        items: vec![
            Item::Shape {
                at: (20.0, 20.0),
                size: (90.0, 90.0),
                color: T2,
            },
            Item::Shape {
                at: (60.0, 60.0),
                size: (90.0, 90.0),
                color: T1,
            },
        ],
        ..Unit::rive(160.0, 160.0)
    };
    let hosted_in = |root: Unit, opacity: f32| {
        Scene {
            root: Unit {
                items: vec![Item::Host(Host::plain("host", 0, AT, opacity))],
                ..root
            },
            units: vec![unit()],
            photo: None,
        }
        .bytes()
    };
    for (what, root) in [
        ("in a Rive artboard", Unit::rive(300.0, 300.0)),
        ("in a CSS Screen", Unit::css(300.0, 300.0)),
    ] {
        let full = render(&hosted_in(root.clone(), 1.0), U, FRAME, PLAIN);
        assert_reads(
            full.at(OVERLAP.0, OVERLAP.1),
            [200.0; 3],
            1.0,
            &format!("{what}: 200 is on top at rest"),
        );
        let half = render(&hosted_in(root, 0.5), U, FRAME, PLAIN);
        assert_reads(
            half.at(OVERLAP.0, OVERLAP.1),
            [190.0; 3],
            1.0,
            &format!("{what}: upstream's formula"),
        );
        no_picture(&half, what);
    }
}

/// A red box with one image, over 120. A build that groups it reads 100.
fn lone_picture(opacity: f32) -> Vec<u8> {
    let unit = Unit {
        background: Some(0xfff0_0000),
        items: vec![Item::Photo { at: (40.0, 40.0) }],
        ..Unit::compiled(160.0, 160.0)
    };
    Scene {
        photo: Some(png(80, 80, |_, _| [80, 0, 0])),
        ..Scene::hosted((300.0, 300.0), unit, AT, opacity)
    }
    .bytes()
}

/// Design T7: a lone picture stays per-paint at any effective fade, so red
/// shows through by f(1 - f), as Hero's own fade of a picture does. An image
/// draws as a path on most frames and as `ImageRectDraw` on an atomics frame,
/// and a scope reaches it once on both.
#[test]
fn a_lone_picture_keeps_its_paints_at_half_and_under_a_scope() {
    for (what, drawn) in [
        ("own fade 0.5", render(&lone_picture(0.5), U, FRAME, PLAIN)),
        (
            "own 1 under a scope at 0.5",
            render(&lone_picture(1.0), U, FRAME, under_scope(0.5)),
        ),
        (
            "own 1 under a scope at 0.5, on an atomics frame",
            render(
                &lone_picture(1.0),
                U,
                FRAME,
                View {
                    atomics: true,
                    ..under_scope(0.5)
                },
            ),
        ),
    ] {
        let image = drawn.at(120, 120);
        assert!(
            (image[0] - 130.0).abs() <= 1.0,
            "{what}: the image reads {image:?}, expected red 130"
        );
        let red = drawn.at(50, 50);
        assert!(
            (red[0] - 180.0).abs() <= 1.0,
            "{what}: the background reads {red:?}, expected red 180"
        );
        no_picture(&drawn, what);
    }
}

/// Night T1. The photo reads 0.75·P + 0.25·255 paint by paint, and the title
/// about 63 against 0.
#[test]
fn over_a_held_copy_the_fading_copy_adds_nothing_to_the_identical_pixels() {
    let held = render(
        &two_copies(base_unit(), 0.0, Unit::css(300.0, 300.0)),
        K,
        FRAME,
        PLAIN,
    );
    let fading = render(
        &two_copies(base_unit(), 0.5, Unit::css(300.0, 300.0)),
        K,
        FRAME,
        PLAIN,
    );
    assert_area(
        &fading,
        PHOTO,
        2.0,
        "the photo over its held copy",
        |x, y| Some(held.at(x, y)),
    );
    assert_area(
        &fading,
        TITLE,
        2.0,
        "the title over its held copy",
        |x, y| Some(held.at(x, y)),
    );
    assert_area(&fading, INTERIOR, 2.0, "the interior", |x, y| {
        Some(held.at(x, y))
    });
    one_picture(&fading, "the fading copy, and none for the held one");
}

/// Night T2.
#[test]
fn alone_over_the_backdrop_the_unit_is_one_source_over() {
    let full = render(&scene_of(base_unit(), 1.0), K, FRAME, PLAIN);
    let half = render(&scene_of(base_unit(), 0.5), K, FRAME, PLAIN);
    assert_half(&half, &full, K, INTERIOR, "the unit at 0.5");
    one_picture(&half, "the unit at 0.5");
}

/// Rive's photo, title and white rectangle, in draw order bottom last.
fn rive_unit() -> Unit {
    Unit {
        items: vec![
            title_item((8.0, 96.0), 144.0),
            photo_item(),
            Item::Shape {
                at: (0.0, 0.0),
                size: (160.0, 160.0),
                color: WHITE,
            },
        ],
        ..Unit::rive(160.0, 160.0)
    }
}

/// Night T3, a guard.
#[test]
fn a_rive_made_copy_over_a_held_copy_fades_paint_by_paint() {
    let held = render(
        &two_copies(rive_unit(), 0.0, Unit::css(300.0, 300.0)),
        K,
        FRAME,
        PLAIN,
    );
    let fading = render(
        &two_copies(rive_unit(), 0.5, Unit::css(300.0, 300.0)),
        K,
        FRAME,
        PLAIN,
    );
    assert_per_paint(
        &fading,
        &held,
        |x, y| held.at(x, y),
        Some(PHOTO),
        TITLE,
        "Rive-made",
    );
    no_picture(&fading, "Rive-made");
}

/// Night T4, a guard: the CSS scene through a factory with no canvas host.
#[test]
fn with_no_canvas_host_the_fade_stays_paint_by_paint() {
    let held = render_immediate(
        &two_copies(base_unit(), 0.0, Unit::css(300.0, 300.0)),
        K,
        FRAME,
    );
    let fading = render_immediate(
        &two_copies(base_unit(), 0.5, Unit::css(300.0, 300.0)),
        K,
        FRAME,
    );
    assert_per_paint(
        &fading,
        &held,
        |x, y| held.at(x, y),
        Some(PHOTO),
        TITLE,
        "no canvas host",
    );
}

fn turned(radians: f32, tx: f32, ty: f32) -> Mat2D {
    let (sin, cos) = radians.sin_cos();
    Mat2D([cos, sin, -sin, cos, tx, ty])
}

/// Night T5. Upstream's corner rounding fails the quarter-pixel offsets, and a
/// bilinear composite fails the turn at the edges.
#[test]
fn the_picture_lands_on_the_pixels_the_vectors_would_at_every_transform() {
    const SIZE: (u32, u32) = (680, 680);
    let mut transforms = vec![
        ("scale 3", Mat2D([3.0, 0.0, 0.0, 3.0, 0.0, 0.0])),
        ("scale 3, offset", Mat2D([3.0, 0.0, 0.0, 3.0, 10.3, 7.6])),
        ("turned 30 degrees", turned(0.5235988, 150.0, 20.0)),
    ];
    for (name, scale) in [("1.1 by 1.1", (1.1, 1.1)), ("1.1 by 0.9", (1.1, 0.9))] {
        for offset in [0.0, 0.25, 0.5, 0.75] {
            transforms.push((name, Mat2D([scale.0, 0.0, 0.0, scale.1, offset, offset])));
        }
    }
    for (what, transform) in transforms {
        let view = View { transform, ..PLAIN };
        let full = render(&scene_of(base_unit(), 1.0), K, SIZE, view);
        let half = render(&scene_of(base_unit(), 0.5), K, SIZE, view);
        assert_half(
            &half,
            &full,
            K,
            (0, 0, SIZE.0 as usize, SIZE.1 as usize),
            &format!("{what} {:?}", transform.0),
        );
        one_picture(&half, what);
    }
}

fn differs_from(pixel: [f32; 3], backdrop: u32) -> bool {
    let k = channels(backdrop);
    (0..3).any(|c| (pixel[c] - k[c]).abs() > 2.0)
}

/// Night T6 (a).
#[test]
fn a_rounded_record_clip_cuts_the_picture_as_it_cuts_the_vectors() {
    let unit = || Unit {
        radius: 16.0,
        background: Some(WHITE),
        items: vec![block((100.0, 40.0), (120.0, 60.0), BLUE)],
        ..Unit::compiled(160.0, 160.0)
    };
    let full = render(&scene_of(unit(), 1.0), K, FRAME, PLAIN);
    assert!(!differs_from(full.at(41, 41), K), "the rounded-off corner");
    assert!(!differs_from(full.at(220, 90), K), "the overflow");
    let half = render(&scene_of(unit(), 0.5), K, FRAME, PLAIN);
    assert_half(&half, &full, K, ALL, "a rounded clip with overflow");
    one_picture(&half, "a rounded clip with overflow");
}

/// Night T6 (b).
#[test]
fn a_host_clip_cuts_the_picture() {
    let scene = |opacity: f32| {
        Scene {
            root: Unit {
                items: vec![Item::Block(Block {
                    at: AT,
                    size: (80.0, 160.0),
                    overflow: Overflow::Hidden,
                    items: vec![Item::Host(Host::plain("host", 0, (0.0, 0.0), opacity))],
                    ..Block::default()
                })],
                ..Unit::css(300.0, 300.0)
            },
            units: vec![base_unit()],
            photo: Some(photo()),
        }
        .bytes()
    };
    let full = render(&scene(1.0), K, FRAME, PLAIN);
    assert!(differs_from(full.at(100, 60), K), "left of the host's clip");
    assert!(
        !differs_from(full.at(160, 60), K),
        "right of the host's clip"
    );
    let half = render(&scene(0.5), K, FRAME, PLAIN);
    assert_half(&half, &full, K, ALL, "under a host's clip");
    one_picture(&half, "under a host's clip");
}

/// Night T6 (c). The overflow does not size the canvas: 160 points and 2 guard
/// pixels a side, rounded up by 64.
#[test]
fn overflow_past_the_record_clip_neither_shows_nor_sizes_the_picture() {
    let unit = || Unit {
        background: Some(WHITE),
        items: vec![block((40.0, 40.0), (320.0, 320.0), BLUE)],
        ..Unit::compiled(160.0, 160.0)
    };
    let scene = |opacity: f32| Scene::hosted((600.0, 600.0), unit(), AT, opacity).bytes();
    const SIZE: (u32, u32) = (600, 600);
    let full = render(&scene(1.0), K, SIZE, PLAIN);
    assert!(differs_from(full.at(150, 150), K), "inside the clip");
    assert!(!differs_from(full.at(260, 260), K), "past the clip");
    let half = render(&scene(0.5), K, SIZE, PLAIN);
    assert_half(&half, &full, K, (0, 0, 600, 600), "overflow past the clip");
    one_picture(&half, "overflow past the clip");
    assert_eq!(half.canvases, [(192, 192)]);
}

const SHADOW: BoxShadow = BoxShadow {
    color: BLACK,
    blur: 8.0,
    spread: 4.0,
};

/// Night T6 (d), case 1: the shadow and the clip are both the unit's own. A box
/// cut by the CSS clip would leave K over the shadow's pixels.
#[test]
fn a_units_own_shadow_stays_in_the_picture() {
    let unit = || Unit {
        background: Some(WHITE),
        shadow: Some(SHADOW),
        items: vec![block((40.0, 40.0), (80.0, 80.0), BLUE)],
        ..Unit::compiled(160.0, 160.0)
    };
    let full = render(&scene_of(unit(), 1.0), K, FRAME, PLAIN);
    assert!(differs_from(full.at(203, 120), K), "the shadow outside");
    let half = render(&scene_of(unit(), 0.5), K, FRAME, PLAIN);
    assert_half(&half, &full, K, ALL, "a unit's own shadow");
    one_picture(&half, "a unit's own shadow");
}

/// Night T6 (d), case 2: an empty record, no frame clip, and one root box that
/// carries the shadow and clips.
#[test]
fn a_root_boxs_shadow_stays_in_the_picture() {
    let unit = || Unit {
        items: vec![Item::Block(Block {
            size: (160.0, 160.0),
            color: Some(WHITE),
            overflow: Overflow::Hidden,
            shadow: Some(SHADOW),
            items: vec![block((40.0, 40.0), (80.0, 80.0), BLUE)],
            ..Block::default()
        })],
        ..Unit::css(160.0, 160.0)
    };
    let full = render(&scene_of(unit(), 1.0), K, FRAME, PLAIN);
    assert!(differs_from(full.at(203, 120), K), "the shadow outside");
    let half = render(&scene_of(unit(), 0.5), K, FRAME, PLAIN);
    assert_half(&half, &full, K, ALL, "a root box's shadow");
    one_picture(&half, "a root box's shadow");
}

/// A unit whose root hosts component `inner`, in the compiled shape: an empty
/// record, `clip: false`, and one layout-sized host whose item record says
/// `overflow: hidden`.
fn hosted_root(inner: Unit, opacity: f32) -> Vec<u8> {
    Scene {
        root: Unit {
            items: vec![Item::Host(Host::plain("host", 0, AT, opacity))],
            ..Unit::css(300.0, 300.0)
        },
        units: vec![
            Unit {
                items: vec![Item::Host(Host {
                    layout: Some((160.0, 160.0)),
                    overflow: Overflow::Hidden,
                    ..Host::plain("root", 1, (0.0, 0.0), 1.0)
                })],
                ..Unit::css(160.0, 160.0)
            },
            inner,
        ],
        photo: None,
    }
    .bytes()
}

/// Night T6 (e), case 3.
#[test]
fn a_hosted_root_takes_its_components_box() {
    let inner = || Unit {
        frame_clip: true,
        background: Some(WHITE),
        items: vec![block((100.0, 40.0), (120.0, 60.0), BLUE)],
        ..Unit::css(160.0, 160.0)
    };
    let full = render(&hosted_root(inner(), 1.0), K, FRAME, PLAIN);
    assert!(differs_from(full.at(160, 90), K), "inside the frame");
    assert!(!differs_from(full.at(220, 90), K), "over the overflow");
    let half = render(&hosted_root(inner(), 0.5), K, FRAME, PLAIN);
    assert_half(&half, &full, K, ALL, "a hosted root");
    one_picture(&half, "a hosted root, at the unit");
}

/// Night T6 (f), a guard: `overflow: visible` holding the photo and the title
/// directly, so its draw list ends with a text. No case gives it a box.
#[test]
fn an_unclipped_unit_that_ends_in_text_falls_back() {
    let unit = || Unit {
        background: Some(WHITE),
        items: vec![photo_item(), title_item((8.0, 96.0), 144.0)],
        ..Unit::css(160.0, 160.0)
    };
    let full = render(&scene_of(unit(), 1.0), K, FRAME, PLAIN);
    let half = render(&scene_of(unit(), 0.5), K, FRAME, PLAIN);
    no_picture(&half, "no box");
    assert_per_paint(
        &half,
        &full,
        |_, _| channels(K),
        Some(PHOTO),
        TITLE,
        "no box",
    );
}

/// Night T6 (f), a guard: a clipping root box beside a title at the top level,
/// so the draw list is not one root box.
#[test]
fn a_clipping_box_beside_a_title_falls_back() {
    let unit = || Unit {
        items: vec![
            Item::Block(Block {
                size: (160.0, 100.0),
                color: Some(WHITE),
                overflow: Overflow::Hidden,
                items: vec![block((20.0, 20.0), (60.0, 60.0), BLUE)],
                ..Block::default()
            }),
            title_item((8.0, 110.0), 144.0),
        ],
        ..Unit::css(160.0, 160.0)
    };
    let full = render(&scene_of(unit(), 1.0), K, FRAME, PLAIN);
    let half = render(&scene_of(unit(), 0.5), K, FRAME, PLAIN);
    no_picture(&half, "no box");
    let k = channels(K);
    assert_reads(
        half.at(90, 90),
        each(|c| 0.5 * channels(BLUE)[c] + 0.25 * 255.0 + 0.25 * k[c]),
        2.0,
        "blue over the box's white, paint by paint",
    );
    assert_reads(
        half.at(50, 50),
        each(|c| 0.5 * 255.0 + 0.5 * k[c]),
        2.0,
        "the box's white",
    );
    assert_area(
        &half,
        (41, 150, 199, 199),
        2.0,
        "the title over K",
        |x, y| {
            full.at(x, y)
                .iter()
                .all(|c| *c <= 2.0)
                .then(|| each(|c| 0.5 * k[c]))
        },
    );
}

/// Night T6 (f), a guard: a hosted root whose component neither clips nor
/// holds one box, so its box is unknown and the unit's case 3 has none.
#[test]
fn a_hosted_root_whose_component_has_no_box_falls_back() {
    let inner = || Unit {
        background: Some(WHITE),
        items: vec![Item::Title {
            at: (8.0, 60.0),
            size: (480.0, 40.0),
            text: "A title much wider than its frame",
            color: BLACK,
        }],
        ..Unit::css(160.0, 160.0)
    };
    let full = render(&hosted_root(inner(), 1.0), K, FRAME, PLAIN);
    let half = render(&hosted_root(inner(), 0.5), K, FRAME, PLAIN);
    no_picture(&half, "no box");
    assert_per_paint(&half, &full, |_, _| channels(K), None, INTERIOR, "no box");
}

/// Night T10, case 4. Paint by paint the border reads
/// 0.5·B + 0.25·255 + 0.25·K, the background showing through it.
#[test]
fn a_top_that_holds_nothing_fades_its_border_and_background_as_one() {
    let unit = || Unit {
        background: Some(WHITE),
        border: Some((4.0, BLUE)),
        ..Unit::css(160.0, 160.0)
    };
    let full = render(&scene_of(unit(), 1.0), K, FRAME, PLAIN);
    assert_reads(full.at(42, 120), channels(BLUE), 2.0, "the border at rest");
    let half = render(&scene_of(unit(), 0.5), K, FRAME, PLAIN);
    assert_half(&half, &full, K, ALL, "a border over its background");
    one_picture(&half, "a border over its background");
}
