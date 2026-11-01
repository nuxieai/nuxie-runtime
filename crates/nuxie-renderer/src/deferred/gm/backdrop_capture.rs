//! Not in upstream: the frame so far, drawn into a canvas.
//!
//! A box that asks for a backdrop gets every Screen draw recorded before it,
//! replayed small. These read the capture canvas itself, never the picture the
//! box is handed, so they say nothing about a blur and stay true once one runs
//! on top. The reference is the same first draws rendered straight into a
//! canvas at that scale.
use super::ore_gm_helper::*;
use crate::deferred::cmd::deferred_replayer::snapshot_frame;
use crate::deferred::cmd::{
    deferred_replayer::{DeferredFrameSink, DeferredReplayer},
    deferred_session::{DeferredSession, ReplayCaps, SegmentTarget},
};

const WIDTH: u32 = 512;
const HEIGHT: u32 = 256;
// The box, in device pixels. Its top left corner is the capture's first texel.
const BOX_X: f32 = 16.0;
const BOX_Y: f32 = 16.0;
const BOX_SIZE: f32 = 224.0;
// A capture canvas is half the box. Only the part the scale reaches is in use.
const CANVAS_SIZE: u32 = 112;
// A read draws a canvas here, one pixel to a texel, clear of everything else.
const READ_X: usize = 272;
const READ_Y: usize = 16;
// Blurs that ask for these scales: 3 divided by the blur, in device pixels.
const QUARTER: (f32, f32) = (12.0, 1.0 / 4.0);
const SIXTEENTH: (f32, f32) = (48.0, 1.0 / 16.0);
const HALF: (f32, f32) = (6.0, 1.0 / 2.0);

const CLEAR: u32 = 0;
const WHITE: u32 = 0xffffffff;
const BLACK: u32 = 0xff000000;
const RED: u32 = 0xffff0000;
const GREEN: u32 = 0xff00ff00;
const BLUE: u32 = 0xff0000ff;
const YELLOW: u32 = 0xffffff00;
const MAGENTA: u32 = 0xffff00ff;
const WHITE_PIXEL: [u8; 4] = [255, 255, 255, 255];
const RED_PIXEL: [u8; 4] = [255, 0, 0, 255];
const GREEN_PIXEL: [u8; 4] = [0, 255, 0, 255];
const BLUE_PIXEL: [u8; 4] = [0, 0, 255, 255];
const YELLOW_PIXEL: [u8; 4] = [255, 255, 0, 255];

fn rectangle(factory: &mut dyn Factory, bounds: Aabb) -> Box<dyn RenderPath> {
    let mut raw = RawPath::new();
    raw.add_rect(bounds);
    factory.make_render_path(raw, FillRule::NonZero)
}
fn solid(factory: &mut dyn Factory, color: u32) -> Box<dyn RenderPaint> {
    let mut paint = factory.make_render_paint();
    paint.color(color);
    paint
}
fn fill(factory: &mut dyn Factory, renderer: &mut dyn Renderer, bounds: Aabb, color: u32) {
    let path = rectangle(factory, bounds);
    let paint = solid(factory, color);
    renderer.draw_path(path.as_ref(), paint.as_ref());
}
fn fill_box(factory: &mut dyn Factory, renderer: &mut dyn Renderer, color: u32) {
    let bounds = Aabb::new(BOX_X, BOX_Y, BOX_X + BOX_SIZE, BOX_Y + BOX_SIZE);
    fill(factory, renderer, bounds, color);
}

fn request(blur: f32) -> BackdropRequest {
    BackdropRequest {
        rect: Aabb::new(0.0, 0.0, BOX_SIZE, BOX_SIZE),
        world: Mat2D([1.0, 0.0, 0.0, 1.0, BOX_X, BOX_Y]),
        blur,
        saturate: 1.0,
    }
}
fn recording_session(host: &GmHost, clear: u32) -> DeferredSession {
    let mut session = DeferredSession::with_caps(ReplayCaps::from(&*host.ore.borrow()));
    session.bind_render_context(host.factory.persistent_context());
    session.set_screen(WIDTH, HEIGHT, clear);
    session
}
fn read_capture(session: &DeferredSession, screen: &mut dyn Renderer) {
    if let Some(capture) = session.backdrop_captures().first() {
        draw_canvas(screen, capture, READ_X as f32, READ_Y as f32);
    }
}
fn replay(session: &mut DeferredSession, replayer: &mut DeferredReplayer, host: &mut GmHost) {
    let frame = snapshot_frame(session);
    session.reset_frame();
    replayer.replay_frame(&frame, host);
    assert_eq!(replayer.dropped_draws(), 0);
}
// The same scale and offset a capture's head takes, written by hand.
fn scale_into_box(renderer: &mut dyn Renderer, scale: f32) {
    renderer.transform(Mat2D([
        scale,
        0.0,
        0.0,
        scale,
        -BOX_X * scale,
        -BOX_Y * scale,
    ]));
}
fn texel(pixels: &[u8], x: usize, y: usize) -> [u8; 4] {
    let at = ((READ_Y + y) * WIDTH as usize + READ_X + x) * 4;
    pixels[at..at + 4].try_into().unwrap()
}
// A device pixel inside the box, as the texel a capture at this scale holds it in.
fn texel_of(pixels: &[u8], scale: f32, x: f32, y: f32) -> [u8; 4] {
    texel(
        pixels,
        ((x - BOX_X) * scale) as usize,
        ((y - BOX_Y) * scale) as usize,
    )
}
fn assert_part_in_use_equal(what: &str, scale: f32, direct: &[u8], captured: &[u8]) {
    let part = (BOX_SIZE * scale) as usize;
    let rows = |pixels: &[u8]| {
        (0..part)
            .flat_map(|y| (0..part).map(move |x| (x, y)))
            .map(|(x, y)| texel(pixels, x, y))
            .collect::<Vec<_>>()
    };
    let (direct, captured) = (rows(direct), rows(captured));
    let first_difference = direct
        .iter()
        .zip(&captured)
        .position(|(a, b)| a != b)
        .map(|index| (index % part, index / part, direct[index], captured[index]));
    assert_eq!(first_difference, None, "{what} at scale {scale}");
}

// Shapes, a picture and one paint that changes color between two draws. The
// reference never changes a paint: it keeps one for each color, so it can't
// share a bug in how a paint's versions replay.
fn draw_behind(factory: &mut dyn Factory, renderer: &mut dyn Renderer, change_paint: bool) {
    let left = rectangle(factory, Aabb::new(16.0, 16.0, 112.0, 112.0));
    let right = rectangle(factory, Aabb::new(128.0, 16.0, 224.0, 112.0));
    let mut paint = solid(factory, RED);
    renderer.draw_path(left.as_ref(), paint.as_ref());
    if change_paint {
        paint.color(BLUE);
        renderer.draw_path(right.as_ref(), paint.as_ref());
    } else {
        let blue = solid(factory, BLUE);
        renderer.draw_path(right.as_ref(), blue.as_ref());
    }
    let mut raw = RawPath::new();
    raw.add_oval(Aabb::new(16.0, 128.0, 112.0, 224.0));
    let oval = factory.make_render_path(raw, FillRule::NonZero);
    let green = solid(factory, GREEN);
    renderer.draw_path(oval.as_ref(), green.as_ref());
    let picture = factory
        .decode_image(&fixture("css_layout/chrome/gradient_linear_angle.png"))
        .expect("tracked picture");
    renderer.save();
    renderer.transform(Mat2D([0.48, 0.0, 0.0, 0.48, 128.0, 128.0]));
    renderer.draw_image(
        Some(picture.as_ref()),
        ImageSampler::LINEAR_CLAMP,
        BlendMode::SrcOver,
        1.0,
    );
    renderer.restore();
}
fn captured_frame(blur: f32) -> (Vec<u8>, BackdropAnswer) {
    let mut host = GmHost::with_size(CLEAR, false, WIDTH, HEIGHT);
    let mut session = recording_session(&host, CLEAR);
    let mut screen = session.make_screen_renderer(0);
    draw_behind(&mut session, screen.as_mut(), true);
    let answer = screen.backdrop(&request(blur));
    // Over the whole box, so a capture that runs on past the marker can't match.
    fill_box(&mut session, screen.as_mut(), MAGENTA);
    read_capture(&session, screen.as_mut());
    replay(&mut session, &mut DeferredReplayer::default(), &mut host);
    (host.finish(), answer)
}
fn direct_frame(scale: f32) -> Vec<u8> {
    let mut host = GmHost::with_size(CLEAR, false, WIDTH, HEIGHT);
    let canvas = host.canvas(CANVAS_SIZE, CANVAS_SIZE);
    let content = host.begin_canvas_content(canvas.clone(), CLEAR).unwrap();
    {
        let mut renderer = content.borrow_mut();
        scale_into_box(renderer.as_mut(), scale);
        draw_behind(&mut host.factory, renderer.as_mut(), false);
    }
    host.end_canvas_content();
    let screen = host.begin_screen_frame(0).unwrap();
    draw_canvas(
        screen.borrow_mut().as_mut(),
        &canvas,
        READ_X as f32,
        READ_Y as f32,
    );
    host.finish()
}

#[test]
fn backdrop_capture_matches_direct() {
    for (blur, scale) in [QUARTER, SIXTEENTH] {
        let direct = direct_frame(scale);
        let (captured, answer) = captured_frame(blur);
        for (x, y, color) in [(64.0, 64.0, RED_PIXEL), (176.0, 64.0, BLUE_PIXEL)] {
            assert_eq!(texel_of(&direct, scale, x, y), color, "direct at {scale}");
            assert_eq!(
                texel_of(&captured, scale, x, y),
                color,
                "capture at {scale}"
            );
        }
        assert_part_in_use_equal("first draws", scale, &direct, &captured);
        assert!(matches!(answer, BackdropAnswer::Image { .. }));
    }
}

const BEHIND_SIZE: u32 = 32;
const BEHIND_COLORS: [u32; 2] = [GREEN, YELLOW];
const BEHIND_PIXELS: [[u8; 4]; 2] = [GREEN_PIXEL, YELLOW_PIXEL];

fn fill_behind_canvas(factory: &mut dyn Factory, renderer: &mut dyn Renderer, color: u32) {
    let size = BEHIND_SIZE as f32;
    fill(factory, renderer, Aabb::new(0.0, 0.0, size, size), color);
}
// As a paint's image and not a drawn image: a Screen range names a paint's
// canvas only through CanvasSample, so nothing else can order this canvas.
fn draw_canvas_behind(
    factory: &mut dyn Factory,
    renderer: &mut dyn Renderer,
    canvas: &RenderCanvasHandle,
) {
    let path = rectangle(
        factory,
        Aabb::new(BOX_X, BOX_Y, BOX_X + BOX_SIZE, BOX_Y + BOX_SIZE),
    );
    let mut paint = solid(factory, WHITE);
    paint.modulated_image(
        Some(canvas.borrow().render_image().as_ref()),
        ImageSampler {
            filter: ImageFilter::Nearest,
            ..Default::default()
        },
        Mat2D([BOX_SIZE, 0.0, 0.0, BOX_SIZE, BOX_X, BOX_Y]),
    );
    renderer.draw_path(path.as_ref(), paint.as_ref());
}
fn captured_frames_with_canvas_behind(blur: f32) -> [Vec<u8>; 2] {
    let mut host = GmHost::with_size(CLEAR, false, WIDTH, HEIGHT);
    let mut session = recording_session(&host, CLEAR);
    let mut replayer = DeferredReplayer::default();
    let behind =
        DeferredCanvasHost::make_content_canvas(&mut session, BEHIND_SIZE, BEHIND_SIZE).unwrap();
    BEHIND_COLORS.map(|color| {
        session.set_screen(WIDTH, HEIGHT, CLEAR);
        let mut screen = session.make_screen_renderer(0);
        draw_canvas_behind(&mut session, screen.as_mut(), &behind);
        screen.backdrop(&request(blur));
        // The canvas's content comes after the marker, so the capture is the
        // first canvas in the stream. Only its own scan can order this one
        // ahead of it; without that it reads last frame's content.
        let mut content = session.begin_canvas_content(behind.clone(), CLEAR).unwrap();
        fill_behind_canvas(&mut session, content.as_mut(), color);
        session.end_canvas_content(&behind);
        read_capture(&session, screen.as_mut());
        replay(&mut session, &mut replayer, &mut host);
        host.finish_frame()
    })
}
fn direct_frames_with_canvas_behind(scale: f32) -> [Vec<u8>; 2] {
    let mut host = GmHost::with_size(CLEAR, false, WIDTH, HEIGHT);
    let behind = host.canvas(BEHIND_SIZE, BEHIND_SIZE);
    let canvas = host.canvas(CANVAS_SIZE, CANVAS_SIZE);
    BEHIND_COLORS.map(|color| {
        let content = host.begin_canvas_content(behind.clone(), CLEAR).unwrap();
        fill_behind_canvas(&mut host.factory, content.borrow_mut().as_mut(), color);
        host.end_canvas_content();
        let content = host.begin_canvas_content(canvas.clone(), CLEAR).unwrap();
        {
            let mut renderer = content.borrow_mut();
            scale_into_box(renderer.as_mut(), scale);
            draw_canvas_behind(&mut host.factory, renderer.as_mut(), &behind);
        }
        host.end_canvas_content();
        let screen = host.begin_screen_frame(0).unwrap();
        draw_canvas(
            screen.borrow_mut().as_mut(),
            &canvas,
            READ_X as f32,
            READ_Y as f32,
        );
        host.finish_frame()
    })
}

#[test]
fn backdrop_capture_matches_direct_with_a_canvas_behind() {
    let (blur, scale) = QUARTER;
    let direct = direct_frames_with_canvas_behind(scale);
    let captured = captured_frames_with_canvas_behind(blur);
    for (frame, color) in BEHIND_PIXELS.into_iter().enumerate() {
        assert_eq!(texel_of(&direct[frame], scale, 128.0, 128.0), color);
        assert_eq!(
            texel_of(&captured[frame], scale, 128.0, 128.0),
            color,
            "frame {frame}"
        );
        assert_part_in_use_equal("canvas behind", scale, &direct[frame], &captured[frame]);
    }
}

// Reads over a black patch. A real white frame would show white through a
// capture that started clear, so here clear reads black and white reads white.
fn fill_read_patch(factory: &mut dyn Factory, renderer: &mut dyn Renderer) {
    let (x, y) = (READ_X as f32, READ_Y as f32);
    let size = CANVAS_SIZE as f32;
    fill(
        factory,
        renderer,
        Aabb::new(x, y, x + size, y + size),
        BLACK,
    );
}
fn draw_part_of_box(factory: &mut dyn Factory, renderer: &mut dyn Renderer) {
    fill(factory, renderer, Aabb::new(16.0, 16.0, 112.0, 112.0), RED);
}
fn captured_frame_cleared_white(blur: f32) -> Vec<u8> {
    let mut host = GmHost::with_size(WHITE, false, WIDTH, HEIGHT);
    let mut session = recording_session(&host, WHITE);
    let mut screen = session.make_screen_renderer(0);
    draw_part_of_box(&mut session, screen.as_mut());
    screen.backdrop(&request(blur));
    fill_read_patch(&mut session, screen.as_mut());
    read_capture(&session, screen.as_mut());
    replay(&mut session, &mut DeferredReplayer::default(), &mut host);
    host.finish()
}
fn direct_frame_cleared_white(scale: f32) -> Vec<u8> {
    let mut host = GmHost::with_size(WHITE, false, WIDTH, HEIGHT);
    let canvas = host.canvas(CANVAS_SIZE, CANVAS_SIZE);
    let content = host.begin_canvas_content(canvas.clone(), WHITE).unwrap();
    {
        let mut renderer = content.borrow_mut();
        scale_into_box(renderer.as_mut(), scale);
        draw_part_of_box(&mut host.factory, renderer.as_mut());
    }
    host.end_canvas_content();
    let screen = host.begin_screen_frame(0).unwrap();
    {
        let mut renderer = screen.borrow_mut();
        fill_read_patch(&mut host.factory, renderer.as_mut());
        draw_canvas(renderer.as_mut(), &canvas, READ_X as f32, READ_Y as f32);
    }
    host.finish()
}

#[test]
fn backdrop_capture_matches_direct_on_a_frame_cleared_white() {
    let (blur, scale) = QUARTER;
    let direct = direct_frame_cleared_white(scale);
    let captured = captured_frame_cleared_white(blur);
    assert_eq!(texel_of(&direct, scale, 200.0, 200.0), WHITE_PIXEL);
    assert_eq!(texel_of(&captured, scale, 200.0, 200.0), WHITE_PIXEL);
    assert_eq!(texel_of(&captured, scale, 64.0, 64.0), RED_PIXEL);
    assert_part_in_use_equal("frame cleared white", scale, &direct, &captured);
}

#[test]
fn backdrop_reads_only_what_is_behind() {
    let (blur, scale) = HALF;
    let mut host = GmHost::with_size(CLEAR, false, WIDTH, HEIGHT);
    let mut session = recording_session(&host, CLEAR);
    let mut screen = session.make_screen_renderer(0);
    fill_box(&mut session, screen.as_mut(), WHITE);
    screen.save();
    let clip = rectangle(&mut session, Aabb::new(16.0, 16.0, 128.0, 240.0));
    screen.clip_path(clip.as_ref());
    let first = Aabb::new(32.0, 32.0, 80.0, 80.0);
    fill(&mut session, screen.as_mut(), first, RED);
    // The clip's edge at 128 runs down the middle of this one.
    let second = Aabb::new(96.0, 32.0, 160.0, 80.0);
    fill(&mut session, screen.as_mut(), second, GREEN);
    let answer = screen.backdrop(&request(blur));
    // Inside the same clip, so a capture of the whole frame would hold it.
    let third = Aabb::new(32.0, 128.0, 112.0, 208.0);
    fill(&mut session, screen.as_mut(), third, BLUE);
    screen.restore();
    read_capture(&session, screen.as_mut());
    replay(&mut session, &mut DeferredReplayer::default(), &mut host);
    let pixels = host.finish();
    assert_eq!(texel_of(&pixels, scale, 56.0, 56.0), RED_PIXEL, "first box");
    assert_eq!(
        texel_of(&pixels, scale, 112.0, 56.0),
        GREEN_PIXEL,
        "second box, kept half"
    );
    assert_eq!(
        texel_of(&pixels, scale, 144.0, 56.0),
        WHITE_PIXEL,
        "second box, cut half"
    );
    assert_eq!(
        texel_of(&pixels, scale, 72.0, 168.0),
        WHITE_PIXEL,
        "third box"
    );
    assert!(matches!(answer, BackdropAnswer::Image { .. }));
}

// Everything below keeps to the fit's units, doubled on the way to the screen.
const FIT: Mat2D = Mat2D([2.0, 0.0, 0.0, 2.0, 0.0, 0.0]);
const POSED_BOX: Aabb = Aabb {
    min_x: 0.0,
    min_y: 0.0,
    max_x: 80.0,
    max_y: 48.0,
};

// The box's center sits on the frame's center, where red meets blue.
fn turned_box() -> Mat2D {
    let (sin, cos) = 20f32.to_radians().sin_cos();
    let (tx, ty) = (
        128.0 - 40.0 * cos + 24.0 * sin,
        64.0 - 40.0 * sin - 24.0 * cos,
    );
    Mat2D([cos, sin, -sin, cos, tx, ty])
}
fn mirrored_box() -> Mat2D {
    Mat2D([-1.0, 0.0, 0.0, 1.0, 168.0, 40.0])
}
fn pixel(pixels: &[u8], x: usize, y: usize) -> [u8; 4] {
    let at = (y * WIDTH as usize + x) * 4;
    pixels[at..at + 4].try_into().unwrap()
}
// Black goes over everything after the marker, so only the picture can show
// the two colors, and only if its mapping puts them back where they were drawn.
fn composited_frame(world: Mat2D) -> Vec<u8> {
    let mut host = GmHost::with_size(CLEAR, false, WIDTH, HEIGHT);
    let mut session = recording_session(&host, CLEAR);
    let mut screen = session.make_screen_renderer(0);
    screen.transform(FIT);
    let left = Aabb::new(0.0, 0.0, 128.0, 128.0);
    let right = Aabb::new(128.0, 0.0, 256.0, 128.0);
    fill(&mut session, screen.as_mut(), left, RED);
    fill(&mut session, screen.as_mut(), right, BLUE);
    let answer = screen.backdrop(&BackdropRequest {
        rect: POSED_BOX,
        world,
        blur: 3.0,
        saturate: 1.0,
    });
    let BackdropAnswer::Image { image, mapping } = answer else {
        panic!("a box on the screen gets a picture");
    };
    let whole = Aabb::new(0.0, 0.0, 256.0, 128.0);
    fill(&mut session, screen.as_mut(), whole, BLACK);
    let path = rectangle(&mut session, POSED_BOX);
    let mut paint = solid(&mut session, WHITE);
    paint.modulated_image(Some(image.as_ref()), ImageSampler::LINEAR_CLAMP, mapping);
    screen.save();
    screen.transform(world);
    screen.draw_path(path.as_ref(), paint.as_ref());
    screen.restore();
    replay(&mut session, &mut DeferredReplayer::default(), &mut host);
    host.finish()
}

#[test]
fn backdrop_picture_lands_where_it_was_drawn() {
    for (what, world) in [("turned", turned_box()), ("mirrored", mirrored_box())] {
        let pixels = composited_frame(world);
        assert_eq!(pixel(&pixels, 216, 128), RED_PIXEL, "{what} box, left");
        assert_eq!(pixel(&pixels, 296, 128), BLUE_PIXEL, "{what} box, right");
        let outside = pixel(&pixels, 100, 128);
        assert_eq!(outside, [0, 0, 0, 255], "outside the {what} box");
    }
}

fn answers_no(renderer: &mut dyn Renderer) -> bool {
    matches!(renderer.backdrop(&request(12.0)), BackdropAnswer::No)
}

#[test]
fn backdrop_answers_no_where_it_cannot_capture() {
    let host = GmHost::with_size(CLEAR, false, WIDTH, HEIGHT);
    let mut session = DeferredSession::with_caps(ReplayCaps::from(&*host.ore.borrow()));
    session.bind_render_context(host.factory.persistent_context());
    let mut screen = session.make_screen_renderer(0);
    assert!(answers_no(screen.as_mut()), "a session told nothing");
    session.set_screen(0, HEIGHT, CLEAR);
    assert!(answers_no(screen.as_mut()), "a session told a zero size");
    session.set_screen(WIDTH, HEIGHT, CLEAR);
    session.reset_frame();
    assert!(answers_no(screen.as_mut()), "a session told last frame");
    session.set_screen(WIDTH, HEIGHT, CLEAR);
    session.bind_render_context(None);
    assert!(answers_no(screen.as_mut()), "a session with no device");
    session.bind_render_context(host.factory.persistent_context());
    let canvas = DeferredCanvasHost::make_content_canvas(&mut session, 16, 16).unwrap();
    let mut content = session.begin_canvas_content(canvas.clone(), CLEAR).unwrap();
    assert!(answers_no(content.as_mut()), "inside a canvas");
    session.end_canvas_content(&canvas);
    assert!(session.backdrop_captures().is_empty());
}

#[test]
fn backdrop_answers_nothing_for_a_box_with_no_area() {
    let host = GmHost::with_size(CLEAR, false, WIDTH, HEIGHT);
    let session = recording_session(&host, CLEAR);
    let mut screen = session.make_screen_renderer(0);
    let whole = request(12.0).rect;
    let empty = Aabb::new(0.0, 0.0, 0.0, BOX_SIZE);
    let flat = Mat2D([0.0, 0.0, 0.0, 1.0, BOX_X, BOX_Y]);
    let off_screen = Mat2D([1.0, 0.0, 0.0, 1.0, WIDTH as f32, 0.0]);
    let not_a_number = Mat2D([f32::NAN, 0.0, 0.0, 1.0, 0.0, 0.0]);
    for (what, rect, world) in [
        ("scale-x 0", whole, flat),
        ("off the screen", whole, off_screen),
        ("empty", empty, Mat2D::IDENTITY),
        ("not a number", whole, not_a_number),
    ] {
        let answer = screen.backdrop(&BackdropRequest {
            rect,
            world,
            blur: 12.0,
            saturate: 1.0,
        });
        assert!(matches!(answer, BackdropAnswer::Nothing), "{what}");
    }
    assert!(session.backdrop_captures().is_empty());
}

#[test]
fn backdrop_capture_takes_only_its_own_targets_ranges() {
    let host = GmHost::with_size(CLEAR, false, WIDTH, HEIGHT);
    let mut session = recording_session(&host, CLEAR);
    let mut other = session.make_screen_renderer(1);
    let mut screen = session.make_screen_renderer(0);
    fill_box(&mut session, screen.as_mut(), RED);
    fill_box(&mut session, other.as_mut(), GREEN);
    fill_box(&mut session, screen.as_mut(), BLUE);
    screen.backdrop(&request(12.0));
    fill_box(&mut session, screen.as_mut(), MAGENTA);
    session.close_open_range();
    let segments = session.scheduler_segments();
    let own = segments
        .iter()
        .filter(|segment| segment.target == SegmentTarget::Screen && segment.target_id == 0)
        .map(|segment| (segment.begin, segment.end))
        .collect::<Vec<_>>();
    assert_eq!(own.len(), 3, "red, blue, and magenta after the marker");
    let head = segments
        .iter()
        .position(|segment| segment.target == SegmentTarget::Canvas)
        .unwrap();
    assert!(segments[head].starts_frame);
    let captured = segments[head + 1..]
        .iter()
        .filter(|segment| segment.screen_range)
        .inspect(|segment| assert_eq!(segment.target_id, segments[head].target_id))
        .map(|segment| (segment.begin, segment.end))
        .collect::<Vec<_>>();
    assert_eq!(captured, own[..2]);
}
