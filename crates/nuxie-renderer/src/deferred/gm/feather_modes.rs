//! Feathered fills draw in both native Metal interlock modes (UNIV-5018).
//!
//! Upstream `RiveRenderer::drawPath` drops a feathered fill whose fill rule is
//! not clockwise unless the frame overrides fills to clockwise, so
//! raster-ordering frames (every iPhone) drew nothing where clockwise-atomic
//! frames drew the fill. Nuxie draws such a fill as a clockwise-override frame
//! does. The Rive editor writes feathered fills as clockwise, and those take
//! the upstream path unchanged, so they are the oracle: in each mode a
//! non-zero or even-odd feathered fill draws the clockwise one's pixels, and
//! the two modes agree to within a level.
use super::ore_gm_helper::GmHost;
use crate::RenderMode;
use nuxie_render_api::{Factory, FillRule, RawPath};
use nuxie_runtime::{File, RuntimeFactoryHandle};
use std::collections::BTreeMap;

const SIZE: u32 = 200;
const MODES: [RenderMode; 2] = [RenderMode::RasterOrdering, RenderMode::ClockwiseAtomic];

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

// Keys (defs/upstream-runtime).
const BACKBOARD: u16 = 23;
const ARTBOARD: u16 = 1;
const SHAPE: u16 = 3;
const RECTANGLE: u16 = 7;
const FILL: u16 = 20;
const SOLID_COLOR: u16 = 18;
const FEATHER: u16 = 533;
const PARENT_ID: u16 = 5;
const WIDTH: u16 = 7;
const HEIGHT: u16 = 8;
const X: u16 = 13;
const Y: u16 = 14;
const SCALE_X: u16 = 16;
const PATH_WIDTH: u16 = 20;
const PATH_HEIGHT: u16 = 21;
const CORNER_RADIUS_TL: u16 = 31;
const FILL_RULE: u16 = 40;
const COLOR_VALUE: u16 = 37;
const FEATHER_SPACE: u16 = 748;
const FEATHER_STRENGTH: u16 = 749;
const FEATHER_OFFSET_X: u16 = 750;
const FEATHER_OFFSET_Y: u16 = 751;
const FEATHER_INNER: u16 = 752;

/// A Rive rectangle with one feathered Fill, as an editor or the CSS
/// compiler writes it.
#[derive(Clone, Copy)]
struct Shape {
    radius: f32,
    mirrored: bool,
    strength: f32,
    local_space: bool,
    offset: f32,
    inner: bool,
}

const SOFT: Shape = Shape {
    radius: 0.0,
    mirrored: false,
    strength: 16.0,
    local_space: false,
    offset: 0.0,
    inner: false,
};

fn scene(shape: Shape, fill_rule: FillRule) -> Vec<u8> {
    let mut riv = Riv::default();
    riv.object(BACKBOARD).end();
    // 0 artboard, 1 shape, 2 rectangle, 3 fill, 4 colour, 5 feather.
    riv.object(ARTBOARD)
        .float(WIDTH, SIZE as f32)
        .float(HEIGHT, SIZE as f32)
        .end();
    riv.object(SHAPE)
        .uint(PARENT_ID, 0)
        .float(X, 100.0)
        .float(Y, 100.0)
        .float(SCALE_X, if shape.mirrored { -1.0 } else { 1.0 })
        .end();
    riv.object(RECTANGLE)
        .uint(PARENT_ID, 1)
        .float(PATH_WIDTH, 100.0)
        .float(PATH_HEIGHT, 60.0)
        .float(CORNER_RADIUS_TL, shape.radius)
        .end();
    let rule = match fill_rule {
        FillRule::NonZero => 0,
        FillRule::EvenOdd => 1,
        FillRule::Clockwise => 2,
    };
    riv.object(FILL)
        .uint(PARENT_ID, 1)
        .uint(FILL_RULE, rule)
        .end();
    riv.object(SOLID_COLOR)
        .uint(PARENT_ID, 3)
        .color(COLOR_VALUE, 0xcc000000)
        .end();
    riv.object(FEATHER)
        .uint(PARENT_ID, 3)
        .uint(FEATHER_SPACE, u64::from(shape.local_space))
        .float(FEATHER_STRENGTH, shape.strength)
        .float(FEATHER_OFFSET_X, shape.offset)
        .float(FEATHER_OFFSET_Y, shape.offset)
        .uint(FEATHER_INNER, u64::from(shape.inner))
        .end();
    riv.bytes()
}

fn render_scene(bytes: &[u8], mode: RenderMode) -> Vec<u8> {
    let mut host = GmHost::with_mode(0xffffffff, true, SIZE, SIZE, mode);
    let file = File::import(
        bytes,
        RuntimeFactoryHandle::from_factory(&mut host.factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("the feather scene imports");
    let artboard = file.with_file(File::artboard_default).expect("artboard");
    artboard.advance_default(0.0);
    let screen = host.screen();
    artboard.draw_internal(screen.borrow_mut().as_mut());
    drop(screen);
    host.finish()
}

fn render_path(path: RawPath, fill_rule: FillRule, mode: RenderMode) -> Vec<u8> {
    let mut host = GmHost::with_mode(0xffffffff, true, SIZE, SIZE, mode);
    let screen = host.screen();
    let mut paint = host.factory.make_render_paint();
    paint.color(0xcc000000);
    paint.feather(16.0);
    let path = host.factory.make_render_path(path, fill_rule);
    screen.borrow_mut().draw_path(path.as_ref(), paint.as_ref());
    drop(screen);
    host.finish()
}

/// Pixels darker than the white background.
fn inked(pixels: &[u8]) -> usize {
    pixels.chunks(4).filter(|pixel| pixel[0] < 255).count()
}

/// The largest per-pixel channel difference.
fn difference(a: &[u8], b: &[u8]) -> u8 {
    a.iter()
        .zip(b)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap_or(0)
}

fn save(name: &str, pixels: &[u8]) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("nuxie-univ-5018-{name}.png"));
    pixel_compare::RgbaImage::new(SIZE, SIZE, pixels.to_vec())
        .unwrap()
        .write_png(&path)
        .unwrap();
    path
}

/// How far the raster-ordering and clockwise-atomic feather evaluations
/// differ on the same clockwise fill: one level of rounding.
const MODES_MAX: u8 = 1;

/// In each mode, `draw(rule)` for a non-clockwise rule draws the clockwise
/// drawing's pixels, to within `rule_max` levels, and the modes agree.
fn check(name: &str, rule_max: u8, draw: impl Fn(FillRule, RenderMode) -> Vec<u8>) {
    let mut clockwise = Vec::new();
    for mode in MODES {
        let expected = draw(FillRule::Clockwise, mode);
        let ink = inked(&expected);
        assert!(
            ink > 4000,
            "{name} {mode:?}: the clockwise fill drew {ink} pixels"
        );
        for rule in [FillRule::NonZero, FillRule::EvenOdd] {
            let actual = draw(rule, mode);
            let max = difference(&expected, &actual);
            assert!(
                max <= rule_max,
                "{name} {mode:?}: the {rule:?} feathered fill differs from the clockwise one by \
                 {max} levels ({} pixels inked against {ink}); ours {}",
                inked(&actual),
                save(&format!("{name}-{mode:?}-{rule:?}"), &actual).display()
            );
        }
        clockwise.push(expected);
    }
    let max = difference(&clockwise[0], &clockwise[1]);
    assert!(
        max <= MODES_MAX,
        "{name}: raster-ordering and clockwise-atomic differ by {max} levels; raster {}, atomic {}",
        save(&format!("{name}-raster"), &clockwise[0]).display(),
        save(&format!("{name}-atomic"), &clockwise[1]).display()
    );
}

fn check_shape(name: &str, shape: Shape) {
    check(name, 0, |rule, mode| {
        render_scene(&scene(shape, rule), mode)
    });
}

#[test]
fn feathered_rectangle_draws_in_both_modes() {
    check_shape("rectangle", SOFT);
}

#[test]
fn feathered_drop_shadow_draws_in_both_modes() {
    // A soft, rounded shadow offset down and right in the shape's space, the
    // way a CSS box-shadow is written.
    check_shape(
        "drop-shadow",
        Shape {
            radius: 12.0,
            local_space: true,
            offset: 8.0,
            ..SOFT
        },
    );
}

#[test]
fn feathered_world_offset_draws_in_both_modes() {
    check_shape(
        "world-offset",
        Shape {
            offset: 8.0,
            ..SOFT
        },
    );
}

#[test]
fn inner_feather_draws_in_both_modes() {
    check_shape(
        "inner",
        Shape {
            radius: 12.0,
            inner: true,
            ..SOFT
        },
    );
}

#[test]
fn mirrored_feathered_shape_draws_in_both_modes() {
    check_shape(
        "mirrored",
        Shape {
            radius: 12.0,
            mirrored: true,
            ..SOFT
        },
    );
}

#[test]
fn wide_feather_through_the_atlas_draws_in_both_modes() {
    check_shape(
        "atlas",
        Shape {
            strength: 60.0,
            ..SOFT
        },
    );
}

#[test]
fn counterclockwise_feathered_fill_draws_in_both_modes() {
    // A non-clockwise rule draws a counterclockwise path as clockwise-override
    // frames do, flipped to clockwise, so it matches the clockwise winding to
    // within the rounding of the flipped coverage.
    let rect = |clockwise: bool| {
        let corners = [(50.0, 70.0), (150.0, 70.0), (150.0, 130.0), (50.0, 130.0)];
        let order = if clockwise {
            [0, 1, 2, 3]
        } else {
            [0, 3, 2, 1]
        };
        let mut path = RawPath::new();
        path.move_to(corners[order[0]].0, corners[order[0]].1);
        for &index in &order[1..] {
            path.line_to(corners[index].0, corners[index].1);
        }
        path.close();
        path
    };
    check("counterclockwise", 1, |rule, mode| {
        render_path(rect(rule == FillRule::Clockwise), rule, mode)
    });
}
