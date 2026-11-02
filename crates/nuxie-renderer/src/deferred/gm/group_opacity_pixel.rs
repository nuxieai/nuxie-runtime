//! A CSS element fades as one picture, read from live Metal pixels: a hosted
//! element, a box that clips both axes, and a Screen with its flights drawn
//! outside it.
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
use crate::RenderMode;
use nuxie_runtime::css_layout::keys::{DISPLAY, FADE_OUTSIDE, OVERFLOW_X, OVERFLOW_Y};
use nuxie_runtime::group_opacity_fixture::{
    artboard_record, bind_page, box_record, row_opacity, screen_row, set_row_opacity, Block,
    BoxShadow, Host, Item, Overflow, Page, Scene, Unit, KEYED_SCREEN, MARKS,
};
use nuxie_runtime::live_move::{slot_pose, Flight, LiveMoves};
use nuxie_runtime::live_move_fixture::{
    css_flight_over_backing_fixture, css_flight_over_cached_backing_fixture,
    css_flight_under_fading_box_fixture, css_paint_fixture, PaintCase, FADING_BOX_UNIT,
    MARK_BACKING, MARK_CHILD, MARK_FLIGHT, MARK_PANEL, SCREEN_B_BACKING,
};
use nuxie_runtime::mechanical_port::source::{
    artboard::Artboard,
    artboard_component_list::ArtboardComponentList,
    generated::{
        bitmap_cache_base::BitmapCacheBase, core_registry::CoreRegistry,
        layout_component_base::LayoutComponentBase,
    },
    math::vec2d::Vec2D,
    viewmodel::viewmodel_instance_list_item::ViewModelInstanceListItem,
};
use nuxie_runtime::{
    File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
    RuntimeStateMachineInstanceHandle,
};

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
    /// The mode the frame replays in.
    mode: RenderMode,
}
const PLAIN: View = View {
    transform: Mat2D([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
    scope: None,
    atomics: false,
    mode: RenderMode::RasterOrdering,
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

/// A file imported once and drawn frame by frame, so a test can write to it
/// between frames. One replayer keeps the resources each frame names across
/// frames. Fields drop in order: the replayer, the host, then the root, the
/// file and the session.
struct Live {
    replayer: DeferredReplayer,
    host: GmHost,
    root: RuntimeArtboardInstanceHandle,
    file: RuntimeFileHandle,
    session: PersistentFactory<DeferredSession>,
    size: (u32, u32),
}
impl Live {
    /// `root` names the artboard to draw, the file's default when none. A mode
    /// other than raster ordering is also the mode of a picture's canvas, as
    /// Android's are.
    fn new(
        bytes: &[u8],
        clear: u32,
        size: (u32, u32),
        atomics: bool,
        mode: RenderMode,
        root: Option<&str>,
    ) -> Self {
        let mut host = GmHost::with_mode(clear, false, size.0, size.1, mode);
        if atomics {
            host.set_frame_mode(RenderCanvasFrameMode {
                disable_raster_ordering: true,
                ..Default::default()
            });
        } else if mode != RenderMode::RasterOrdering {
            host.set_frame_mode(mode.canvas_frame_mode());
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
        let root = match root {
            Some(name) => file.with_file(|file| file.artboard_named(name)),
            None => file.with_file(File::artboard_default),
        }
        .expect("root");
        host.factory.borrow_mut().resize(size.0, size.1).unwrap();
        Self {
            replayer: DeferredReplayer::default(),
            host,
            root,
            file,
            session,
            size,
        }
    }
    /// Records one frame of the root as it stands, and replays it.
    fn draw(&mut self, view: View) -> Drawn {
        let screen = self.session.borrow().screen_renderer(0);
        {
            let mut screen = screen.borrow_mut();
            screen.save();
            screen.transform(view.transform);
            if let Some(opacity) = view.scope {
                screen.modulate_opacity(opacity);
            }
            self.root.draw(screen.as_mut());
            screen.restore();
        }
        let frame = snapshot_frame(&mut self.session.borrow_mut());
        self.session.borrow_mut().reset_frame();
        let canvases = frame
            .content_canvases
            .values()
            .map(|canvas| (canvas.borrow().width(), canvas.borrow().height()))
            .collect();
        let composites = composites(&frame);
        self.replayer.replay_frame(&frame, &mut self.host);
        assert_eq!(self.replayer.dropped_draws(), 0, "no draw is dropped");
        Drawn {
            pixels: self.host.finish_frame(),
            width: self.size.0 as usize,
            canvases,
            composites,
        }
    }
    /// Settles the root, then draws it.
    fn settled(&mut self, view: View) -> Drawn {
        settle(&self.root);
        self.draw(view)
    }
}

impl Live {
    /// A page of `FRAME`'s size, its model bound so its rows mount, settled.
    fn page(bytes: &[u8], clear: u32) -> Self {
        let live = Self::new(bytes, clear, FRAME, false, RenderMode::RasterOrdering, None);
        bind_page(&live.file, &live.root);
        settle(&live.root);
        live
    }
}

fn render(bytes: &[u8], clear: u32, size: (u32, u32), view: View) -> Drawn {
    Live::new(bytes, clear, size, view.atomics, view.mode, None).settled(view)
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

/// A depth-stencil frame, the kind every Android frame is.
const DEPTH_STENCIL: View = View {
    mode: RenderMode::ClockwiseMsaa1,
    ..PLAIN
};

/// The control beside the depth-stencil tests below: a picture is an image, and
/// an image paint is never flagged opaque, so it is safe on this frame.
#[test]
fn a_hosted_element_at_half_reads_as_one_picture_on_a_depth_stencil_frame() {
    let half = render(&scene_of(red_over_blue(), 0.5), GREEN, FRAME, DEPTH_STENCIL);
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

/// Design T10 at 0, with an enabled bitmap cache on the Screen (ruling U11):
/// a Screen at 0 draws only its flights, straight onto the renderer, so it
/// makes no canvas, its cache's included. With no flight nothing draws, and
/// the row reads the page's blue 40. A draw through the cache at 0 makes the
/// cache's canvas and composites it, empty.
#[test]
fn a_cached_screen_at_zero_makes_no_canvas() {
    let mut live = Live::page(
        &page_of(Unit {
            cached: true,
            ..t10_screen()
        }),
        BLUE_40,
    );
    set_row_opacity(&screen_row(&live.root, "Screen"), 0.0);
    let drawn = live.settled(PLAIN);
    no_picture(&drawn, "a cached Screen at 0");
    assert_reads(
        drawn.at(IN_CHILD.0, IN_CHILD.1),
        channels(BLUE_40),
        1.0,
        "a cached Screen at 0, the row",
    );
}

/// Design T16 with an enabled bitmap cache on Screen B (ruling U11): at 0 its
/// flight still reads its own color, drawn straight onto the renderer, and
/// the Screen makes no canvas, its cache's included. A draw through the cache
/// at 0 makes the cache's canvas and draws the flight into it.
#[test]
fn a_cached_screen_at_zero_makes_no_canvas_and_its_flight_draws() {
    assert_screen_at_zero_makes_no_canvas_and_its_flight_draws(
        &css_flight_over_cached_backing_fixture(),
        "cached Screen B at 0",
    );
}

/// The resolution a Screen's bitmap cache draws its raster at in the tests
/// of ruling U13: twice the device pixels of the Screen's picture at
/// `PLAIN`, so a frame's canvases tell the cache's raster from the picture
/// by size. At 1 the two are the same size, and a count of canvases can't
/// tell a build that draws through the cache from one that doesn't.
const CACHE_RESOLUTION: f32 = 2.0;

/// Sets `row`'s bitmap cache to draw at `CACHE_RESOLUTION`.
fn cache_at_resolution(row: &RuntimeArtboardInstanceHandle) {
    let cache = row
        .with_artboard(|artboard| artboard.bitmap_cache())
        .expect("the Screen's bitmap cache");
    assert!(CoreRegistry::set_double_handle(
        &cache,
        i32::from(BitmapCacheBase::RESOLUTION_PROPERTY_KEY),
        CACHE_RESOLUTION
    ));
}

/// How many of `drawn`'s canvases are `row`'s cache raster: its bounds at
/// `CACHE_RESOLUTION`, drawn at `PLAIN`.
fn rasters(drawn: &Drawn, row: &RuntimeArtboardInstanceHandle) -> usize {
    let bounds = row.with_artboard(|artboard| artboard.bounds());
    let raster = (
        (bounds.width() * CACHE_RESOLUTION).ceil() as u32,
        (bounds.height() * CACHE_RESOLUTION).ceil() as u32,
    );
    drawn
        .canvases
        .iter()
        .filter(|&&canvas| canvas == raster)
        .count()
}

/// `drawn`'s canvases in a fixed order, to compare two frames'.
fn sorted_canvases(drawn: &Drawn) -> Vec<(u32, u32)> {
    let mut canvases = drawn.canvases.clone();
    canvases.sort_unstable();
    canvases
}

/// Rule 7's page for a cached Screen: a Screen painting B 60, holding a box
/// of T2 200 at own 1 named "marked", at `AT` and 160 points square, over U
/// 120. With `cached` the Screen carries an enabled bitmap cache, drawing at
/// `CACHE_RESOLUTION`. The Screen is at 1.
fn box_on_backing(cached: bool) -> (Live, RuntimeArtboardInstanceHandle) {
    let live = Live::page(
        &page_of(Unit {
            background: Some(B),
            cached,
            items: vec![Item::Block(Block {
                name: Some("marked"),
                at: AT,
                size: (160.0, 160.0),
                color: Some(T2),
                ..Block::default()
            })],
            ..Unit::css(300.0, 300.0)
        }),
        U,
    );
    let row = screen_row(&live.root, "Screen");
    if cached {
        cache_at_resolution(&row);
    }
    (live, row)
}

/// Rule 7 with an enabled bitmap cache on the Screen (ruling U13): while it
/// fades, a cached Screen fades as one picture drawn fresh, exactly as an
/// uncached one does, and its cache sits out. With the Screen at 0.5 the box
/// reads 0.5·200 + 0.5·120, 160, cached or not, from the same canvases and
/// composites: the Screen's picture, and no raster. A build whose cached
/// Screen takes no picture draws it into its cache's raster, each paint
/// under a scope at the Screen's fade, and reads 0.5·200 + 0.5·(0.5·60 +
/// 0.5·120), 145.
#[test]
fn a_cached_screen_fades_as_one_picture() {
    let at_half = |cached: bool| {
        let (mut live, row) = box_on_backing(cached);
        set_row_opacity(&row, 0.5);
        let drawn = live.settled(PLAIN);
        let raster_count = rasters(&drawn, &row);
        (drawn, raster_count)
    };
    let (uncached, _) = at_half(false);
    assert_reads(
        uncached.at(IN_CHILD.0, IN_CHILD.1),
        gray(160.0),
        1.0,
        "uncached, the box",
    );
    one_picture(&uncached, "uncached");
    let (cached, raster_count) = at_half(true);
    assert_reads(
        cached.at(IN_CHILD.0, IN_CHILD.1),
        gray(160.0),
        1.0,
        "cached, the box",
    );
    assert_eq!(
        raster_count, 0,
        "cached: no raster while the Screen fades, {:?}",
        cached.canvases
    );
    assert_eq!(
        sorted_canvases(&cached),
        sorted_canvases(&uncached),
        "cached: the uncached Screen's canvases"
    );
    assert_eq!(
        cached.composites, uncached.composites,
        "cached: the uncached Screen's composites"
    );
}

/// Ruling U13: a mark written alone shows on a cached Screen's next draw.
/// `a_cached_screen_fades_as_one_picture`'s cached page, its Screen at 0.5,
/// draws twice. Then the box's mark is written to 2 alone, with no other key
/// and no advance, and the next draw reads the box at its own strength,
/// 200. While the Screen fades its cache makes no raster. A build whose
/// cached Screen draws into its cache's raster makes one on the first draw,
/// and after the write reads 145: nothing tells the cache its raster is
/// stale, so it composites the kept raster again.
#[test]
fn a_mark_written_alone_on_a_cached_screen_reads_on_the_next_draw() {
    let (mut live, row) = box_on_backing(true);
    set_row_opacity(&row, 0.5);
    let first = live.settled(PLAIN);
    let second = live.draw(PLAIN);
    assert!(CoreRegistry::set_double_handle(
        &box_record(&row, "marked"),
        i32::from(FADE_OUTSIDE),
        2.0
    ));
    let marked = live.draw(PLAIN);
    assert_reads(
        marked.at(IN_CHILD.0, IN_CHILD.1),
        gray(200.0),
        1.0,
        "marked 2 alone, at its own strength",
    );
    for (drawn, what) in [
        (&first, "the first draw"),
        (&second, "the second draw"),
        (&marked, "the draw after the mark"),
    ] {
        assert_eq!(
            rasters(drawn, &row),
            0,
            "{what}: no raster while the Screen fades, {:?}",
            drawn.canvases
        );
    }
}

/// Ruling U13 with a flight: Screen B, with an enabled bitmap cache, fades
/// while unit c flies over its backing, mid-step. At rest the cache serves
/// B: one raster of B's size. While B fades, at 0.5 and then 0.25, the
/// cache makes no raster, and the frame is the uncached twin's: the same
/// canvases, the flight at its own color, and every pixel within a level. A
/// build whose cached Screen draws through its cache while it fades makes
/// the raster again with the flight drawn into it: one where B takes no
/// picture, and one where B takes its picture but draws what is outside it
/// through `draw`, which enters the cache.
#[test]
fn a_cached_screen_with_a_flight_makes_no_cache_raster_while_it_fades() {
    let mut cached = Journey::new(&css_flight_over_cached_backing_fixture());
    let mut uncached = Journey::new(&css_flight_over_backing_fixture());
    let row = screen_row(&cached.live.root, "B");
    cache_at_resolution(&row);
    cached.follow(0.5, "cached, B at rest");
    uncached.follow(0.5, "uncached, B at rest");
    let (rest, _) = cached.draw();
    let _ = uncached.draw();
    assert_eq!(
        rasters(&rest, &row),
        1,
        "at rest the cache serves B: {:?}",
        rest.canvases
    );
    for f in [0.5, 0.25] {
        let what = format!("Screen B at {f}, mid-step");
        for journey in [&mut cached, &mut uncached] {
            journey.set_screen("B", f);
            journey.follow(0.5, &what);
        }
        let (drawn, read) = cached.draw();
        let (twin, _) = uncached.draw();
        assert_eq!(
            rasters(&drawn, &row),
            0,
            "{what}: no raster while B fades, {:?}",
            drawn.canvases
        );
        assert_eq!(
            sorted_canvases(&drawn),
            sorted_canvases(&twin),
            "{what}: the uncached twin's canvases"
        );
        assert_eq!(
            drawn.composites, twin.composites,
            "{what}: the uncached twin's composites"
        );
        assert_reads(
            read,
            channels(FADING_BOX_UNIT),
            1.0,
            &format!("{what}, the flight"),
        );
        let whole = (0, 0, twin.width, twin.pixels.len() / 4 / twin.width);
        assert_area(
            &drawn,
            whole,
            1.0,
            &format!("{what}, against the uncached twin"),
            |x, y| Some(twin.at(x, y)),
        );
    }
}

/// Ruling U13, a guard on a cached Screen at rest, driven by its own
/// opacity. At rest the cache draws its raster on the first frame and
/// composites it after, so a second frame with nothing changed makes no
/// canvas, as upstream's cache does. While the Screen fades, at 0.5, the
/// cache sits out on both frames: no raster, and the Screen's picture. Back
/// at 1 the first frame draws the raster again, since the opacity writes
/// changed what the Screen draws, and the next makes no canvas. The box
/// reads 200 at rest and 160 at 0.5. A build whose cached Screen draws
/// through its cache while it fades makes a raster at 0.5. One that turns a
/// Screen's cache off at every fade makes none at rest.
#[test]
fn a_cached_screen_at_rest_keeps_its_raster() {
    let (mut live, row) = box_on_backing(true);
    let box_reads = |drawn: &Drawn, want: f32, what: &str| {
        assert_reads(
            drawn.at(IN_CHILD.0, IN_CHILD.1),
            gray(want),
            1.0,
            &format!("{what}, the box"),
        );
    };
    let rest = live.settled(PLAIN);
    assert_eq!(
        rasters(&rest, &row),
        1,
        "at rest, the first frame: {:?}",
        rest.canvases
    );
    box_reads(&rest, 200.0, "at rest, the first frame");
    let kept = live.draw(PLAIN);
    assert!(
        kept.canvases.is_empty(),
        "at rest, nothing changed: {:?}",
        kept.canvases
    );
    box_reads(&kept, 200.0, "at rest, nothing changed");
    set_row_opacity(&row, 0.5);
    for frame in ["first", "second"] {
        let what = format!("at 0.5, the {frame} frame");
        let drawn = if frame == "first" {
            live.settled(PLAIN)
        } else {
            live.draw(PLAIN)
        };
        assert_eq!(
            rasters(&drawn, &row),
            0,
            "{what}: no raster, {:?}",
            drawn.canvases
        );
        one_picture(&drawn, &what);
        box_reads(&drawn, 160.0, &what);
    }
    set_row_opacity(&row, 1.0);
    let back = live.settled(PLAIN);
    assert_eq!(
        rasters(&back, &row),
        1,
        "back at 1, the first frame: {:?}",
        back.canvases
    );
    box_reads(&back, 200.0, "back at 1, the first frame");
    let after = live.draw(PLAIN);
    assert!(
        after.canvases.is_empty(),
        "back at 1, nothing changed: {:?}",
        after.canvases
    );
    box_reads(&after, 200.0, "back at 1, nothing changed");
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

/// `unit` hosted at `AT` at `opacity` in a root whose gray background is a draw:
/// the frame clears to black, and the prepass can hide a draw but not the clear.
fn over_a_drawn_backdrop(unit: Unit, opacity: f32) -> Vec<u8> {
    let scene = Scene::hosted((300.0, 300.0), unit, AT, opacity);
    Scene {
        root: Unit {
            background: Some(K),
            ..scene.root
        },
        photo: Some(photo()),
        ..scene
    }
    .bytes()
}

/// A held fade through the scope fallback, on the boxes' side: the paints stay
/// opaque under the scope and must each blend over what is beneath them. 240 at
/// 0.5 over 128 reads 184, 200 at 0.5 over 128 reads 164, and 200 at 0.5 over
/// the 184 reads 192. Raster ordering reads the same, which checks the arithmetic
/// on today's code.
#[test]
fn a_held_fade_with_no_box_blends_over_its_backdrop_on_a_depth_stencil_frame_the_boxes() {
    let unit = Unit {
        items: vec![
            block((20.0, 20.0), (90.0, 90.0), T1),
            block((60.0, 60.0), (90.0, 90.0), T2),
        ],
        ..Unit::css(160.0, 160.0)
    };
    let bytes = over_a_drawn_backdrop(unit, 0.5);
    let reads = [
        ("the 240 box alone", BLUE_ALONE, 184.0),
        ("the 200 box alone", RED_ALONE, 164.0),
        ("the overlap", OVERLAP, 192.0),
        ("outside the unit", (10, 10), 128.0),
    ];
    for (frame, view) in [("raster ordering", PLAIN), ("depth-stencil", DEPTH_STENCIL)] {
        let drawn = render(&bytes, BLACK, FRAME, view);
        no_picture(&drawn, frame);
        for (what, (x, y), expected) in reads {
            eprintln!(
                "{frame}: {what} reads {:?}, expected {expected}",
                drawn.at(x, y)
            );
        }
        for (what, (x, y), expected) in reads {
            assert_reads(
                drawn.at(x, y),
                [expected; 3],
                1.0,
                &format!("{frame}: {what}"),
            );
        }
    }
}

/// The same fade on the label's side: a white background, the photo and a title,
/// the unit of `an_unclipped_unit_that_ends_in_text_falls_back`. A glyph that
/// fades over black is Basic's black label.
#[test]
fn a_held_fade_with_no_box_blends_over_its_backdrop_on_a_depth_stencil_frame_the_label() {
    let unit = || Unit {
        background: Some(WHITE),
        items: vec![photo_item(), title_item((8.0, 96.0), 144.0)],
        ..Unit::css(160.0, 160.0)
    };
    let full = render(
        &over_a_drawn_backdrop(unit(), 1.0),
        BLACK,
        FRAME,
        DEPTH_STENCIL,
    );
    let half = render(
        &over_a_drawn_backdrop(unit(), 0.5),
        BLACK,
        FRAME,
        DEPTH_STENCIL,
    );
    no_picture(&half, "no box");
    let first_in_title = |matches: fn(&[f32; 3]) -> bool| {
        (TITLE.1..TITLE.3)
            .flat_map(|y| (TITLE.0..TITLE.2).map(move |x| (x, y)))
            .find(|(x, y)| matches(&full.at(*x, *y)))
            .expect("the title's area holds this color at full opacity")
    };
    let white = first_in_title(|v| v.iter().all(|c| *c >= 253.0));
    let glyph = first_in_title(|v| v.iter().all(|c| *c <= 2.0));
    eprintln!(
        "the white reads {:?}, expected 191.5; a glyph reads {:?}, expected 95.75",
        half.at(white.0, white.1),
        half.at(glyph.0, glyph.1),
    );
    // The title first: it reads the design's numbers, where the photo reads the
    // white beneath it.
    assert_per_paint(&half, &full, |_, _| channels(K), None, TITLE, "no box");
    assert_per_paint(
        &half,
        &full,
        |_, _| channels(K),
        Some(PHOTO),
        TITLE,
        "no box",
    );
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

/// Night T6 (f) and design rule 5: a clipping root box beside a title at the
/// top level, so the draw list is not one root box and the unit falls back to
/// a scope at 0.5. The box, at own 1 under that scope, takes its own picture:
/// a canvas of its 160 × 100 plus guards, not the unit's 160 × 160. Blue over
/// its white reads 0.5·blue + 0.5·K, where paint by paint reads
/// 0.5·blue + 0.25·255 + 0.25·K. The title stays paint by paint.
#[test]
fn a_clipping_box_beside_a_title_takes_its_picture_under_the_fallback() {
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
    one_picture(&half, "the box's picture");
    assert_eq!(
        half.canvases,
        [(192, 128)],
        "the box's canvas, not the unit's"
    );
    let k = channels(K);
    assert_reads(
        half.at(90, 90),
        each(|c| 0.5 * channels(BLUE)[c] + 0.5 * k[c]),
        2.0,
        "blue over the box's white, as one picture",
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

/// A plain box's tile: ListToGrid's shape, a background of 50 with its text,
/// 255, on top. The text is a second box, so the read sits wholly inside it.
const TILE: u32 = 0xff32_3232;
/// The design's B, the Screen's backing.
const B: u32 = 0xff3c_3c3c;
/// Inside the tile's text, and inside T23's T2.
const IN_CHILD: (usize, usize) = (100, 100);

/// A box named "fading" at `AT`, 160 points square, painting `background`
/// with `child` on top of it at (20, 20), 90 points square.
fn fading_box(opacity: f32, overflow: Overflow, background: u32, child: u32) -> Item {
    Item::Block(Block {
        name: Some("fading"),
        at: AT,
        size: (160.0, 160.0),
        color: Some(background),
        opacity: Some(opacity),
        overflow,
        items: vec![block((20.0, 20.0), (90.0, 90.0), child)],
        ..Block::default()
    })
}

/// A CSS root of `FRAME`'s size, painting `backing` under `items`.
fn root_of(items: Vec<Item>, backing: Option<u32>, photo: Option<Vec<u8>>) -> Vec<u8> {
    Scene {
        root: Unit {
            background: backing,
            items,
            ..Unit::css(300.0, 300.0)
        },
        units: Vec::new(),
        photo,
    }
    .bytes()
}

fn tile(opacity: f32, backing: Option<u32>) -> Vec<u8> {
    root_of(
        vec![fading_box(opacity, Overflow::Hidden, TILE, WHITE)],
        backing,
        None,
    )
}

fn gray(level: f32) -> [f32; 3] {
    [level; 3]
}

/// Design T8: a box that clips both axes fades as one picture, so over 120
/// the text reads o·255 + (1 - o)·120 and over white 255. Paint by paint, as
/// main draws it, it reads 140.6, 170 and 208.1 over 120, and 217, 204 and
/// 217 over white. A canvas whose paints keep o, composited at o, reads
/// 125.2, 145 and 186.1.
#[test]
fn a_clipping_box_fades_as_one_picture() {
    for o in [0.25, 0.5, 0.75] {
        for (under, clear) in [(120.0, U), (255.0, WHITE)] {
            let what = format!("own fade {o} over {under}");
            let drawn = render(&tile(o, None), clear, FRAME, PLAIN);
            assert_reads(
                drawn.at(IN_CHILD.0, IN_CHILD.1),
                gray(o * 255.0 + (1.0 - o) * under),
                1.0,
                &format!("{what}, the text"),
            );
            one_picture(&drawn, &what);
        }
    }
}

/// Design T8b: a box at own fade 1 under a scope takes its picture too, so it
/// doesn't step when its own fade reaches 1 while the fade around it runs. The
/// scope, 0.5, stands in for the Screen's fade, and the root's backing, B 60,
/// draws under it, so below the box reads f·B + (1 - f)·U, 90. The text reads
/// 0.5·255 + 0.5·90, 172.5, and 255 over white. A build that reads the own
/// fade alone draws paint by paint: 162.5, or 203.75 over white.
#[test]
fn a_clipping_box_at_own_one_under_a_scope_reads_as_one_picture() {
    for (backing, clear, want) in [(B, U, 172.5), (WHITE, WHITE, 255.0)] {
        let what = format!("own 1 under a scope at 0.5, backing {backing:#010x}");
        let drawn = render(&tile(1.0, Some(backing)), clear, FRAME, under_scope(0.5));
        assert_reads(
            drawn.at(IN_CHILD.0, IN_CHILD.1),
            gray(want),
            1.0,
            &format!("{what}, the text"),
        );
        one_picture(&drawn, &what);
    }
    let near = render(&tile(0.999, Some(B)), U, FRAME, under_scope(0.5));
    let at_one = render(&tile(1.0, Some(B)), U, FRAME, under_scope(0.5));
    // Tests read to within a level, and a build that reads the own fade alone
    // steps by 10.
    assert_reads(
        near.at(IN_CHILD.0, IN_CHILD.1),
        at_one.at(IN_CHILD.0, IN_CHILD.1),
        1.0,
        "the text at own 0.999 against own 1",
    );
}

const PROGRESS: i32 = 62003;

/// A step of a page with one flight: on a versions page, unit c flying alone
/// from A to B by "t2", driven as the page player drives a frame, the page's
/// own machine bound to its model. Fields drop in order, the page last.
struct Journey {
    flight: Flight,
    moves: LiveMoves,
    machine: RuntimeStateMachineInstanceHandle,
    live: Live,
}
impl Journey {
    fn new(bytes: &[u8]) -> Self {
        Self::on(bytes, (400, 600), "B", "t2")
    }
    /// The page `bytes`, drawn at `size` over U, taking off for `to` by
    /// `step`, which flies one unit.
    fn on(bytes: &[u8], size: (u32, u32), to: &str, step: &str) -> Self {
        let live = Live::new(bytes, U, size, false, RenderMode::RasterOrdering, Some("Page"));
        let model = live
            .file
            .with_file(|file| {
                file.create_default_view_model_instance_for_artboard(live.root.core_handle())
            })
            .expect("the page's model");
        let machine = live.root.default_state_machine_handle().expect("a machine");
        machine.with_instance_mut(|machine| machine.bind_view_model_instance(model));
        machine.advance_and_apply(0.0);
        let mut moves = LiveMoves::default();
        moves.prepare(&live.root).unwrap();
        moves.step_begin(&live.root, to, Some(step)).unwrap();
        let mut flights = moves.flights(&live.root);
        assert_eq!(flights.len(), 1, "one unit flies alone");
        let flight = flights.remove(0);
        Self {
            flight,
            moves,
            machine,
            live,
        }
    }
    /// Writes `value` to the own opacity of the Screen `name`'s row.
    fn set_screen(&self, name: &str, value: f32) {
        set_row_opacity(&screen_row(&self.live.root, name), value);
    }
    /// Moves the flight to `progress` and runs a frame's advance and its
    /// follower, which must report no error.
    fn follow(&mut self, progress: f32, what: &str) {
        assert!(CoreRegistry::set_double_handle(
            &self.flight.controls(),
            PROGRESS,
            progress
        ));
        self.machine.advance_and_apply(0.0);
        if let Err(error) = self.moves.after_advance(&self.live.root) {
            panic!("{what}: the follower reports {error:#}");
        }
    }
    /// The flight's pose opacity, as the follower last wrote it.
    fn pose_opacity(&self) -> f32 {
        slot_pose(&self.flight.flight).unwrap().opacity
    }
    /// Draws the frame, and reads it at the center of the flight's pose.
    fn draw(&mut self) -> (Drawn, [f32; 3]) {
        let pose = slot_pose(&self.flight.flight).unwrap();
        let center = pose.transform * Vec2D::new(pose.size.x / 2.0, pose.size.y / 2.0);
        let drawn = self.live.draw(PLAIN);
        let read = drawn.at(center.x as usize, center.y as usize);
        (drawn, read)
    }
}

/// Design T9: unit c, gray 200, flies alone from a home in a box that clips
/// both axes at 0.5, on Screen A at `screen_a`, over 120. At progress 0 it
/// reads 0.5·200 + 0.5·120, 160, and its pose 0.5.
fn assert_flight_takes_its_boxs_fade(screen_a: f32) {
    let mut journey = Journey::new(&css_flight_under_fading_box_fixture(0.5));
    if screen_a != 1.0 {
        journey.set_screen("A", screen_a);
    }
    let what = format!("Screen A at {screen_a}, the flight at progress 0");
    journey.follow(0.0, &what);
    let (_, read) = journey.draw();
    let unit = channels(FADING_BOX_UNIT);
    let k = channels(U);
    assert_reads(read, each(|c| 0.5 * unit[c] + 0.5 * k[c]), 1.0, &what);
    let pose = journey.pose_opacity();
    assert!((pose - 0.5).abs() < 0.002, "{what}: the pose reads {pose}");
}

/// Design T9, a guard: a flight's pose carries the fade of the box its home
/// sits in (decisions 68 and 69), with its Screen at 1. A build that holds the
/// box's fade out of the slot's number and doesn't read it back reads 200.
#[test]
fn a_flight_takes_the_fade_of_the_box_its_home_sits_in() {
    assert_flight_takes_its_boxs_fade(1.0);
}

/// Design T9: with its Screen at 0.5 the flight still reads 160. The
/// Screen's fade is its picture's, which the flight draws outside. A read that
/// runs past the Screen takes its 0.5 too: a pose of 0.25, reading 140.
#[test]
fn a_flight_takes_its_boxs_fade_and_not_its_screens() {
    assert_flight_takes_its_boxs_fade(0.5);
}

/// Design T11: a flight draws outside its Screen's picture, at its own
/// strength. Unit c, gray 200, flies from A to B over B's backing, B 60, over
/// U 120, with Screen B at `screen_b`, and reads 200 at each progress. In the
/// Screen's picture it reads 160 at 0.5 and 120 at 0. Under pass two's scope
/// it reads 145 at 0.5. The follower divides by its parent's opacity, so on a
/// Screen whose number it reads it errors at 0.
fn assert_flight_reads_its_own_color(screen_b: f32) {
    assert_eq!(SCREEN_B_BACKING, B, "B's backing is the design's B");
    let mut journey = Journey::new(&css_flight_over_backing_fixture());
    journey.set_screen("B", screen_b);
    for progress in [0.0, 0.5, 1.0] {
        let what = format!("Screen B at {screen_b}, the flight at progress {progress}");
        journey.follow(progress, &what);
        let (_, read) = journey.draw();
        assert_reads(read, channels(FADING_BOX_UNIT), 1.0, &what);
    }
}

/// Design T11 at 0.5. Main's follower divides the flight's strength by B's
/// 0.5, so it reads 200 at take-off and 172.5 at progress 0.5.
#[test]
fn a_flight_draws_outside_its_fading_screen() {
    assert_flight_reads_its_own_color(0.5);
}

/// Design T11 at 0: the flight reads 200 and the follower reports no error.
/// Main's follower errors on B's 0.
#[test]
fn a_flight_draws_outside_its_screen_at_zero() {
    assert_flight_reads_its_own_color(0.0);
}

/// Design T16: a Screen at 0 makes no canvas of its plain content, while the
/// flight on it still draws, at its landing pose here. Main draws the flight
/// at B's 0, so only U shows, 120.
#[test]
fn a_screen_at_zero_makes_no_canvas_and_its_flight_draws() {
    assert_screen_at_zero_makes_no_canvas_and_its_flight_draws(
        &css_flight_over_backing_fixture(),
        "Screen B at 0",
    );
}

fn assert_screen_at_zero_makes_no_canvas_and_its_flight_draws(bytes: &[u8], what: &str) {
    let mut journey = Journey::new(bytes);
    journey.set_screen("B", 0.0);
    let what = format!("{what}, the flight at progress 1");
    journey.follow(1.0, &what);
    let (drawn, read) = journey.draw();
    no_picture(&drawn, &what);
    assert_reads(read, channels(FADING_BOX_UNIT), 1.0, &what);
}

/// Design T16, a guard: at an effective fade of 1 a box takes no picture.
#[test]
fn a_clipping_box_at_own_one_takes_no_picture() {
    let drawn = render(&tile(1.0, None), U, FRAME, PLAIN);
    assert_reads(
        drawn.at(IN_CHILD.0, IN_CHILD.1),
        gray(255.0),
        1.0,
        "the text",
    );
    no_picture(&drawn, "own 1, no scope");
}

/// T23's box: T1 with T2 on top, at 0.5, over U 120. Paint by paint the
/// overlap reads 190. As one picture it reads 160.
fn t23_box(overflow: Overflow) -> Live {
    Live::new(
        &root_of(vec![fading_box(0.5, overflow, T1, T2)], None, None),
        U,
        FRAME,
        false,
        RenderMode::RasterOrdering,
        None,
    )
}

/// Writes `value` to `key` of the record of the box named `name`.
fn write_style(live: &Live, name: &str, key: u16, value: u32) {
    assert!(CoreRegistry::set_uint_handle(
        &box_record(&live.root, name),
        i32::from(key),
        value
    ));
}

fn assert_paint_by_paint(drawn: &Drawn, what: &str) {
    assert_reads(drawn.at(IN_CHILD.0, IN_CHILD.1), gray(190.0), 1.0, what);
    no_picture(drawn, what);
}

fn assert_t23_picture(drawn: &Drawn, what: &str) {
    assert_reads(drawn.at(IN_CHILD.0, IN_CHILD.1), gray(160.0), 1.0, what);
    one_picture(drawn, what);
}

/// Design T23: whether a box holds its fade follows its overflow, with no
/// opacity write. A split worked out while the box was visible and never
/// refreshed draws its paints at 0.5 into the canvas and composites at 0.5
/// again, 155.
#[test]
fn a_box_that_starts_to_clip_takes_its_picture() {
    let mut live = t23_box(Overflow::Visible);
    assert_paint_by_paint(&live.settled(PLAIN), "visible");
    write_style(&live, "fading", OVERFLOW_X, 2);
    write_style(&live, "fading", OVERFLOW_Y, 2);
    assert_t23_picture(&live.settled(PLAIN), "overflow visible to hidden");
}

/// Design T23, a guard: a box that stops clipping fades paint by paint. A
/// split worked out while it clipped and never refreshed loses the fade, 200.
#[test]
fn a_box_that_stops_clipping_fades_paint_by_paint() {
    let mut live = t23_box(Overflow::Hidden);
    live.settled(PLAIN);
    write_style(&live, "fading", OVERFLOW_X, 0);
    write_style(&live, "fading", OVERFLOW_Y, 0);
    assert_paint_by_paint(&live.settled(PLAIN), "overflow hidden to visible");
}

/// Design T23: `overflow-x: hidden` alone computes to clipping both axes, so
/// the box takes its picture. A build that reads the keys as written reads
/// 190.
#[test]
fn overflow_x_hidden_alone_clips_both_axes() {
    let mut live = t23_box(Overflow::Visible);
    assert_paint_by_paint(&live.settled(PLAIN), "visible");
    write_style(&live, "fading", OVERFLOW_X, 2);
    assert_t23_picture(&live.settled(PLAIN), "overflow-x: hidden alone");
}

/// Design T23, a guard: `overflow-x: clip` alone clips one axis, so the box
/// has no box and fades paint by paint. A build that reads one axis as
/// clipping reads 160.
#[test]
fn overflow_x_clip_alone_clips_one_axis() {
    let mut live = t23_box(Overflow::Visible);
    live.settled(PLAIN);
    write_style(&live, "fading", OVERFLOW_X, 1);
    assert_paint_by_paint(&live.settled(PLAIN), "overflow-x: clip alone");
}

/// A box that clips both axes, painting red 240 with one image, 80, at
/// (40, 40), then `more`.
fn lone_box(opacity: f32, more: Vec<Item>) -> Vec<u8> {
    let mut items = vec![Item::Photo { at: (40.0, 40.0) }];
    items.extend(more);
    root_of(
        vec![Item::Block(Block {
            name: Some("fading"),
            at: AT,
            size: (160.0, 160.0),
            color: Some(0xfff0_0000),
            opacity: Some(opacity),
            overflow: Overflow::Hidden,
            items,
            ..Block::default()
        })],
        None,
        Some(png(80, 80, |_, _| [80, 0, 0])),
    )
}

/// The image's red over 120: 130 paint by paint, 100 as one picture.
fn assert_image_red(drawn: &Drawn, want: f32, what: &str) {
    let image = drawn.at(120, 120);
    assert!(
        (image[0] - want).abs() <= 1.0,
        "{what}: the image reads {image:?}, expected red {want}"
    );
}

/// Design T7, a guard, on a box: a lone picture keeps its per-paint fade at
/// any effective fade, so red shows through its image by f(1 - f). A build
/// that groups it reads 100.
#[test]
fn a_lone_picture_box_keeps_its_paints_at_half_and_under_a_scope() {
    for (what, opacity, view) in [
        ("own fade 0.5", 0.5, PLAIN),
        ("own 1 under a scope at 0.5", 1.0, under_scope(0.5)),
        (
            "own 1 under a scope at 0.5, on an atomics frame",
            1.0,
            View {
                atomics: true,
                ..under_scope(0.5)
            },
        ),
    ] {
        let drawn = render(&lone_box(opacity, Vec::new()), U, FRAME, view);
        assert_image_red(&drawn, 130.0, what);
        let red = drawn.at(50, 50);
        assert!(
            (red[0] - 180.0).abs() <= 1.0,
            "{what}: the background reads {red:?}, expected red 180"
        );
        no_picture(&drawn, what);
    }
}

/// Design T23: a lone picture that gains a second child, with no opacity
/// write, takes its picture. The child shows by its display, below the read.
/// A build that left the box's fade in its paints while it was lone, and never
/// refreshed it, reads 125.
#[test]
fn a_lone_picture_box_that_gains_a_child_takes_its_picture() {
    let second = Item::Block(Block {
        name: Some("second"),
        at: (0.0, 140.0),
        size: (40.0, 20.0),
        color: Some(T2),
        display_none: true,
        ..Block::default()
    });
    let mut live = Live::new(
        &lone_box(0.5, vec![second]),
        U,
        FRAME,
        false,
        RenderMode::RasterOrdering,
        None,
    );
    let lone = live.settled(PLAIN);
    assert_image_red(&lone, 130.0, "lone");
    no_picture(&lone, "lone");
    write_style(&live, "second", DISPLAY, 0);
    let grouped = live.settled(PLAIN);
    assert_image_red(&grouped, 100.0, "with a second child");
    one_picture(&grouped, "with a second child");
}

/// T10's colors: a gray row of 85 on a white Screen, over blue 40, a level of
/// 40 on each channel here, since the oracle reads one level.
const ROW_GRAY: u32 = 0xff55_5555;
const BLUE_40: u32 = 0xff28_2828;

/// T10's Screen: white, holding the gray row at `AT`, 160 points square.
fn t10_screen() -> Unit {
    Unit {
        background: Some(WHITE),
        items: vec![block(AT, (160.0, 160.0), ROW_GRAY)],
        ..Unit::css(300.0, 300.0)
    }
}

/// A page of `FRAME`'s size whose root clips, as the compiler writes one,
/// holding `screen` as its one Screen.
fn page_of(screen: Unit) -> Vec<u8> {
    Page::one((300.0, 300.0), screen).bytes()
}

/// `page` with its Screen at `p`, settled.
fn page_at(page: Page, p: f32) -> (Live, RuntimeArtboardInstanceHandle) {
    let mut live = Live::page(&page.bytes(), BLUE_40);
    let row = screen_row(&live.root, "Screen");
    set_row_opacity(&row, p);
    live.settled(PLAIN);
    (live, row)
}

/// T10's page with its Screen at `p`, settled.
fn t10_page(p: f32) -> (Live, RuntimeArtboardInstanceHandle) {
    page_at(Page::one((300.0, 300.0), t10_screen()), p)
}

/// T10's page with its Screen keyed and at `p`, settled.
fn t23_page(p: f32) -> (Live, RuntimeArtboardInstanceHandle) {
    page_at(
        Page {
            keyed: true,
            ..Page::one((300.0, 300.0), t10_screen())
        },
        p,
    )
}

/// Design T10: a Screen fades as one picture, so inside the row it reads
/// exactly p·85 + (1 - p)·40, from one canvas: 51.25, 62.5 and 73.75. Main
/// fades it paint by paint: 91.6, 116.3 and 114.1. A canvas whose paints keep
/// p, composited at p, reads 52.9, 78.1 and 95.6.
#[test]
fn a_screen_fades_as_one_picture() {
    for p in [0.25, 0.5, 0.75] {
        let what = format!("the Screen at {p}");
        let (mut live, row) = t10_page(p);
        let drawn = live.draw(PLAIN);
        assert_eq!(
            row_opacity(&row),
            p,
            "{what}: the row keeps its own opacity"
        );
        assert_reads(
            drawn.at(IN_CHILD.0, IN_CHILD.1),
            gray(p * 85.0 + (1.0 - p) * 40.0),
            1.0,
            &format!("{what}, the row"),
        );
        one_picture(&drawn, &what);
    }
}

/// Design T13: a box that clips both axes at o, with no own motion, in a
/// Screen at f is a picture in the Screen's picture. Over the Screen's backing
/// B 60 and U 120, at o = f = 0.5, its child reads f·(o·T2 + (1 - o)·B) +
/// (1 - f)·U, 125, from two canvases and one composite on the screen. One
/// group with no nested picture reads 147.5, each paint at o·f 145.6, f twice
/// in the composite 107.5, and o lost 160.
#[test]
fn a_fading_box_in_a_fading_screen_is_a_picture_in_its_picture() {
    let mut live = Live::page(
        &page_of(Unit {
            background: Some(B),
            items: vec![fading_box(0.5, Overflow::Hidden, T1, T2)],
            ..Unit::css(300.0, 300.0)
        }),
        U,
    );
    set_row_opacity(&screen_row(&live.root, "Screen"), 0.5);
    let drawn = live.settled(PLAIN);
    assert_reads(
        drawn.at(IN_CHILD.0, IN_CHILD.1),
        gray(125.0),
        1.0,
        "the box's child",
    );
    assert_eq!(drawn.canvases.len(), 2, "{:?}", drawn.canvases);
    assert_eq!(drawn.composites, 1, "on the screen");
}

/// How a test writes a Screen row's property: by a keyframe, which the
/// Screen's own animation applies to the row as a playing animation does, or
/// by a binding, which writes through the handle as a native binding does.
#[derive(Clone, Copy, Debug)]
enum Route {
    Keyframe,
    Binding,
}

/// Applies the keyed Screen's animation `name` to `row`, at its one key.
fn play(row: &RuntimeArtboardInstanceHandle, name: &str) {
    row.animation_named(name)
        .unwrap_or_else(|| panic!("the Screen's animation {name}"))
        .apply(1.0);
}

/// Writes the row's `clip` by `route`.
fn write_clip(row: &RuntimeArtboardInstanceHandle, on: bool, route: Route) {
    let [clip_on, clip_off, _, _] = KEYED_SCREEN;
    match route {
        Route::Keyframe => play(row, if on { clip_on } else { clip_off }),
        Route::Binding => assert!(CoreRegistry::set_bool_handle(
            &row.core_handle(),
            i32::from(LayoutComponentBase::CLIP_PROPERTY_KEY),
            on
        )),
    }
}

/// Writes the overflow of the row's record on both axes by `route`: hidden,
/// or visible.
fn write_overflow(row: &RuntimeArtboardInstanceHandle, hidden: bool, route: Route) {
    let [_, _, overflow_hidden, overflow_visible] = KEYED_SCREEN;
    match route {
        Route::Keyframe => play(
            row,
            if hidden {
                overflow_hidden
            } else {
                overflow_visible
            },
        ),
        Route::Binding => {
            let record = artboard_record(row);
            for key in [OVERFLOW_X, OVERFLOW_Y] {
                assert!(CoreRegistry::set_uint_handle(
                    &record,
                    i32::from(key),
                    if hidden { 2 } else { 0 }
                ));
            }
        }
    }
}

/// T23's row paint by paint, as today: 0.5·85 + 0.5·(0.5·255 + 0.5·40).
fn assert_row_paint_by_paint(drawn: &Drawn, what: &str) {
    assert_reads(drawn.at(IN_CHILD.0, IN_CHILD.1), gray(116.25), 1.0, what);
    no_picture(drawn, what);
}

/// T23's row as one picture: 0.5·85 + 0.5·40.
fn assert_row_picture(drawn: &Drawn, what: &str) {
    assert_reads(drawn.at(IN_CHILD.0, IN_CHILD.1), gray(62.5), 1.0, what);
    one_picture(drawn, what);
}

/// Design T23: a Screen row has the page's frame for its box while it has no
/// clip of its own, so its `clip` decides whether it fades as one picture,
/// with no opacity write. T10's row at 0.5: `clip` written on, it fades as
/// today, 116.25, and off again as one picture, 62.5. A build that marks the
/// row only from the typed setter reads 85 and then 78.1 through the binding.
fn assert_row_clip_decides_its_picture(route: Route) {
    let (mut live, row) = t23_page(0.5);
    write_clip(&row, true, route);
    assert_row_paint_by_paint(&live.settled(PLAIN), &format!("clip on by {route:?}"));
    write_clip(&row, false, route);
    assert_row_picture(&live.settled(PLAIN), &format!("clip off by {route:?}"));
}

/// Design T23, the row's `clip` written by a keyframe.
#[test]
fn a_screen_rows_clip_by_a_keyframe_decides_its_picture() {
    assert_row_clip_decides_its_picture(Route::Keyframe);
}

/// Design T23, the row's `clip` written by a binding.
#[test]
fn a_screen_rows_clip_by_a_binding_decides_its_picture() {
    assert_row_clip_decides_its_picture(Route::Binding);
}

/// Design T23: the same with the row's record clipping both axes, and then
/// neither.
fn assert_row_overflow_decides_its_picture(route: Route) {
    let (mut live, row) = t23_page(0.5);
    write_overflow(&row, true, route);
    assert_row_paint_by_paint(
        &live.settled(PLAIN),
        &format!("overflow hidden by {route:?}"),
    );
    write_overflow(&row, false, route);
    assert_row_picture(
        &live.settled(PLAIN),
        &format!("overflow visible by {route:?}"),
    );
}

/// Design T23, the row's overflow written by a keyframe.
#[test]
fn a_screen_rows_overflow_by_a_keyframe_decides_its_picture() {
    assert_row_overflow_decides_its_picture(Route::Keyframe);
}

/// Design T23, the row's overflow written by a binding.
#[test]
fn a_screen_rows_overflow_by_a_binding_decides_its_picture() {
    assert_row_overflow_decides_its_picture(Route::Binding);
}

/// Design T23: a row at 0.5 that has updated with no host, and is then
/// attached to the page's list, takes its picture once attached. A build that
/// marks nothing at attach keeps the split it worked out with no host: paints
/// at 0.5 in a canvas composited at 0.5 again, 78.1.
#[test]
fn a_screen_row_attached_after_it_updates_takes_its_picture() {
    let page = Page {
        bound: false,
        ..Page::one((300.0, 300.0), t10_screen())
    };
    let mut live = Live::page(&page.bytes(), BLUE_40);
    let list = live
        .root
        .with_artboard(|root| root.find_handle::<ArtboardComponentList>("screens"))
        .expect("the page's list");
    let arena = list.retain_arena().expect("the list's arena");
    let item = arena.insert(ViewModelInstanceListItem::default());
    ArtboardComponentList::update_list_occurrence(&list, &[item]);
    let source = live
        .file
        .with_file(|file| file.artboard_at_source(1))
        .expect("the Screen");
    let row = Artboard::nested_instance_from_handle(&source).expect("a row");
    set_row_opacity(&row, 0.5);
    settle(&row);
    ArtboardComponentList::add_artboard_at_occurrence(&list, row.clone(), 0, true);
    let drawn = live.settled(PLAIN);
    assert_eq!(row_opacity(&row), 0.5, "the row keeps its own opacity");
    assert_row_picture(&drawn, "attached after it updated");
}

/// A box's mark, its `fadeOutside`, which its Screen's `MARKS` keys.
fn marked(item: Item, mark: f32) -> Item {
    let Item::Block(block) = item else {
        panic!("a mark is a box's");
    };
    Item::Block(Block {
        mark: Some(mark),
        ..block
    })
}

/// A page whose one Screen paints `backing` under `items`, over `clear`,
/// with the Screen at `f` and its marks applied, as a step's timeline
/// writes them.
fn marked_screen(items: Vec<Item>, backing: u32, clear: u32, f: f32) -> Live {
    let live = Live::page(
        &page_of(Unit {
            background: Some(backing),
            items,
            ..Unit::css(300.0, 300.0)
        }),
        clear,
    );
    let row = screen_row(&live.root, "Screen");
    set_row_opacity(&row, f);
    play(&row, MARKS);
    live
}

/// Design T8b on a Screen: a box at own fade 1, marked 1 on a Screen with no
/// flight, so below its first flight, draws outside the Screen's picture
/// under a scope at the Screen's fade, rule 8's layer, and takes its picture
/// there. With the Screen at 0.5 over B 60 and U 120, the tile's text, 255
/// on 50, reads 0.5·255 + 0.5·(0.5·60 + 0.5·120), 172.5, and 255 over
/// white. The Screen's picture and the box's are a canvas each, and one
/// `DrawImage` names each. Between own 0.999 and own 1 the text moves less
/// than a level. A build that leaves the box in the Screen's picture, as
/// main draws an unmarked one, reads 187.5. One that reads the own fade
/// alone reads 162.5, or 203.75 over white.
#[test]
fn a_marked_box_at_own_one_takes_its_picture_under_its_screens_fade() {
    let tile_at = |o: f32, backing: u32, clear: u32| {
        marked_screen(
            vec![marked(fading_box(o, Overflow::Hidden, TILE, WHITE), 1.0)],
            backing,
            clear,
            0.5,
        )
        .settled(PLAIN)
    };
    for (backing, clear, want) in [(B, U, 172.5), (WHITE, WHITE, 255.0)] {
        let what = format!("marked 1 at own 1, Screen at 0.5, backing {backing:#010x}");
        let drawn = tile_at(1.0, backing, clear);
        assert_reads(
            drawn.at(IN_CHILD.0, IN_CHILD.1),
            gray(want),
            1.0,
            &format!("{what}, the text"),
        );
        assert_eq!(
            drawn.canvases.len(),
            2,
            "{what}: the Screen's picture and the box's, {:?}",
            drawn.canvases
        );
        assert_eq!(drawn.composites, 2, "{what}: one DrawImage names each");
    }
    let near = tile_at(0.999, B, U);
    let at_one = tile_at(1.0, B, U);
    assert_reads(
        near.at(IN_CHILD.0, IN_CHILD.1),
        at_one.at(IN_CHILD.0, IN_CHILD.1),
        1.0,
        "the text at own 0.999 against own 1",
    );
}

/// Design T12's element on a Screen at `f` over B 60 and U 120: a box of T1
/// with T2 on top, at own 0.5, marked `mark`, clipping both axes or not.
/// Read in the overlap.
fn own_motion(mark: f32, overflow: Overflow, f: f32) -> [f32; 3] {
    marked_screen(
        vec![marked(fading_box(0.5, overflow, T1, T2), mark)],
        B,
        U,
        f,
    )
    .settled(PLAIN)
    .at(IN_CHILD.0, IN_CHILD.1)
}

/// Design T12, forward: a clipping box with own motion, marked 1, below the
/// first flight, draws outside the Screen's picture under a scope at f, and
/// its own picture at o goes down at o·f: o·f·T2 + (1 - o·f)·(f·B + (1 -
/// f)·U), 117.5 at o = f = 0.5. One group in the Screen's picture reads
/// 125, paint by paint 145.6, the scope lost 145, and the fade twice 103.75.
#[test]
fn own_motion_forward_draws_outside_the_screens_picture() {
    assert_reads(
        own_motion(1.0, Overflow::Hidden, 0.5),
        gray(117.5),
        1.0,
        "marked 1 at own 0.5, Screen at 0.5",
    );
}

/// Design T12, Back: marked 2, the box draws outside the Screen's picture
/// at its own strength, over the leaving Screen's picture: o·T2 + (1 -
/// o)·(f·B + (1 - f)·U), 145 at f = 0.5, and 160 at f = 0. Read as mark 1
/// it reads 117.5, in the picture 125 and, at f = 0, 120, gone with the
/// Screen. Paint by paint it reads 182.5.
#[test]
fn own_motion_back_draws_at_its_own_strength() {
    for (f, want) in [(0.5, 145.0), (0.0, 160.0)] {
        assert_reads(
            own_motion(2.0, Overflow::Hidden, f),
            gray(want),
            1.0,
            &format!("marked 2 at own 0.5, Screen at {f}"),
        );
    }
}

/// Design T12 with no box: an element that doesn't clip, marked 1, fades
/// paint by paint under the scope, rule 5's fallback, each paint at o·f:
/// T1 at 0.25 over 90 is 127.5, and T2 at 0.25 over that 145.6. In the
/// Screen's picture it reads 147.5.
#[test]
fn own_motion_with_no_box_fades_paint_by_paint_under_the_scope() {
    assert_reads(
        own_motion(1.0, Overflow::Visible, 0.5),
        gray(145.625),
        1.0,
        "marked 1 at own 0.5 with no clip, Screen at 0.5",
    );
}

/// Design T16 at an effective fade of 0: a marked box under a Screen at 0,
/// a clipping box of T1 and T2 at own 0.5 marked 1, draws nothing and makes
/// no canvas. A build that asks for the box's canvas before reading the
/// scope's 0 makes one.
#[test]
fn a_marked_box_under_a_screen_at_zero_makes_no_canvas() {
    let drawn = marked_screen(
        vec![marked(fading_box(0.5, Overflow::Hidden, T1, T2), 1.0)],
        B,
        U,
        0.0,
    )
    .settled(PLAIN);
    no_picture(&drawn, "marked 1 under a Screen at 0");
    assert_reads(
        drawn.at(IN_CHILD.0, IN_CHILD.1),
        gray(120.0),
        1.0,
        "marked 1 under a Screen at 0",
    );
}

/// Design T14: a child of T2 at own 1, marked 1, in a parent of 240 that
/// clips both axes and doesn't fade, on a Screen at 0.5 over B 60 and U 120.
/// The parent sits at `AT`, 160 square, and the child at (80, 80) in it, 120
/// square, so it passes the parent's edge. Past that edge the Screen's
/// picture reads alone, 90. On the parent alone, 180. On the child inside
/// the clip, 0.5·200 + 0.5·180, 190. With the clip lost the child reads 145
/// past the edge. With the parent's own paints drawn again in pass two the
/// child reads 210 under the scope, or 240 at full strength. In the
/// Screen's picture, as main draws it, the child reads 160.
#[test]
fn a_marked_child_draws_outside_the_picture_inside_its_parents_clip() {
    let drawn = marked_screen(
        vec![Item::Block(Block {
            name: Some("parent"),
            at: AT,
            size: (160.0, 160.0),
            color: Some(T1),
            overflow: Overflow::Hidden,
            items: vec![marked(block((80.0, 80.0), (120.0, 120.0), T2), 1.0)],
            ..Block::default()
        })],
        B,
        U,
        0.5,
    )
    .settled(PLAIN);
    assert_reads(
        drawn.at(220, 220),
        gray(90.0),
        1.0,
        "past the parent's edge",
    );
    assert_reads(drawn.at(60, 60), gray(180.0), 1.0, "on the parent alone");
    assert_reads(
        drawn.at(150, 150),
        gray(190.0),
        1.0,
        "on the child, inside the clip",
    );
}

/// Design T14b's page: a box at `AT`, 160 square, at own `box_fade`, that
/// clips both axes and paints nothing, holding a child at (40, 40), 160
/// square, at own 1, marked 2, that clips both axes and paints T1, with T2
/// at (20, 20) in it, 140 square. The child's T2 passes the box's edge. On
/// a Screen at 0 over U 120.
fn mark_under_a_fading_box(box_fade: f32) -> Drawn {
    let child = Item::Block(Block {
        name: Some("marked"),
        at: (40.0, 40.0),
        size: (160.0, 160.0),
        color: Some(T1),
        overflow: Overflow::Hidden,
        items: vec![block((20.0, 20.0), (140.0, 140.0), T2)],
        mark: Some(2.0),
        ..Block::default()
    });
    marked_screen(
        vec![Item::Block(Block {
            name: Some("fading"),
            at: AT,
            size: (160.0, 160.0),
            opacity: Some(box_fade),
            overflow: Overflow::Hidden,
            items: vec![child],
            ..Block::default()
        })],
        B,
        U,
        0.0,
    )
    .settled(PLAIN)
}

/// Design T14b: a marked child under a fading box isn't in that box's
/// picture. It draws at its own strength, rule 9, under the box's fade put
/// back once as a scope, so at own 1 its T2 reads 0.5·200 + 0.5·120, 160,
/// and past the box's edge 120. With the box at 0 it reads 120, and nothing
/// is drawn. Swallowed by the box's picture it reads 120, gone with the
/// Screen. The scope lost reads 200, the box's fade twice 140, the own fade
/// read alone paint by paint 190, and the clip lost 160 past the edge.
#[test]
fn a_mark_under_a_fading_box_draws_under_the_boxs_fade() {
    let drawn = mark_under_a_fading_box(0.5);
    assert_reads(
        drawn.at(150, 150),
        gray(160.0),
        1.0,
        "the box at 0.5, on the marked child's T2",
    );
    assert_reads(
        drawn.at(220, 220),
        gray(120.0),
        1.0,
        "the box at 0.5, past its edge",
    );
    let drawn = mark_under_a_fading_box(0.0);
    assert_reads(
        drawn.at(150, 150),
        gray(120.0),
        1.0,
        "the box at 0, on the marked child's T2",
    );
    no_picture(&drawn, "the box at 0");
}

/// Points on Match's page, where every row sits at the page's origin: in the
/// panel's child over the red's landed flight, in it off the flight, past
/// the panel's top edge over the flight, on the panel alone over the
/// flight, and on the flight alone.
const OVER_FLIGHT: (usize, usize) = (200, 175);
const OFF_FLIGHT: (usize, usize) = (200, 225);
const PAST_PANEL: (usize, usize) = (200, 130);
const PANEL_ALONE: (usize, usize) = (50, 175);
const FLIGHT_ALONE: (usize, usize) = (200, 100);

/// `case`, a page of Match's red and panel, with Screen B at `f` and the red
/// landed on B, still in flight, over U 120. B's entrance has applied its
/// marks.
fn match_at(case: PaintCase, f: f32) -> Drawn {
    assert_eq!(
        (MARK_FLIGHT, MARK_PANEL, MARK_CHILD, MARK_BACKING),
        (0xff28_2828, T1, T2, B),
        "the design's 40, T1, T2 and B"
    );
    let mut journey = Journey::on(&css_paint_fixture(case), (400, 300), "B", "go");
    journey.set_screen("B", f);
    journey.machine.advance_and_apply(0.5);
    journey.follow(1.0, &format!("{case:?} at {f}, the red landing"));
    journey.live.draw(PLAIN)
}

fn match_marks(
    red_above: bool,
    (panel, panel_mark): (f32, f32),
    (child, child_mark): (f32, f32),
) -> PaintCase {
    PaintCase::MatchMarks {
        red_above,
        panel,
        panel_mark,
        child,
        child_mark,
    }
}

/// Design T15 above a flight, on paint order's Match with the red's flight
/// gray 40 and the panel's paints T1 and T2, at o = f = 0.5. The panel,
/// marked 1 at own 0.5, comes after the red's flight, so it draws at its own
/// strength: 0.5·200 + 0.5·40, 120, and the red stays under it. Its flight
/// alone reads 40. A plain clipping box there, the panel at own 1 and
/// unmarked, takes its picture under the scope at f: 120 too. The panel at
/// o·f reads 80, paint by paint 170, and in the picture under the flight
/// 40. The plain box paint by paint reads 170, at own strength 200, and in
/// the picture 40.
#[test]
fn a_marked_panel_above_a_flight_draws_at_its_own_strength() {
    let drawn = match_at(match_marks(false, (0.5, 1.0), (1.0, 0.0)), 0.5);
    assert_reads(
        drawn.at(OVER_FLIGHT.0, OVER_FLIGHT.1),
        gray(120.0),
        1.0,
        "the panel marked 1, over the red's flight",
    );
    assert_reads(
        drawn.at(FLIGHT_ALONE.0, FLIGHT_ALONE.1),
        gray(40.0),
        1.0,
        "the red's flight alone",
    );
    let drawn = match_at(match_marks(false, (1.0, 0.0), (1.0, 0.0)), 0.5);
    assert_reads(
        drawn.at(OVER_FLIGHT.0, OVER_FLIGHT.1),
        gray(120.0),
        1.0,
        "a plain clipping box at own 1, over the red's flight",
    );
}

/// Design T15 below a flight: the same panel, marked 1, with the red's
/// resting slot after it, so the red's flight draws above it. Off the
/// flight the panel reads rule 8's mix, 0.25·200 + 0.75·(0.5·60 + 0.5·120),
/// 117.5, and under the flight the red, 40. At its own strength off the
/// flight it reads 145, and in the Screen's picture 125.
#[test]
fn a_marked_panel_below_a_flight_draws_under_its_screens_fade() {
    let drawn = match_at(match_marks(true, (0.5, 1.0), (1.0, 0.0)), 0.5);
    assert_reads(
        drawn.at(OFF_FLIGHT.0, OFF_FLIGHT.1),
        gray(117.5),
        1.0,
        "the panel marked 1, off the red's flight",
    );
    assert_reads(
        drawn.at(OVER_FLIGHT.0, OVER_FLIGHT.1),
        gray(40.0),
        1.0,
        "the red's flight over the panel",
    );
}

/// Design T15, nested at f = 0.5: the panel, a plain clipping box of T1 at
/// own 1 above the red's flight, holds a child of T2 at own 0.5, marked 1.
/// The child isn't in the panel's picture: it draws at its own strength
/// over it, 0.5·200 + 0.5·140, 170. The panel alone reads its picture at f
/// over the flight, 140, and past its edge the child is clipped, so the
/// flight reads, 40. Swallowed by the panel's picture the child reads 130,
/// with a scope left open over it 155, with the panel lost 120, and with
/// the clip lost 120 past the edge.
#[test]
fn a_marked_child_above_a_flight_leaves_its_plain_boxs_picture() {
    let drawn = match_at(match_marks(false, (1.0, 0.0), (0.5, 1.0)), 0.5);
    assert_reads(
        drawn.at(OVER_FLIGHT.0, OVER_FLIGHT.1),
        gray(170.0),
        1.0,
        "the child marked 1, over the red's flight",
    );
    assert_reads(
        drawn.at(PANEL_ALONE.0, PANEL_ALONE.1),
        gray(140.0),
        1.0,
        "the panel alone, over the red's flight",
    );
    assert_reads(
        drawn.at(PAST_PANEL.0, PAST_PANEL.1),
        gray(40.0),
        1.0,
        "past the panel's edge",
    );
}

/// Design T15, nested at f = 0: the child marked 2 draws at its own
/// strength with the Screen gone, 0.5·200 + 0.5·40, 120, and past the
/// panel's edge the flight reads, 40. Swallowed by the panel's picture it
/// reads 40, gone with the Screen, and with the clip lost 120 past the edge.
#[test]
fn a_marked_child_above_a_flight_draws_with_its_screen_at_zero() {
    let drawn = match_at(match_marks(false, (1.0, 0.0), (0.5, 2.0)), 0.0);
    assert_reads(
        drawn.at(OVER_FLIGHT.0, OVER_FLIGHT.1),
        gray(120.0),
        1.0,
        "the child marked 2, over the red's flight",
    );
    assert_reads(
        drawn.at(PAST_PANEL.0, PAST_PANEL.1),
        gray(40.0),
        1.0,
        "past the panel's edge",
    );
}

/// Ruling U12: with Screen B at 0 and the red in flight, what draws after
/// the flight draws under a scope at 0, so it paints nothing and makes no
/// canvas, while the flight reads its own color. `mask` names the element
/// after the panel that would make one: a layer mask, or a hosted artboard
/// with an enabled bitmap cache. Main draws it under the scope at 0 and
/// makes its canvas.
fn assert_no_canvas_after_a_flight_at_zero(mask: bool) {
    let what = if mask {
        "a layer mask after the flight"
    } else {
        "a cached host after the flight"
    };
    let drawn = match_at(PaintCase::CanvasAfterFlight { mask }, 0.0);
    no_picture(&drawn, what);
    assert_reads(
        drawn.at(FLIGHT_ALONE.0, FLIGHT_ALONE.1),
        channels(MARK_FLIGHT),
        1.0,
        &format!("{what}, the red's flight"),
    );
}

/// Ruling U12 (a), a hosted artboard with an enabled bitmap cache.
#[test]
fn a_cached_host_after_a_flight_makes_no_canvas_at_zero() {
    assert_no_canvas_after_a_flight_at_zero(false);
}

/// Ruling U12 (b), a layer mask.
#[test]
fn a_layer_mask_after_a_flight_makes_no_canvas_at_zero() {
    assert_no_canvas_after_a_flight_at_zero(true);
}
