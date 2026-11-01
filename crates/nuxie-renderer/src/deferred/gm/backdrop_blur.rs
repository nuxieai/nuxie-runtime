//! Not in upstream: what a blurred backdrop reads on the Screen.
//!
//! Each test draws something, asks for the backdrop, fills the box with the
//! picture as a box does, and reads device pixels. The answers are closed forms
//! and CSS's saturate matrix worked by hand, so none can share a bug with the
//! blur. Every read of a test runs, and a failure lists each one that missed.
use super::backdrop_capture::{fill, rectangle, replay, solid};
use super::ore_gm_helper::*;
use crate::deferred::cmd::deferred_replayer::DeferredReplayer;
use crate::deferred::cmd::deferred_session::{DeferredSession, ReplayCaps};

const CLEAR: u32 = 0;
const WHITE: u32 = 0xffffffff;
const BLACK: u32 = 0xff000000;
// rgb(200, 80, 40) and rgb(40, 80, 200).
const WARM: u32 = 0xffc85028;
const COOL: u32 = 0xff2850c8;
const WARM_AT_HALF_ALPHA: u32 = 0x80c85028;

// How far a read may sit from its answer, in levels of 255. Past a blur of 24 a
// bar is too faint to tell from nothing at this tolerance, so edges carry those.
const BAR_LEVELS: f64 = 2.0;
const EDGE_LEVELS: f64 = 8.0;
const AVERAGE_LEVELS: f64 = 3.0;
const FLAT_LEVELS: f64 = 2.0;
const SATURATED_EDGE_LEVELS: f64 = 3.0;
const PLAIN_PASS_LEVELS: f64 = 1.0;

struct Pixels {
    width: usize,
    bytes: Vec<u8>,
}
impl Pixels {
    fn at(&self, x: usize, y: usize) -> [f64; 4] {
        let at = (y * self.width + x) * 4;
        std::array::from_fn(|channel| f64::from(self.bytes[at + channel]))
    }
    // A line between two pixels reads as the pair averaged. This is the line on
    // pixel x's left.
    fn line_before(&self, x: usize, y: usize) -> [f64; 3] {
        let (left, right) = (self.at(x - 1, y), self.at(x, y));
        std::array::from_fn(|channel| (left[channel] + right[channel]) / 2.0)
    }
}

/// Every miss of a test, so one failure shows them all.
#[derive(Default)]
struct Reads {
    misses: Vec<String>,
}
impl Reads {
    fn near(&mut self, what: &str, got: &[f64], want: &[f64], levels: f64) {
        let missed = got
            .iter()
            .zip(want)
            .any(|(got, want)| (got - want).abs() > levels);
        if missed {
            self.misses.push(format!(
                "{what}: read {got:.1?}, want {want:.1?} within {levels}"
            ));
        }
    }
    fn gray(&mut self, what: &str, got: [f64; 4], want: f64, levels: f64) {
        self.near(what, &got[..3], &[want; 3], levels);
    }
    fn finish(self) {
        assert!(self.misses.is_empty(), "\n{}", self.misses.join("\n"));
    }
}

// The Gaussian's integral up to x standard deviations, by Simpson's rule.
fn gaussian_integral(x: f64) -> f64 {
    const START: f64 = -8.0;
    const STEPS: u32 = 4096;
    let x = x.clamp(START, -START);
    let step = (x - START) / f64::from(STEPS);
    let density = |t: f64| (-t * t / 2.0).exp() / std::f64::consts::TAU.sqrt();
    let inner: f64 = (1..STEPS)
        .map(|i| density(START + f64::from(i) * step) * if i % 2 == 1 { 4.0 } else { 2.0 })
        .sum();
    (density(START) + inner + density(x)) * step / 3.0
}
/// A black and white edge after a blur, `past` pixels to the white side of it.
fn edge_level(past: f64, blur: f64) -> f64 {
    255.0 * gaussian_integral(past / blur)
}
/// A white bar on black after a blur, at a place along the blur.
fn bar_level(at: f64, (start, end): (f64, f64), blur: f64) -> f64 {
    edge_level(end - at, blur) - edge_level(start - at, blur)
}

#[derive(Clone, Copy, Debug)]
enum Axis {
    X,
    Y,
}
impl Axis {
    /// Across `width`, from `start` to `end` along this axis.
    fn band(self, start: f32, end: f32, width: f32) -> Aabb {
        match self {
            Self::X => Aabb::new(start, 0.0, end, width),
            Self::Y => Aabb::new(0.0, start, width, end),
        }
    }
    /// An x and y, given a place along this axis and a place across it.
    fn point<T>(self, along: T, across: T) -> (T, T) {
        match self {
            Self::X => (along, across),
            Self::Y => (across, along),
        }
    }
}

fn moved(x: f32, y: f32) -> Mat2D {
    Mat2D([1.0, 0.0, 0.0, 1.0, x, y])
}

// A frame recorded in a phone's order, with what a box does at its first paint:
// it asks, then fills itself with the picture, opaque and with no tint.
fn blurred_frame(
    (width, height): (u32, u32),
    clear: u32,
    request: &BackdropRequest,
    draw_behind: impl FnOnce(&mut dyn Factory, &mut dyn Renderer),
) -> Pixels {
    let mut host = GmHost::with_size(clear, false, width, height);
    let mut session = DeferredSession::with_caps(ReplayCaps::from(&*host.ore.borrow()));
    session.bind_render_context(host.factory.persistent_context());
    session.record_ore_replay_marker();
    session.set_screen(width, height, clear);
    let mut screen = session.make_screen_renderer(0);
    draw_behind(&mut session, screen.as_mut());
    let BackdropAnswer::Image { image, mapping } = screen.backdrop(request) else {
        panic!("a box on the screen gets a picture");
    };
    let path = rectangle(&mut session, request.rect);
    let mut paint = solid(&mut session, WHITE);
    paint.modulated_image(Some(image.as_ref()), ImageSampler::LINEAR_CLAMP, mapping);
    screen.save();
    screen.transform(request.world);
    screen.draw_path(path.as_ref(), paint.as_ref());
    screen.restore();
    replay(&mut session, &mut DeferredReplayer::default(), &mut host);
    Pixels {
        width: width as usize,
        bytes: host.finish(),
    }
}

// Bars: wide enough around the middle that no mirror image reaches a read.
const BARS_FRAME: u32 = 640;
const BARS_BOX_AT: f32 = 16.0;

fn bars_frame(axis: Axis, (start, end): (f64, f64), blur: f64) -> Pixels {
    let frame = BARS_FRAME as f32;
    let side = frame - 2.0 * BARS_BOX_AT;
    let request = BackdropRequest {
        rect: Aabb::new(0.0, 0.0, side, side),
        world: moved(BARS_BOX_AT, BARS_BOX_AT),
        blur: blur as f32,
        saturate: 1.0,
    };
    let frame_size = (BARS_FRAME, BARS_FRAME);
    blurred_frame(frame_size, BLACK, &request, |factory, renderer| {
        let white = axis.band(start as f32, end as f32, frame);
        fill(factory, renderer, white, WHITE);
    })
}
// `at` is a pixel's center, along the blur.
fn read_bar(reads: &mut Reads, pixels: &Pixels, axis: Axis, at: f64, bar: (f64, f64), blur: f64) {
    let (x, y) = axis.point(at as usize, BARS_FRAME as usize / 2);
    reads.gray(
        &format!("bar across {axis:?}, blur {blur}, at {at}"),
        pixels.at(x, y),
        bar_level(at, bar, blur),
        BAR_LEVELS,
    );
}

#[test]
fn backdrop_blur_matches_gaussian() {
    let middle = f64::from(BARS_FRAME) / 2.0;
    let bar = (middle - 1.0, middle + 1.0);
    let mut reads = Reads::default();
    for axis in [Axis::X, Axis::Y] {
        for blur in [8.0, 24.0, 47.0, 49.0, 90.0] {
            let pixels = bars_frame(axis, bar, blur);
            // On the bar, a blur away and two blurs away, on both sides.
            for distance in [0.5, blur + 0.5, 2.0 * blur + 0.5] {
                for at in [middle - distance, middle + distance] {
                    read_bar(&mut reads, &pixels, axis, at, bar, blur);
                }
            }
        }
    }
    reads.finish();
}

#[test]
fn backdrop_blur_matches_gaussian_mirrored_at_the_box_edge() {
    let blur = 8.0;
    let edge = f64::from(BARS_BOX_AT);
    // The bar and its mirror image meet on the box's edge. Black lies past it,
    // so a blur that read past the box would see half of this.
    let bar_and_mirror = (edge - 2.0, edge + 2.0);
    let mut reads = Reads::default();
    for axis in [Axis::X, Axis::Y] {
        let pixels = bars_frame(axis, (edge, edge + 2.0), blur);
        for inside in [0.5, 8.5] {
            let at = edge + inside;
            read_bar(&mut reads, &pixels, axis, at, bar_and_mirror, blur);
        }
    }
    reads.finish();
}

// Edges: black to the left of the middle, white to its right, wide enough that
// a mirror image adds less than a level at the farthest read.
const WIDE_FRAME: (u32, u32) = (3200, 64);
const WIDE_EDGE: f32 = 1600.0;

fn wide_edge_frame(rect: Aabb, world: Mat2D, blur: f32) -> Pixels {
    let request = BackdropRequest {
        rect,
        world,
        blur,
        saturate: 1.0,
    };
    let (width, height) = (WIDE_FRAME.0 as f32, WIDE_FRAME.1 as f32);
    blurred_frame(WIDE_FRAME, BLACK, &request, |factory, renderer| {
        let white = Axis::X.band(WIDE_EDGE, width, height);
        fill(factory, renderer, white, WHITE);
    })
}
// `dark` pixels off the edge on its dark side, and as far on its white side.
fn read_wide_edge(reads: &mut Reads, pixels: &Pixels, device_blur: f64, dark: usize) {
    let edge = WIDE_EDGE as usize;
    for x in [edge - dark - 1, edge + dark] {
        let past = x as f64 + 0.5 - f64::from(WIDE_EDGE);
        reads.gray(
            &format!("edge at {device_blur} px, {past} px past it"),
            pixels.at(x, WIDE_FRAME.1 as usize / 2),
            edge_level(past, device_blur),
            EDGE_LEVELS,
        );
    }
}
// Larger than the frame, so the screen cuts it to the whole frame.
fn turned_and_scaled_4_times() -> (Aabb, Mat2D) {
    const SIDE: f32 = 800.0;
    let (sin, cos) = 20f32.to_radians().sin_cos();
    let (scaled_cos, scaled_sin) = (4.0 * cos, 4.0 * sin);
    let half = SIDE / 2.0;
    let (center_x, center_y) = (WIDE_EDGE, WIDE_FRAME.1 as f32 / 2.0);
    let world = Mat2D([
        scaled_cos,
        scaled_sin,
        -scaled_sin,
        scaled_cos,
        center_x - half * (scaled_cos - scaled_sin),
        center_y - half * (scaled_sin + scaled_cos),
    ]);
    (Aabb::new(0.0, 0.0, SIDE, SIDE), world)
}

#[test]
fn backdrop_blur_matches_gaussian_at_large_blurs() {
    let (width, height) = (WIDE_FRAME.0 as f32, WIDE_FRAME.1 as f32);
    let whole = Aabb::new(0.0, 0.0, width, height);
    let mut reads = Reads::default();
    for (blur, dark) in [(97.0, 50), (180.0, 100), (900.0, 300)] {
        let pixels = wide_edge_frame(whole, Mat2D::IDENTITY, blur);
        read_wide_edge(&mut reads, &pixels, f64::from(blur), dark);
        read_wide_edge(&mut reads, &pixels, f64::from(blur), 2 * dark);
    }
    // 90 points on a box scaled 4 times is 360 device pixels.
    let (rect, world) = turned_and_scaled_4_times();
    let pixels = wide_edge_frame(rect, world, 90.0);
    read_wide_edge(&mut reads, &pixels, 360.0, 100);
    read_wide_edge(&mut reads, &pixels, 360.0, 200);
    reads.finish();
}

#[test]
fn backdrop_blur_matches_gaussian_on_an_unevenly_scaled_box() {
    let (width, height) = (WIDE_FRAME.0 as f32, WIDE_FRAME.1 as f32);
    let rect = Aabb::new(0.0, 0.0, width / 4.0, height);
    let world = Mat2D([4.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    let pixels = wide_edge_frame(rect, world, 45.0);
    let mut reads = Reads::default();
    // The square root of 4 by 1 makes 45 points 90 device pixels.
    read_wide_edge(&mut reads, &pixels, 90.0, 100);
    read_wide_edge(&mut reads, &pixels, 90.0, 200);
    reads.finish();
}

// A box smaller than the blur, with black around it.
const SMALL_FRAME: u32 = 340;
const SMALL_BOX_AT: f32 = 20.0;
const SMALL_BOX: f32 = 300.0;

// White over all but the first `black` pixels of the box.
fn small_box_frame(black: f32) -> Pixels {
    let request = BackdropRequest {
        rect: Aabb::new(0.0, 0.0, SMALL_BOX, SMALL_BOX),
        world: moved(SMALL_BOX_AT, SMALL_BOX_AT),
        blur: 900.0,
        saturate: 1.0,
    };
    let (start, end) = (SMALL_BOX_AT, SMALL_BOX_AT + SMALL_BOX);
    let frame_size = (SMALL_FRAME, SMALL_FRAME);
    blurred_frame(frame_size, BLACK, &request, |factory, renderer| {
        let white = Aabb::new(start + black, start, end, end);
        fill(factory, renderer, white, WHITE);
    })
}

#[test]
fn backdrop_blur_matches_gaussian_on_a_box_smaller_than_the_blur() {
    let middle = SMALL_FRAME as usize / 2;
    let mut reads = Reads::default();
    let half_black = small_box_frame(SMALL_BOX / 2.0);
    // Each side of the line, so a box with no blur reads 0 and 255.
    for x in [middle - 75, middle + 75] {
        reads.gray(
            &format!("half black box, at {x}"),
            half_black.at(x, middle),
            127.5,
            AVERAGE_LEVELS,
        );
    }
    let quarter_black = small_box_frame(SMALL_BOX / 4.0);
    reads.gray(
        "quarter black box, at its center",
        quarter_black.at(middle, middle),
        191.25,
        AVERAGE_LEVELS,
    );
    reads.finish();
}

// A box with almost no room across it, in a frame that holds the edge past it.
const THIN_LENGTH: u32 = 1206;
const THIN_BOX: u32 = 32;
const THIN_FRAME: u32 = 96;

fn thin_box_frame(axis: Axis) -> Pixels {
    let length = THIN_LENGTH as f32;
    let (x, y) = axis.point(0.0, THIN_BOX as f32);
    let request = BackdropRequest {
        rect: axis.band(0.0, length, THIN_BOX as f32),
        world: moved(x, y),
        blur: 180.0,
        saturate: 1.0,
    };
    let frame_size = axis.point(THIN_LENGTH, THIN_FRAME);
    blurred_frame(frame_size, BLACK, &request, |factory, renderer| {
        let white = axis.band(length / 2.0, length, THIN_FRAME as f32);
        fill(factory, renderer, white, WHITE);
    })
}

#[test]
fn backdrop_blur_matches_gaussian_on_thin_boxes() {
    let edge = THIN_LENGTH as usize / 2;
    let mut reads = Reads::default();
    for axis in [Axis::X, Axis::Y] {
        let pixels = thin_box_frame(axis);
        for along in [edge - 201, edge + 200] {
            let past = along as f64 + 0.5 - edge as f64;
            let (x, y) = axis.point(along, THIN_FRAME as usize / 2);
            reads.gray(
                &format!("thin box along {axis:?}, {past} px past the edge"),
                pixels.at(x, y),
                edge_level(past, 180.0),
                EDGE_LEVELS,
            );
        }
    }
    reads.finish();
}

// An edge on the texel grid of a capture at half size, with the box the frame.
const NARROW_FRAME: (u32, u32) = (256, 64);
const NARROW_EDGE: usize = 128;

fn narrow_edge_frame(blur: f32, saturate: f32) -> Pixels {
    let (width, height) = (NARROW_FRAME.0 as f32, NARROW_FRAME.1 as f32);
    let request = BackdropRequest {
        rect: Aabb::new(0.0, 0.0, width, height),
        world: Mat2D::IDENTITY,
        blur,
        saturate,
    };
    blurred_frame(NARROW_FRAME, BLACK, &request, |factory, renderer| {
        let white = Axis::X.band(NARROW_EDGE as f32, width, height);
        fill(factory, renderer, white, WHITE);
    })
}

#[test]
fn backdrop_blur_matches_gaussian_at_small_blurs() {
    let mut reads = Reads::default();
    let three_px = narrow_edge_frame(3.0, 1.0);
    for x in [NARROW_EDGE - 3, NARROW_EDGE + 2] {
        let past = x as f64 + 0.5 - NARROW_EDGE as f64;
        reads.gray(
            &format!("edge at 3 px, {past} px past it"),
            three_px.at(x, NARROW_FRAME.1 as usize / 2),
            edge_level(past, 3.0),
            EDGE_LEVELS,
        );
    }
    // Saturate keeps blur 0 drawing, on the one plain pass. At 0.4 px the
    // kernel's far weights vanish, and that blur has to land on the same picture.
    let (at_0_4_px, plain_pass) = (narrow_edge_frame(0.4, 1.8), narrow_edge_frame(0.0, 1.8));
    let worst = at_0_4_px
        .bytes
        .iter()
        .zip(&plain_pass.bytes)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap();
    reads.near(
        "worst channel at 0.4 px against the plain pass",
        &[f64::from(worst)],
        &[0.0],
        PLAIN_PASS_LEVELS,
    );
    reads.finish();
}

const SATURATE_FRAME: u32 = 256;
const SATURATE_BOX_AT: f32 = 16.0;

fn saturated_frame(
    clear: u32,
    blur: f32,
    saturate: f32,
    draw_behind: impl FnOnce(&mut dyn Factory, &mut dyn Renderer),
) -> Pixels {
    let side = SATURATE_FRAME as f32 - 2.0 * SATURATE_BOX_AT;
    let request = BackdropRequest {
        rect: Aabb::new(0.0, 0.0, side, side),
        world: moved(SATURATE_BOX_AT, SATURATE_BOX_AT),
        blur,
        saturate,
    };
    let frame_size = (SATURATE_FRAME, SATURATE_FRAME);
    blurred_frame(frame_size, clear, &request, draw_behind)
}

#[test]
fn backdrop_saturate_matches_css() {
    let frame = SATURATE_FRAME as f32;
    let whole = Aabb::new(0.0, 0.0, frame, frame);
    let middle = SATURATE_FRAME as usize / 2;
    let mut reads = Reads::default();
    for blur in [8.0, 0.0] {
        let flat = |saturate| {
            saturated_frame(CLEAR, blur, saturate, |factory, renderer| {
                fill(factory, renderer, whole, WARM);
            })
        };
        // luma + 1.8 (channel - luma), with Filter Effects' weights 0.213, 0.715
        // and 0.072: luma 102.7, red past 255, green 61.9, blue below 0.
        reads.near(
            &format!("flat at 1.8, blur {blur}"),
            &flat(1.8).at(middle, middle)[..3],
            &[255.0, 62.0, 0.0],
            FLAT_LEVELS,
        );
        reads.gray(
            &format!("flat at 0, blur {blur}"),
            flat(0.0).at(middle, middle),
            103.0,
            FLAT_LEVELS,
        );
        // Premultiplied this is (100.4, 40.2, 20.1, 128). Saturate gives red 139.5
        // and stops it at the alpha, 128, and the picture goes over the sharp
        // pixel, which adds half of that pixel back.
        let half_alpha = saturated_frame(CLEAR, blur, 1.8, |factory, renderer| {
            fill(factory, renderer, whole, WARM_AT_HALF_ALPHA);
        });
        reads.near(
            &format!("half alpha at 1.8, blur {blur}"),
            &half_alpha.at(middle, middle),
            &[178.0, 51.0, 10.0, 191.0],
            FLAT_LEVELS,
        );
    }
    let edge = saturated_frame(CLEAR, 8.0, 1.8, |factory, renderer| {
        let (warm, cool) = (0.0, frame / 2.0);
        fill(factory, renderer, Axis::X.band(warm, cool, frame), WARM);
        fill(factory, renderer, Axis::X.band(cool, frame, frame), COOL);
    });
    // Blur first: the two colors mixed by the closed form, then the matrix. The
    // mix is half and half on the line, and 84% and 16% a blur off it.
    for (what, x, want) in [
        (
            "a blur to the warm side of the line",
            middle - 8,
            [235.0, 65.0, 38.0],
        ),
        ("on the line", middle, [143.0, 71.0, 143.0]),
        (
            "a blur to the cool side of the line",
            middle + 8,
            [51.0, 77.0, 247.0],
        ),
    ] {
        let got = edge.line_before(x, middle);
        reads.near(what, &got, &want, SATURATED_EDGE_LEVELS);
    }
    reads.finish();
}
