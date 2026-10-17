//! CSS `box-shadow` on a CSS layout box (docs/CSS_LAYOUT.md, 4.1 and 4.2),
//! against Chrome's own pixels for the same CSS. The references are
//! fixtures/css_layout/chrome/*.png, made by tools/css-layout/
//! chrome-references.sh from the .html beside them. The box draws a
//! feathered fill whose strength is the CSS blur radius (both are two
//! standard deviations of the Gaussian), offset by the CSS offsets, with
//! BoxShadow's spread and CSS clipping: an outer shadow only outside the
//! border box, an inset one only inside the padding box.
use super::ore_gm_helper::*;
use crate::deferred::cmd::{
    deferred_replayer::{snapshot_frame, DeferredReplayer},
    deferred_session::DeferredSession,
};
use nuxie_runtime::{File, RuntimeFactoryHandle};
use std::collections::BTreeMap;

const SIZE: u32 = 200;
/// Feathered fills are exact against C++ in clockwise-atomic mode
/// (corpus-r.toml's gm-feather_* rows).
const MODE: crate::RenderMode = crate::RenderMode::ClockwiseAtomic;

#[derive(Default)]
struct Riv {
    objects: Vec<u8>,
    toc: BTreeMap<u16, u32>,
}
fn varuint(out: &mut Vec<u8>, mut value: u64) {
    while value >= 128 {
        out.push((value as u8 & 127) | 128);
        value >>= 7;
    }
    out.push(value as u8);
}
impl Riv {
    fn object(&mut self, key: u16) -> &mut Self {
        varuint(&mut self.objects, key.into());
        self
    }
    fn end(&mut self) {
        varuint(&mut self.objects, 0);
    }
    fn uint(&mut self, key: u16, value: u64) -> &mut Self {
        self.toc.insert(key, 0);
        varuint(&mut self.objects, key.into());
        varuint(&mut self.objects, value);
        self
    }
    fn float(&mut self, key: u16, value: f32) -> &mut Self {
        self.toc.insert(key, 2);
        varuint(&mut self.objects, key.into());
        self.objects.extend_from_slice(&value.to_le_bytes());
        self
    }
    fn color(&mut self, key: u16, value: u32) -> &mut Self {
        self.toc.insert(key, 3);
        varuint(&mut self.objects, key.into());
        self.objects.extend_from_slice(&value.to_le_bytes());
        self
    }
    fn bytes(&self) -> Vec<u8> {
        let mut out = b"RIVE".to_vec();
        varuint(&mut out, File::MAJOR_VERSION as u64);
        varuint(&mut out, File::MINOR_VERSION as u64);
        varuint(&mut out, 0);
        for &key in self.toc.keys() {
            varuint(&mut out, key.into());
        }
        varuint(&mut out, 0);
        let mut packed = 0u32;
        let mut bit = 0;
        for &field in self.toc.values() {
            packed |= field << bit;
            bit += 2;
            if bit == 8 {
                out.extend_from_slice(&packed.to_le_bytes());
                packed = 0;
                bit = 0;
            }
        }
        if bit != 0 {
            out.extend_from_slice(&packed.to_le_bytes());
        }
        out.extend_from_slice(&self.objects);
        out
    }
}

// Keys (defs/upstream-runtime and defs/nuxie).
const BACKBOARD: u16 = 23;
const ARTBOARD: u16 = 1;
const LAYOUT_COMPONENT: u16 = 409;
const LAYOUT_COMPONENT_STYLE: u16 = 420;
const FILL: u16 = 20;
const SOLID_COLOR: u16 = 18;
const FEATHER: u16 = 533;
const LAYOUT_CSS_STYLE: u16 = 60002;
const BOX_SHADOW: u16 = 60005;
const STROKE: u16 = 24;
const BOX_BORDER: u16 = 60003;
const LINEAR_GRADIENT: u16 = 22;
const RADIAL_GRADIENT: u16 = 17;
const GRADIENT_STOP: u16 = 19;
const STOP_COLOR: u16 = 38;
const STOP_POSITION: u16 = 39;
const BOX_GRADIENT: u16 = 60004;
const GRADIENT_ANGLE: u16 = 60106;
const GRADIENT_SHAPE: u16 = 60107;
const GRADIENT_SIZE: u16 = 60108;
const GRADIENT_CENTER_X_PERCENT: u16 = 60110;
const GRADIENT_CENTER_Y_PERCENT: u16 = 60112;
const GRADIENT_TO_CORNER: u16 = 60117;
const BORDER_SIDES: u16 = 60105;
const PARENT_ID: u16 = 5;
const WIDTH: u16 = 7;
const HEIGHT: u16 = 8;
const STYLE_ID: u16 = 494;
const CORNER_RADIUS_TL: u16 = 640;
const LINK_CORNER_RADIUS: u16 = 639;
const COLOR_VALUE: u16 = 37;
const FEATHER_SPACE: u16 = 748;
const FEATHER_STRENGTH: u16 = 749;
const FEATHER_OFFSET_X: u16 = 750;
const FEATHER_OFFSET_Y: u16 = 751;
const FEATHER_INNER: u16 = 752;
const CSS_POSITION: u16 = 60024;
const CSS_WIDTH_KIND: u16 = 60043;
const CSS_WIDTH: u16 = 60044;
const CSS_HEIGHT_KIND: u16 = 60046;
const CSS_HEIGHT: u16 = 60047;
const CSS_INSET_TOP_KIND: u16 = 60073;
const CSS_INSET_TOP: u16 = 60074;
const CSS_INSET_LEFT_KIND: u16 = 60082;
const CSS_INSET_LEFT: u16 = 60083;
const CSS_BORDER_TOP: u16 = 60093;
const SHADOW_SPREAD: u16 = 60118;

/// One CSS box with one `box-shadow`, as the compiler writes it.
#[derive(Clone, Copy)]
struct Shadow {
    // The box's border-box position and content size, and its border.
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    border: f32,
    radius: f32,
    // box-shadow: [inset] x y blur spread color.
    inset: bool,
    x: f32,
    y: f32,
    blur: f32,
    spread: f32,
    color: u32,
}

/// The scene for `shadow`, with the feather strength the mapping gives the
/// blur, scaled by `strength_scale` (1 is the contract's mapping).
fn scene(shadow: Shadow, strength_scale: f32) -> Vec<u8> {
    let mut riv = Riv::default();
    riv.object(BACKBOARD).end();
    // 0 artboard, 1 box, 2 record, 3 fill, 4 colour, 5 feather, 6 BoxShadow,
    // 7 the artboard's style, 8 the box's style.
    riv.object(ARTBOARD)
        .float(WIDTH, SIZE as f32)
        .float(HEIGHT, SIZE as f32)
        .uint(STYLE_ID, 7)
        .end();
    riv.object(LAYOUT_COMPONENT)
        .uint(PARENT_ID, 0)
        .uint(STYLE_ID, 8)
        .end();
    riv.object(LAYOUT_CSS_STYLE)
        .uint(PARENT_ID, 1)
        .uint(CSS_POSITION, 2)
        .uint(CSS_INSET_LEFT_KIND, 1)
        .float(CSS_INSET_LEFT, shadow.left)
        .uint(CSS_INSET_TOP_KIND, 1)
        .float(CSS_INSET_TOP, shadow.top)
        .uint(CSS_WIDTH_KIND, 1)
        .float(CSS_WIDTH, shadow.width)
        .uint(CSS_HEIGHT_KIND, 1)
        .float(CSS_HEIGHT, shadow.height);
    for side in 0..4 {
        riv.float(CSS_BORDER_TOP + side, shadow.border);
    }
    riv.end();
    riv.object(FILL).uint(PARENT_ID, 1).end();
    riv.object(SOLID_COLOR)
        .uint(PARENT_ID, 3)
        .color(COLOR_VALUE, shadow.color)
        .end();
    riv.object(FEATHER)
        .uint(PARENT_ID, 3)
        // Local space: a shadow moves with its box, as in CSS.
        .uint(FEATHER_SPACE, 1)
        .float(FEATHER_STRENGTH, shadow.blur * strength_scale)
        .float(FEATHER_OFFSET_X, shadow.x)
        .float(FEATHER_OFFSET_Y, shadow.y)
        .uint(FEATHER_INNER, u64::from(shadow.inset))
        .end();
    riv.object(BOX_SHADOW)
        .uint(PARENT_ID, 5)
        .float(SHADOW_SPREAD, shadow.spread)
        .end();
    riv.object(LAYOUT_COMPONENT_STYLE).uint(PARENT_ID, 0).end();
    riv.object(LAYOUT_COMPONENT_STYLE)
        .uint(PARENT_ID, 1)
        .uint(LINK_CORNER_RADIUS, 1)
        .float(CORNER_RADIUS_TL, shadow.radius)
        .end();
    riv.bytes()
}

fn render(bytes: &[u8]) -> Vec<u8> {
    render_with(bytes, false)
}

/// The artboard drawn straight into the Metal screen, or through a deferred
/// session and its replay.
fn render_with(bytes: &[u8], deferred: bool) -> Vec<u8> {
    if !deferred {
        let mut host = GmHost::with_mode(0xffffffff, true, SIZE, SIZE, MODE);
        let file = File::import(
            bytes,
            RuntimeFactoryHandle::from_factory(&mut host.factory).unwrap(),
            None,
            None,
            None,
        )
        .expect("the shadow scene imports");
        let artboard = file.with_file(File::artboard_default).expect("artboard");
        for _ in 0..3 {
            artboard.advance_default(1.0 / 60.0);
        }
        let screen = host.screen();
        artboard.draw_internal(screen.borrow_mut().as_mut());
        drop(screen);
        return host.finish();
    }
    let mut host = GmHost::with_mode(0xffffffff, false, SIZE, SIZE, MODE);
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
    .expect("the shadow scene imports");
    let artboard = file.with_file(File::artboard_default).expect("artboard");
    for _ in 0..3 {
        artboard.advance_default(1.0 / 60.0);
    }
    let screen = session.borrow().screen_renderer(0);
    artboard.draw_internal(screen.borrow_mut().as_mut());
    let frame = snapshot_frame(&mut session.borrow_mut());
    session.borrow_mut().reset_frame();
    DeferredReplayer::default().replay_frame(&frame, &mut host);
    host.finish_frame()
}

/// Mean and largest per-pixel difference (the largest of the colour
/// channels), in 8-bit levels, and how many pixels differ by more than 8.
fn difference(chrome: &[u8], ours: &[u8]) -> (f32, u8, usize) {
    let mut sum = 0u64;
    let mut max = 0u8;
    let mut outliers = 0;
    for (a, b) in chrome.chunks(4).zip(ours.chunks(4)) {
        let delta = (0..3).map(|c| a[c].abs_diff(b[c])).max().unwrap_or(0);
        sum += u64::from(delta);
        max = max.max(delta);
        outliers += usize::from(delta > 8);
    }
    (sum as f32 / (chrome.len() / 4) as f32, max, outliers)
}

fn chrome(name: &str) -> Vec<u8> {
    let image =
        pixel_compare::RgbaImage::decode_png(&fixture(&format!("css_layout/chrome/{name}.png")))
            .expect("Chrome reference");
    assert_eq!((image.width, image.height), (SIZE, SIZE));
    image.pixels
}

/// How close to Chrome a scene must be: the mean and largest difference,
/// and how many pixels may differ by more than 8 levels.
struct Bound {
    mean: f32,
    max: u8,
    outliers: usize,
}
/// Straight edges on whole pixels: a few levels of Gaussian approximation.
const STRAIGHT: Bound = Bound {
    mean: 0.5,
    max: 8,
    outliers: 0,
};

/// The contract's mapping matches Chrome within `bound`, and better than
/// half or double the strength would.
fn check(name: &str, shadow: Shadow, bound: Bound) {
    let reference = chrome(name);
    let ours = render(&scene(shadow, 1.0));
    let (mean, max, outliers) = difference(&reference, &ours);
    let path = std::env::temp_dir().join(format!("nuxie-css-{name}.png"));
    pixel_compare::RgbaImage::new(SIZE, SIZE, ours.clone())
        .unwrap()
        .write_png(&path)
        .unwrap();
    eprintln!(
        "{name}: mean {mean:.3}, max {max}, {outliers} pixels over 8 levels; ours {}",
        path.display()
    );
    for scale in [0.5, 2.0] {
        let (other, _, _) = difference(&reference, &render(&scene(shadow, scale)));
        eprintln!("{name}: strength x{scale}: mean {other:.3}");
        assert!(
            other > 2.0 * mean,
            "{name}: strength x{scale} is not clearly worse ({other} against {mean})"
        );
    }
    assert!(
        mean <= bound.mean && max <= bound.max && outliers <= bound.outliers,
        "{name}: mean {mean} (at most {}), max {max} (at most {}), {outliers} outliers (at most {})",
        bound.mean,
        bound.max,
        bound.outliers
    );
}

const BOX: Shadow = Shadow {
    left: 50.0,
    top: 70.0,
    width: 100.0,
    height: 60.0,
    border: 0.0,
    radius: 0.0,
    inset: false,
    x: 0.0,
    y: 0.0,
    blur: 0.0,
    spread: 0.0,
    color: 0xff000000,
};

#[test]
fn css_shadow_blur_only_matches_chrome() {
    check("shadow_blur_only", Shadow { blur: 16.0, ..BOX }, STRAIGHT);
}

#[test]
fn css_shadow_outer_spread_matches_chrome() {
    check(
        "shadow_outer_spread",
        Shadow {
            x: 10.0,
            y: 10.0,
            blur: 20.0,
            spread: 5.0,
            ..BOX
        },
        STRAIGHT,
    );
}

#[test]
fn css_shadow_inset_clips_to_the_padding_box_like_chrome() {
    check(
        "shadow_inset_border",
        Shadow {
            left: 44.0,
            top: 64.0,
            border: 6.0,
            inset: true,
            x: 8.0,
            y: 6.0,
            blur: 12.0,
            spread: 4.0,
            ..BOX
        },
        STRAIGHT,
    );
}

#[test]
fn css_shadow_rounded_matches_chrome() {
    check(
        "shadow_rounded",
        Shadow {
            radius: 16.0,
            y: 4.0,
            blur: 24.0,
            spread: 8.0,
            color: 0x99000000,
            ..BOX
        },
        // The knock-out's corner arcs are anti-aliased less than Chrome's:
        // under a hundred edge pixels differ by up to a quarter of the range.
        Bound {
            mean: 0.5,
            max: 64,
            outliers: 100,
        },
    );
}

/// Per-side borders (BoxBorder): each Stroke fills the CSS border ring of its
/// sides, from the record's four widths, with inner radii and diagonal
/// joins, against Chrome's `border-width: 6px 2px 10px 4px` with black top
/// and left, red right and bottom, and a 12-point radius.
#[test]
fn css_border_sides_match_chrome() {
    let mut riv = Riv::default();
    riv.object(BACKBOARD).end();
    // 0 artboard, 1 box, 2 record, 3 and 6 strokes, 4 and 7 colours,
    // 5 and 8 BoxBorders, 9 and 10 styles.
    riv.object(ARTBOARD)
        .float(WIDTH, SIZE as f32)
        .float(HEIGHT, SIZE as f32)
        .uint(STYLE_ID, 9)
        .end();
    riv.object(LAYOUT_COMPONENT)
        .uint(PARENT_ID, 0)
        .uint(STYLE_ID, 10)
        .end();
    riv.object(LAYOUT_CSS_STYLE)
        .uint(PARENT_ID, 1)
        .uint(CSS_POSITION, 2)
        .uint(CSS_INSET_LEFT_KIND, 1)
        .float(CSS_INSET_LEFT, 50.0)
        .uint(CSS_INSET_TOP_KIND, 1)
        .float(CSS_INSET_TOP, 70.0)
        .uint(CSS_WIDTH_KIND, 1)
        .float(CSS_WIDTH, 100.0)
        .uint(CSS_HEIGHT_KIND, 1)
        .float(CSS_HEIGHT, 60.0);
    for (side, width) in [6.0, 2.0, 10.0, 4.0].into_iter().enumerate() {
        riv.float(CSS_BORDER_TOP + side as u16, width);
    }
    riv.end();
    for (stroke, color, sides) in [(3u64, 0xff000000u32, 9u64), (6, 0xffff0000, 6)] {
        riv.object(STROKE).uint(PARENT_ID, 1).end();
        riv.object(SOLID_COLOR)
            .uint(PARENT_ID, stroke)
            .color(COLOR_VALUE, color)
            .end();
        riv.object(BOX_BORDER)
            .uint(PARENT_ID, stroke)
            .uint(BORDER_SIDES, sides)
            .end();
    }
    riv.object(LAYOUT_COMPONENT_STYLE).uint(PARENT_ID, 0).end();
    riv.object(LAYOUT_COMPONENT_STYLE)
        .uint(PARENT_ID, 1)
        .uint(LINK_CORNER_RADIUS, 1)
        .float(CORNER_RADIUS_TL, 12.0)
        .end();
    let reference = chrome("border_sides");
    let ours = render(&riv.bytes());
    let (mean, max, outliers) = difference(&reference, &ours);
    let path = std::env::temp_dir().join("nuxie-css-border_sides.png");
    pixel_compare::RgbaImage::new(SIZE, SIZE, ours)
        .unwrap()
        .write_png(&path)
        .unwrap();
    eprintln!(
        "border_sides: mean {mean:.3}, max {max}, {outliers} pixels over 8 levels; ours {}",
        path.display()
    );
    // Curved edges and the corner joins are anti-aliased differently.
    assert!(
        mean <= 1.0 && outliers <= 150,
        "border_sides: mean {mean}, max {max}, {outliers} outliers"
    );
}

/// A 100 by 60 CSS box at (50, 70) with one gradient background and its
/// BoxGradient (`placement`: uint and float properties).
fn gradient_scene(radial: bool, colors: [u32; 2], placement: &[(u16, f32)]) -> Vec<u8> {
    let mut riv = Riv::default();
    riv.object(BACKBOARD).end();
    // 0 artboard, 1 box, 2 record, 3 fill, 4 gradient, 5 and 6 stops,
    // 7 BoxGradient, 8 and 9 styles.
    riv.object(ARTBOARD)
        .float(WIDTH, SIZE as f32)
        .float(HEIGHT, SIZE as f32)
        .uint(STYLE_ID, 8)
        .end();
    riv.object(LAYOUT_COMPONENT)
        .uint(PARENT_ID, 0)
        .uint(STYLE_ID, 9)
        .end();
    riv.object(LAYOUT_CSS_STYLE)
        .uint(PARENT_ID, 1)
        .uint(CSS_POSITION, 2)
        .uint(CSS_INSET_LEFT_KIND, 1)
        .float(CSS_INSET_LEFT, 50.0)
        .uint(CSS_INSET_TOP_KIND, 1)
        .float(CSS_INSET_TOP, 70.0)
        .uint(CSS_WIDTH_KIND, 1)
        .float(CSS_WIDTH, 100.0)
        .uint(CSS_HEIGHT_KIND, 1)
        .float(CSS_HEIGHT, 60.0)
        .end();
    riv.object(FILL).uint(PARENT_ID, 1).end();
    riv.object(if radial {
        RADIAL_GRADIENT
    } else {
        LINEAR_GRADIENT
    })
    .uint(PARENT_ID, 3)
    .end();
    for (color, position) in colors.into_iter().zip([0.0, 1.0]) {
        riv.object(GRADIENT_STOP)
            .uint(PARENT_ID, 4)
            .color(STOP_COLOR, color)
            .float(STOP_POSITION, position)
            .end();
    }
    riv.object(BOX_GRADIENT).uint(PARENT_ID, 4);
    for &(key, value) in placement {
        if matches!(key, GRADIENT_SHAPE | GRADIENT_SIZE | GRADIENT_TO_CORNER) {
            riv.uint(key, value as u64);
        } else {
            riv.float(key, value);
        }
    }
    riv.end();
    riv.object(LAYOUT_COMPONENT_STYLE).uint(PARENT_ID, 0).end();
    riv.object(LAYOUT_COMPONENT_STYLE).uint(PARENT_ID, 1).end();
    riv.bytes()
}

fn check_gradient(name: &str, bytes: &[u8]) {
    let reference = chrome(name);
    let ours = render(bytes);
    let (mean, max, outliers) = difference(&reference, &ours);
    let path = std::env::temp_dir().join(format!("nuxie-css-{name}.png"));
    pixel_compare::RgbaImage::new(SIZE, SIZE, ours)
        .unwrap()
        .write_png(&path)
        .unwrap();
    eprintln!(
        "{name}: mean {mean:.3}, max {max}, {outliers} pixels over 8 levels; ours {}",
        path.display()
    );
    // Interpolation and dithering differ by a level or two.
    assert!(
        mean <= 1.0 && max <= 8,
        "{name}: mean {mean}, max {max}, {outliers} outliers"
    );
}

/// `linear-gradient(135deg, red, blue)`: the line through the centre whose
/// ends' perpendiculars touch the far corners.
#[test]
fn css_gradient_linear_angle_matches_chrome() {
    check_gradient(
        "gradient_linear_angle",
        &gradient_scene(false, [0xffff0000, 0xff0000ff], &[(GRADIENT_ANGLE, 135.0)]),
    );
}

/// `linear-gradient(to top right, lime, black)`: CSS's corner angle, which
/// depends on the box's shape.
#[test]
fn css_gradient_linear_corner_matches_chrome() {
    check_gradient(
        "gradient_linear_corner",
        &gradient_scene(
            false,
            [0xff00ff00, 0xff000000],
            &[(GRADIENT_TO_CORNER, 1.0)],
        ),
    );
}

/// `radial-gradient(white, black)`: a farthest-corner ellipse at the centre.
#[test]
fn css_gradient_radial_ellipse_matches_chrome() {
    check_gradient(
        "gradient_radial_ellipse",
        &gradient_scene(true, [0xffffffff, 0xff000000], &[]),
    );
}

/// `radial-gradient(circle closest-side at 30% 40%, red, blue)`.
#[test]
fn css_gradient_radial_circle_matches_chrome() {
    check_gradient(
        "gradient_radial_circle",
        &gradient_scene(
            true,
            [0xffff0000, 0xff0000ff],
            &[
                (GRADIENT_SHAPE, 1.0),
                (GRADIENT_SIZE, 3.0),
                (GRADIENT_CENTER_X_PERCENT, 30.0),
                (GRADIENT_CENTER_Y_PERCENT, 40.0),
            ],
        ),
    );
}
