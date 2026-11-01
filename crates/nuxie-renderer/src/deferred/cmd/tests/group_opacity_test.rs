//! What a CSS element's picture costs, on the null device: how many content
//! canvases a recorded frame holds, how large they are and when they go, for a
//! hosted element, for a box that clips both axes and for a Screen.
//! `gm/group_opacity_pixel.rs` reads the pixels.
//!
//! "Design" names the tests of `design-2`, "night" the earlier design's. The
//! two number their tests apart, and both sets apply to this picture.
use super::super::{
    canvas_schedule::{live_segments, schedule_canvases},
    command_stream::CommandReader,
    deferred_replayer::{snapshot_frame, DeferredFrame},
    deferred_session::{DeferredSession, SegmentTarget},
    render_commands::{payload_size_of, DrawImagePod, OpacityPod, RenderCmd},
    render_handle::{CANVAS_HANDLE_FLAG, CANVAS_HANDLE_MASK},
};
use super::render_context_null::*;
use super::*;
use nuxie_runtime::group_opacity_fixture::{
    bind_page, screen_row, set_row_opacity, Block, Host, Item, Overflow, Page, Scene, Unit,
};
use nuxie_runtime::live_move::{records::LiveSlot, slot_pose, Flight, LiveMoves};
use nuxie_runtime::mechanical_port::source::{
    generated::{
        core_registry::CoreRegistry, world_transform_component_base::WorldTransformComponentBase,
    },
    nested_artboard::NestedArtboard,
    offscreen_raster::K_SHRINK_DRAWS,
};
use nuxie_runtime::{CoreHandle, RuntimeArtboardInstanceHandle, RuntimeFileHandle};

const IDENTITY: Mat2D = Mat2D([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

/// A file recorded into a `DeferredSession` bound to a null context, whose
/// texture limit is `limit`.
struct Stage {
    session: PersistentFactory<DeferredSession>,
    _context: ObservingFactory,
    _file: RuntimeFileHandle,
    root: RuntimeArtboardInstanceHandle,
}
impl Stage {
    fn new(bytes: &[u8], limit: u32) -> Self {
        let (context, _, _) = observing_factory(1, 1);
        context
            .borrow()
            .with_backend_mut_for_test(|backend| backend.set_max_texture_size(limit));
        let mut session = PersistentFactory::new(DeferredSession::with_caps(Default::default()));
        session
            .borrow_mut()
            .bind_render_context(context.persistent_context());
        let file = nuxie_runtime::File::import(
            bytes,
            nuxie_runtime::RuntimeFactoryHandle::from_factory(&mut session).unwrap(),
            None,
            None,
            None,
        )
        .expect("the scene imports");
        let root = file
            .with_file(nuxie_runtime::File::artboard_default)
            .expect("root");
        Self {
            session,
            _context: context,
            _file: file,
            root,
        }
    }
    /// A page's stage, its model bound so its rows mount.
    fn page(bytes: &[u8], limit: u32) -> Self {
        let stage = Self::new(bytes, limit);
        bind_page(&stage._file, &stage.root);
        stage
    }
    /// Writes `value` to the own opacity of the row of the Screen `name`, as a
    /// keyed `ScreenOpacity` does.
    fn set_screen(&self, name: &str, value: f32) {
        set_row_opacity(&screen_row(&self.root, name), value);
    }
    fn set_opacity(&self, host: &str, value: f32) {
        let host = self
            .root
            .with_artboard(|root| root.find_handle::<NestedArtboard>(host))
            .expect("host");
        CoreRegistry::set_double_handle(
            &host,
            i32::from(WorldTransformComponentBase::OPACITY_PROPERTY_KEY),
            value,
        );
    }
    fn frame(&mut self, transform: Mat2D) -> DeferredFrame {
        self.frame_under(transform, None)
    }
    /// A frame drawn under an opacity scope, when `scope` gives one.
    fn frame_under(&mut self, transform: Mat2D, scope: Option<f32>) -> DeferredFrame {
        for _ in 0..3 {
            self.root.advance_default(1.0 / 60.0);
        }
        record_under(&mut self.session, &self.root, transform, scope)
    }
}

fn record(
    session: &mut PersistentFactory<DeferredSession>,
    root: &RuntimeArtboardInstanceHandle,
    transform: Mat2D,
) -> DeferredFrame {
    record_under(session, root, transform, None)
}

fn record_under(
    session: &mut PersistentFactory<DeferredSession>,
    root: &RuntimeArtboardInstanceHandle,
    transform: Mat2D,
    scope: Option<f32>,
) -> DeferredFrame {
    let screen = session.borrow().screen_renderer(0);
    {
        let mut screen = screen.borrow_mut();
        screen.save();
        screen.transform(transform);
        if let Some(opacity) = scope {
            screen.modulate_opacity(opacity);
        }
        root.draw(screen.as_mut());
        screen.restore();
    }
    let frame = snapshot_frame(&mut session.borrow_mut());
    session.borrow_mut().reset_frame();
    frame
}

fn canvas_sizes(frame: &DeferredFrame) -> Vec<(u32, u32)> {
    frame
        .content_canvases
        .values()
        .map(|canvas| (canvas.borrow().width(), canvas.borrow().height()))
        .collect()
}

/// Reads the frame's screen segments, command by command.
fn read_screen(frame: &DeferredFrame, mut read: impl FnMut(RenderCmd, &mut CommandReader<'_>)) {
    for segment in &frame.segments {
        if segment.target != SegmentTarget::Screen {
            continue;
        }
        let commands = &frame.commands[segment.begin as usize..segment.end as usize];
        let mut reader = CommandReader::new(commands, &frame.blobs);
        while let Some(command) = reader.next_u8().and_then(RenderCmd::from_byte) {
            read(command, &mut reader);
        }
    }
}

/// The opacity of each `DrawImage` in a screen segment that names one of the
/// frame's canvases.
fn composites(frame: &DeferredFrame) -> Vec<f32> {
    let mut opacities = Vec::new();
    read_screen(frame, |command, reader| {
        if matches!(command, RenderCmd::DrawImage) {
            let draw: DrawImagePod = reader.read();
            let image = draw.image;
            if image & CANVAS_HANDLE_FLAG != 0
                && frame
                    .content_canvases
                    .contains_key(&(image & CANVAS_HANDLE_MASK))
            {
                opacities.push(draw.opacity);
            }
        } else {
            reader.skip(payload_size_of(command));
        }
    });
    opacities
}

/// Every opacity scope the frame opens on the screen.
fn scopes(frame: &DeferredFrame) -> Vec<f32> {
    let mut opacities = Vec::new();
    read_screen(frame, |command, reader| {
        if matches!(command, RenderCmd::ModulateOpacity) {
            let scope: OpacityPod = reader.read();
            opacities.push(scope.opacity);
        } else {
            reader.skip(payload_size_of(command));
        }
    });
    opacities
}

fn block(at: (f32, f32), size: (f32, f32), color: u32) -> Item {
    Item::Block(Block {
        at,
        size,
        color: Some(color),
        ..Block::default()
    })
}

/// A unit in the compiled shape with two opaque boxes over a background.
fn unit(width: f32, height: f32) -> Unit {
    Unit {
        background: Some(0xffff_ffff),
        items: vec![
            block((10.0, 10.0), (60.0, 60.0), 0xff00_00ff),
            block((30.0, 30.0), (60.0, 60.0), 0xffff_0000),
        ],
        ..Unit::compiled(width, height)
    }
}

const BIG_LIMIT: u32 = 8192;

fn hosted(width: f32, height: f32, opacity: f32) -> Vec<u8> {
    Scene::hosted((2000.0, 2000.0), unit(width, height), (0.0, 0.0), opacity).bytes()
}

/// Design T16, a guard: at an effective fade of 1 there is no picture.
#[test]
fn at_own_fade_one_nothing_takes_a_picture() {
    let mut stage = Stage::new(&hosted(160.0, 160.0, 1.0), BIG_LIMIT);
    let frame = stage.frame(IDENTITY);
    assert_eq!(canvas_sizes(&frame), []);
    assert!(composites(&frame).is_empty());
}

/// Night T7: a canvas is made only while something fades, one for each copy
/// that fades, and none is held after landing.
#[test]
fn content_canvases_follow_the_fade_and_go_at_landing() {
    let scene = Scene {
        root: Unit {
            items: vec![
                Item::Host(Host::plain("a", 0, (0.0, 0.0), 1.0)),
                Item::Host(Host::plain("b", 0, (0.0, 0.0), 1.0)),
            ],
            ..Unit::css(300.0, 300.0)
        },
        units: vec![unit(160.0, 160.0)],
        photo: None,
    };
    let mut stage = Stage::new(&scene.bytes(), BIG_LIMIT);
    let counts = |stage: &mut Stage, a: f32, b: f32| {
        stage.set_opacity("a", a);
        stage.set_opacity("b", b);
        let frame = stage.frame(IDENTITY);
        let held: Vec<_> = frame.content_canvases.values().map(Rc::downgrade).collect();
        (frame.content_canvases.len(), composites(&frame).len(), held)
    };
    assert_eq!(counts(&mut stage, 1.0, 1.0).0, 0, "both at 1");
    assert_eq!(counts(&mut stage, 0.0, 0.0).0, 0, "both at 0");
    assert_eq!(counts(&mut stage, 0.5, 1.0).0, 1, "one at 0.5");
    let (canvases, composited, held) = counts(&mut stage, 0.5, 0.5);
    assert_eq!((canvases, composited), (2, 2), "both at 0.5");
    assert_eq!(counts(&mut stage, 1.0, 1.0).0, 0, "landed");
    assert!(
        held.iter().all(|canvas| canvas.upgrade().is_none()),
        "a canvas outlived its landing"
    );
}

/// Design T3, the unit's line: 1206 by 2622 device pixels and 2 guard pixels a
/// side round up by 64, past the old cap of 2048.
#[test]
fn a_full_size_unit_takes_one_canvas_of_its_whole_size() {
    let mut stage = Stage::new(&hosted(402.0, 874.0, 0.5), BIG_LIMIT);
    let frame = stage.frame(Mat2D([3.0, 0.0, 0.0, 3.0, 0.0, 0.0]));
    assert_eq!(canvas_sizes(&frame), [(1216, 2688)]);
    assert_eq!(composites(&frame), [0.5]);
}

/// Design 1.5, item 2: rounding stops at the limit, so a box that fits keeps
/// its picture.
#[test]
fn rounding_stops_at_the_limit() {
    let mut stage = Stage::new(&hosted(402.0, 874.0, 0.5), 2660);
    let frame = stage.frame(Mat2D([3.0, 0.0, 0.0, 3.0, 0.0, 0.0]));
    assert_eq!(canvas_sizes(&frame), [(1216, 2660)]);
}

/// Design 1.5, item 7, a guard: a box past the real limit makes no canvas.
#[test]
fn a_box_past_the_limit_makes_no_canvas() {
    let mut stage = Stage::new(&hosted(402.0, 874.0, 0.5), 2600);
    let frame = stage.frame(Mat2D([3.0, 0.0, 0.0, 3.0, 0.0, 0.0]));
    assert_eq!(canvas_sizes(&frame), []);
}

/// Rule 5: the same frame puts the fade back through a scope, so it is not
/// lost now that the host's factor stays out of the children's number.
#[test]
fn a_box_past_the_limit_fades_through_a_scope() {
    let mut stage = Stage::new(&hosted(402.0, 874.0, 0.5), 2600);
    let frame = stage.frame(Mat2D([3.0, 0.0, 0.0, 3.0, 0.0, 0.0]));
    assert_eq!(scopes(&frame), [0.5]);
}

/// A frame of a 300 by 100 unit at 0.5, stretched to `width` device pixels.
fn stretched(stage: &mut Stage, width: f32) -> (RenderCanvasHandle, (u32, u32)) {
    let frame = stage.frame(Mat2D([width / 300.0, 0.0, 0.0, 1.0, 0.0, 0.0]));
    let canvases: Vec<_> = frame.content_canvases.values().cloned().collect();
    assert_eq!(canvases.len(), 1, "at {width}");
    let size = (canvases[0].borrow().width(), canvases[0].borrow().height());
    (canvases[0].clone(), size)
}

/// Night T7, restated for design 1.5, item 2. The first allocation is exact: a
/// need of 304 by 104 rounds up by 64. A need past it, 321, takes half again on
/// each axis before it rounds, 481.5 by 156, and that is then held.
#[test]
fn a_growing_unit_allocates_once_for_each_step_it_crosses() {
    let mut stage = Stage::new(&hosted(300.0, 100.0, 0.5), BIG_LIMIT);
    let (first, size) = stretched(&mut stage, 300.0);
    assert_eq!(size, (320, 128), "exact, by 64");
    for width in [310.0, 315.0] {
        let (held, size) = stretched(&mut stage, width);
        assert!(Rc::ptr_eq(&first, &held), "at {width}");
        assert_eq!(size, (320, 128), "at {width}");
    }
    let (grown, size) = stretched(&mut stage, 317.0);
    assert!(!Rc::ptr_eq(&first, &grown), "past the step");
    assert_eq!(size, (512, 192), "half again, by 64");
    let (held, size) = stretched(&mut stage, 340.0);
    assert!(Rc::ptr_eq(&grown, &held), "at 340");
    assert_eq!(size, (512, 192), "at 340");
}

/// Design 1.5, item 2: a canvas held past its need shrinks after
/// `K_SHRINK_DRAWS` smaller draws, and not before. A need of 344 by 104 takes
/// half again, 516 by 156, before it rounds.
#[test]
fn a_held_canvas_shrinks_after_its_smaller_draws() {
    let mut stage = Stage::new(&hosted(300.0, 100.0, 0.5), BIG_LIMIT);
    stretched(&mut stage, 300.0);
    assert_eq!(stretched(&mut stage, 340.0).1, (576, 192), "grown");
    for draw in 1..K_SHRINK_DRAWS {
        assert_eq!(stretched(&mut stage, 100.0).1, (576, 192), "draw {draw}");
    }
    let (width, height) = stretched(&mut stage, 100.0).1;
    assert!(
        width < 576 && height < 192,
        "still {width} by {height} after {K_SHRINK_DRAWS} smaller draws"
    );
}

fn unit_key(flight: &Flight) -> u32 {
    flight
        .controls()
        .with_downcast::<LiveSlot, _>(|record| record.values.unit_key)
        .expect("flight record")
}

fn hosted_artboard(host: &CoreHandle) -> RuntimeArtboardInstanceHandle {
    host.with(|object| {
        object
            .as_nested_artboard()
            .expect("nested artboard")
            .artboard_instance_default()
    })
    .flatten()
    .expect("hosted artboard")
}

/// Night T9: a real flight, whose units clip by their records and not by their
/// frames. At progress 0.5 the versions draw 120 by 90, the mix of 80 by 60 and
/// 160 by 120, so with 2 guard pixels a side the top's canvas rounds to 128 by
/// 128. The held version, at fade 1, takes none.
#[test]
fn a_flights_top_version_takes_one_picture_at_its_pose_opacity() {
    const PROGRESS: i32 = 62003;
    const FADE: i32 = 62014;
    /// Unit b, whose slot is above unit a's on Screen B.
    const TOP: u32 = 2;
    let (context, _, _) = observing_factory(1, 1);
    context
        .borrow()
        .with_backend_mut_for_test(|backend| backend.set_max_texture_size(BIG_LIMIT));
    let mut session = PersistentFactory::new(DeferredSession::with_caps(Default::default()));
    session
        .borrow_mut()
        .bind_render_context(context.persistent_context());
    let file = nuxie_runtime::File::import(
        &nuxie_runtime::live_move_fixture::css_box_root_versions_fixture(),
        nuxie_runtime::RuntimeFactoryHandle::from_factory(&mut session).unwrap(),
        None,
        None,
        None,
    )
    .expect("the page imports");
    let root = file.with_file(|f| f.artboard_named("Page")).unwrap();
    let model = file
        .with_file(|f| f.create_default_view_model_instance_for_artboard(root.core_handle()))
        .unwrap();
    let machine = root.default_state_machine_handle().unwrap();
    machine.with_instance_mut(|m| m.bind_view_model_instance(model));
    machine.advance_and_apply(0.0);
    let mut moves = LiveMoves::default();
    moves.prepare(&root).unwrap();
    moves.step_begin(&root, "B", Some("t1")).unwrap();
    let flights = moves.flights(&root);
    assert_eq!(flights.len(), 2);
    for flight in &flights {
        let fade = if unit_key(flight) == TOP { 0.5 } else { 1.0 };
        assert!(CoreRegistry::set_double_handle(
            &flight.controls(),
            PROGRESS,
            0.5
        ));
        assert!(CoreRegistry::set_double_handle(
            &flight.controls(),
            FADE,
            fade
        ));
    }
    machine.advance_and_apply(0.0);
    moves.after_advance(&root).unwrap();

    let top = flights
        .iter()
        .find(|flight| unit_key(flight) == TOP)
        .expect("the top version");
    let (css_clips, frame_clips) =
        hosted_artboard(&top.flight).with_artboard(|unit| (unit.css_clips(), unit.clip()));
    assert!(css_clips, "the unit clips by its record");
    assert!(!frame_clips, "and not by its frame");

    let frame = record(&mut session, &root, IDENTITY);
    assert_eq!(canvas_sizes(&frame), [(128, 128)], "one, for the top alone");
    let pose = slot_pose(&top.flight).unwrap();
    assert!((pose.opacity - 0.5).abs() < 0.002, "{}", pose.opacity);
    assert_eq!(composites(&frame), [pose.opacity]);
}

/// A box that clips both axes at `opacity`, white with a blue box on top.
fn clipping_box(opacity: f32) -> Item {
    Item::Block(Block {
        at: (20.0, 20.0),
        size: (160.0, 160.0),
        color: Some(0xffff_ffff),
        opacity: Some(opacity),
        overflow: Overflow::Hidden,
        items: vec![block((20.0, 20.0), (90.0, 90.0), 0xff00_00ff)],
        ..Block::default()
    })
}

/// A CSS root holding `item`.
fn root_holding(item: Item) -> Vec<u8> {
    Scene {
        root: Unit {
            items: vec![item],
            ..Unit::css(300.0, 300.0)
        },
        units: Vec::new(),
        photo: None,
    }
    .bytes()
}

/// Design T8, its canvas: a box that clips both axes at 0.5 takes one canvas,
/// composited once at its fade.
#[test]
fn a_clipping_box_at_half_takes_one_canvas() {
    let mut stage = Stage::new(&root_holding(clipping_box(0.5)), BIG_LIMIT);
    let frame = stage.frame(IDENTITY);
    assert_eq!(canvas_sizes(&frame).len(), 1, "{:?}", canvas_sizes(&frame));
    assert_eq!(composites(&frame), [0.5]);
}

/// Design T16, a guard: at an effective fade of 1 a box takes no picture, at
/// own 1 with no scope, and at own 1 inside a hosted element's picture at
/// 0.5, whose canvas's renderer reads 1. A build that read the host's fade as
/// the box's scope would make a second canvas there.
#[test]
fn a_clipping_box_at_effective_one_takes_no_picture() {
    let mut stage = Stage::new(&root_holding(clipping_box(1.0)), BIG_LIMIT);
    let frame = stage.frame(IDENTITY);
    assert_eq!(canvas_sizes(&frame), [], "own 1, no scope");
    assert!(composites(&frame).is_empty(), "own 1, no scope");

    let unit = Unit {
        items: vec![clipping_box(1.0)],
        ..Unit::compiled(200.0, 200.0)
    };
    let scene = Scene::hosted((300.0, 300.0), unit, (0.0, 0.0), 0.5);
    let mut stage = Stage::new(&scene.bytes(), BIG_LIMIT);
    let frame = stage.frame(IDENTITY);
    assert_eq!(
        canvas_sizes(&frame).len(),
        1,
        "the hosted element's, and none for the box: {:?}",
        canvas_sizes(&frame)
    );
    assert_eq!(composites(&frame), [0.5]);
}

/// Design T16 and rule 13, a guard: at an effective fade of 0 nothing is drawn
/// and no canvas is made, for a box's own 0 and for a box at own 1 under a
/// scope at 0.
#[test]
fn a_clipping_box_at_effective_zero_makes_no_canvas() {
    let mut stage = Stage::new(&root_holding(clipping_box(0.0)), BIG_LIMIT);
    let frame = stage.frame(IDENTITY);
    assert_eq!(canvas_sizes(&frame), [], "own 0");
    assert!(composites(&frame).is_empty(), "own 0");

    let mut stage = Stage::new(&root_holding(clipping_box(1.0)), BIG_LIMIT);
    let frame = stage.frame_under(IDENTITY, Some(0.0));
    assert_eq!(canvas_sizes(&frame), [], "own 1 under a scope at 0");
    assert!(composites(&frame).is_empty(), "own 1 under a scope at 0");
}

/// A Screen's backing, which the null device doesn't read.
const BACKING: u32 = 0xff3c_3c3c;

/// A page of `size` whose root clips, holding one Screen of the same size: a
/// CSS artboard with an empty record and no frame clip, as the compiler writes
/// a Screen, painting a white backing under `unit`'s two boxes.
fn full_screen(width: f32, height: f32) -> Vec<u8> {
    Page::one((width, height), unit(width, height)).bytes()
}

/// A 300 by 300 page holding one Screen that paints `BACKING` under `item`.
fn screen_holding(item: Item) -> Vec<u8> {
    Page::one(
        (300.0, 300.0),
        Unit {
            background: Some(BACKING),
            items: vec![item],
            ..Unit::css(300.0, 300.0)
        },
    )
    .bytes()
}

/// Design T3, the Screen's line: a 402 by 874 Screen at scale 3 is 1206 by
/// 2622 device pixels. Its box is the page's frame, which takes no guard
/// pixels, so it rounds up by 64 to 1216 by 2624, past the old cap of 2048.
#[test]
fn a_full_screen_takes_one_canvas_of_its_whole_size() {
    let mut stage = Stage::page(&full_screen(402.0, 874.0), BIG_LIMIT);
    stage.set_screen("Screen", 0.5);
    let frame = stage.frame(Mat2D([3.0, 0.0, 0.0, 3.0, 0.0, 0.0]));
    assert_eq!(canvas_sizes(&frame), [(1216, 2624)]);
    assert_eq!(composites(&frame), [0.5]);
}

/// Design T3: a Screen of 2048 device pixels on a limit of 2048 stays 2048.
/// Its box takes no guard pixels, so it fits, and no scope falls back for it.
#[test]
fn a_screen_at_the_limit_keeps_its_picture() {
    let mut stage = Stage::page(&full_screen(2048.0, 512.0), 2048);
    stage.set_screen("Screen", 0.5);
    let frame = stage.frame(IDENTITY);
    assert_eq!(canvas_sizes(&frame), [(2048, 512)]);
    assert_eq!(composites(&frame), [0.5]);
    assert!(scopes(&frame).is_empty(), "a scope: {:?}", scopes(&frame));
}

/// Design T13: a box that clips both axes at 0.5, in a Screen at 0.5, is a
/// picture inside the Screen's picture. Its canvas is drawn into the Screen's,
/// so `canvas_schedule` replays it first: its 160 points and 2 guard pixels a
/// side round up by 64 to 192, and the Screen's box, the page's 300 points, to
/// 320. Only the Screen's composite reaches the screen.
#[test]
fn a_fading_box_in_a_fading_screen_replays_before_the_screen() {
    let mut stage = Stage::page(&screen_holding(clipping_box(0.5)), BIG_LIMIT);
    stage.set_screen("Screen", 0.5);
    let frame = stage.frame(IDENTITY);
    assert_eq!(
        frame.content_canvases.len(),
        2,
        "{:?}",
        canvas_sizes(&frame)
    );
    let schedule = schedule_canvases(&frame.commands, &live_segments(&frame.segments));
    assert!(!schedule.had_cycle, "a cycle");
    assert!(!schedule.multi_write_fallback, "a multi-write fallback");
    let order: Vec<_> = schedule
        .order
        .iter()
        .map(|id| {
            let canvas = frame.content_canvases[&(*id as u32)].borrow();
            (canvas.width(), canvas.height())
        })
        .collect();
    assert_eq!(order, [(192, 192), (320, 320)]);
    assert_eq!(composites(&frame), [0.5]);
}

/// Design T16: a box at own 1 inside a Screen's picture at 0.5 draws at an
/// effective fade of 1, since the Screen's canvas renderer reads 1, so it takes
/// no picture and the Screen's canvas is the only one. A build that read the
/// Screen's fade as the box's scope would make a second canvas there.
#[test]
fn a_box_at_own_one_in_a_fading_screen_takes_no_picture() {
    let mut stage = Stage::page(&screen_holding(clipping_box(1.0)), BIG_LIMIT);
    stage.set_screen("Screen", 0.5);
    let frame = stage.frame(IDENTITY);
    assert_eq!(canvas_sizes(&frame), [(320, 320)], "the Screen's alone");
    assert_eq!(composites(&frame), [0.5]);
}
