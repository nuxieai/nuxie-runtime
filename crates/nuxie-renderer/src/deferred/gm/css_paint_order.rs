//! CSS stacking contexts against Chrome's flat-color reference pixels.
//! The HTML fixtures are rendered by tools/css-layout/chrome-references.sh.
use super::ore_gm_helper::*;
use crate::RenderMode;
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

// Keys from defs/upstream-runtime and defs/nuxie.
const BACKBOARD: u16 = 23;
const ARTBOARD: u16 = 1;
const LAYOUT: u16 = 409;
const STYLE: u16 = 420;
const CSS: u16 = 61000;
const FILL: u16 = 20;
const SOLID_COLOR: u16 = 18;
const PARENT: u16 = 5;
const WIDTH: u16 = 7;
const HEIGHT: u16 = 8;
const STYLE_ID: u16 = 494;
const COLOR: u16 = 37;
const OPACITY: u16 = 18;
const POSITION: u16 = 61002;
const WIDTH_KIND: u16 = 61021;
const CSS_WIDTH: u16 = 61022;
const HEIGHT_KIND: u16 = 61024;
const CSS_HEIGHT: u16 = 61025;
const TOP_KIND: u16 = 61051;
const TOP: u16 = 61052;
const LEFT_KIND: u16 = 61060;
const LEFT: u16 = 61061;
const MARGIN_TOP: u16 = 61040;
const MARGIN_LEFT: u16 = 61049;
const Z_KIND: u16 = 61099;
const Z: u16 = 61100;
const RED: u32 = 0xffcc3333;
const BLUE: u32 = 0xff3333cc;
const GREEN: u32 = 0xff33cc33;

#[derive(Clone)]
struct BoxSpec {
    parent: Option<usize>,
    size: (f32, f32),
    position: u64,
    inset: (f32, f32),
    margin: (f32, f32),
    z: Option<f32>,
    opacity: f32,
    color: Option<u32>,
    keyed_rotation: bool,
}

impl BoxSpec {
    fn new(parent: Option<usize>, size: (f32, f32), color: Option<u32>) -> Self {
        Self {
            parent,
            size,
            color,
            position: 0,
            inset: (0.0, 0.0),
            margin: (0.0, 0.0),
            z: None,
            opacity: 1.0,
            keyed_rotation: false,
        }
    }
}

/// Every box retains its style. The artboard is CSS too, so negative
/// descendants can paint behind a non-context parent's background.
fn scene(boxes: &[BoxSpec]) -> Vec<u8> {
    let first_style = 2 + boxes
        .iter()
        .map(|b| 2 + if b.color.is_some() { 2 } else { 0 })
        .sum::<u64>();
    let mut riv = Riv::default();
    riv.object(BACKBOARD).end();
    riv.object(ARTBOARD)
        .float(WIDTH, SIZE as f32)
        .float(HEIGHT, SIZE as f32)
        .uint(STYLE_ID, first_style)
        .end();
    riv.object(CSS).uint(PARENT, 0).end();
    let mut ids = Vec::new();
    let mut next = 2u64;
    for (index, b) in boxes.iter().enumerate() {
        let id = next;
        ids.push(id);
        riv.object(LAYOUT)
            .uint(PARENT, b.parent.map_or(0, |parent| ids[parent]))
            .uint(STYLE_ID, first_style + 1 + index as u64)
            .float(OPACITY, b.opacity)
            .end();
        riv.object(CSS)
            .uint(PARENT, id)
            .uint(POSITION, b.position)
            .uint(WIDTH_KIND, 1)
            .float(CSS_WIDTH, b.size.0)
            .uint(HEIGHT_KIND, 1)
            .float(CSS_HEIGHT, b.size.1)
            .uint(TOP_KIND, 1)
            .float(TOP, b.inset.1)
            .uint(LEFT_KIND, 1)
            .float(LEFT, b.inset.0)
            .float(MARGIN_LEFT, b.margin.0)
            .float(MARGIN_TOP, b.margin.1);
        if let Some(z) = b.z {
            riv.uint(Z_KIND, 1).float(Z, z);
        }
        riv.end();
        next += 2;
        if let Some(color) = b.color {
            riv.object(FILL).uint(PARENT, id).end();
            riv.object(SOLID_COLOR)
                .uint(PARENT, next)
                .color(COLOR, color)
                .end();
            next += 2;
        }
    }
    riv.object(STYLE).uint(PARENT, 0).end();
    for id in &ids {
        riv.object(STYLE).uint(PARENT, *id).end();
    }
    for (index, b) in boxes.iter().enumerate() {
        if b.keyed_rotation {
            // A held rotate(0) is still an authored transform.
            riv.object(31).uint(56, 60).uint(57, 60).end();
            riv.object(25).uint(51, ids[index]).end();
            riv.object(26).uint(53, 15).end();
            riv.object(30).uint(67, 0).uint(68, 1).float(70, 0.0).end();
        }
    }
    riv.bytes()
}

fn render(bytes: &[u8], mode: RenderMode) -> Vec<u8> {
    let mut host = GmHost::with_mode(0xffffffff, true, SIZE, SIZE, mode);
    let file = File::import(
        bytes,
        RuntimeFactoryHandle::from_factory(&mut host.factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("the scene imports");
    let artboard = file.with_file(File::artboard_default).expect("artboard");
    for _ in 0..3 {
        artboard.advance_default(1.0 / 60.0);
    }
    let screen = host.screen();
    artboard.draw_internal(screen.borrow_mut().as_mut());
    drop(screen);
    host.finish()
}

/// Whole-pixel edges and solid colors allow at most two levels in every
/// channel, including the rounding of the single-paint opacity scenes.
fn check(name: &str, boxes: &[BoxSpec]) {
    let reference =
        pixel_compare::RgbaImage::decode_png(&fixture(&format!("css_layout/chrome/{name}.png")))
            .expect("Chrome reference");
    assert_eq!((reference.width, reference.height), (SIZE, SIZE));
    let bytes = scene(boxes);
    for mode in MODES {
        let ours = render(&bytes, mode);
        assert_eq!(ours.len(), reference.pixels.len());
        let max = ours
            .chunks_exact(4)
            .zip(reference.pixels.chunks_exact(4))
            .flat_map(|(ours, chrome)| (0..3).map(move |c| ours[c].abs_diff(chrome[c])))
            .max()
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("nuxie-css-{name}-{mode:?}.png"));
        pixel_compare::RgbaImage::new(SIZE, SIZE, ours)
            .unwrap()
            .write_png(&path)
            .unwrap();
        assert!(
            max <= 2,
            "{name} {mode:?}: max channel difference {max}; ours {}",
            path.display()
        );
    }
}

#[test]
fn css_order_badge_escapes_card_matches_chrome() {
    check(
        "order_badge_escapes_card",
        &[
            BoxSpec::new(None, (200.0, 100.0), Some(RED)),
            BoxSpec {
                position: 2,
                inset: (10.0, 80.0),
                z: Some(10.0),
                ..BoxSpec::new(Some(0), (60.0, 40.0), Some(GREEN))
            },
            BoxSpec {
                margin: (0.0, -30.0),
                ..BoxSpec::new(None, (200.0, 100.0), Some(BLUE))
            },
        ],
    );
}

#[test]
fn css_order_negative_z_in_context_matches_chrome() {
    let mut boxes = Vec::new();
    for (left, context) in [(0.0, true), (100.0, false)] {
        let wrapper = boxes.len();
        boxes.push(BoxSpec {
            position: 2,
            inset: (left, 40.0),
            ..BoxSpec::new(None, (100.0, 120.0), None)
        });
        let card = boxes.len();
        boxes.push(BoxSpec {
            position: u64::from(context),
            z: context.then_some(0.0),
            ..BoxSpec::new(Some(wrapper), (100.0, 100.0), Some(RED))
        });
        boxes.push(BoxSpec {
            position: 1,
            inset: (10.0, 10.0),
            z: Some(-1.0),
            ..BoxSpec::new(Some(card), (60.0, 40.0), Some(GREEN))
        });
        if context {
            boxes.push(BoxSpec {
                margin: (30.0, -20.0),
                ..BoxSpec::new(Some(card), (60.0, 40.0), Some(BLUE))
            });
        }
    }
    check("order_negative_z_in_context", &boxes);
}

#[test]
fn css_order_opacity_context_matches_chrome() {
    let mut boxes = Vec::new();
    for (top, later_opacity) in [(0.0, true), (100.0, false)] {
        let wrapper = boxes.len();
        boxes.push(BoxSpec {
            position: 2,
            inset: (0.0, top),
            ..BoxSpec::new(None, (200.0, 100.0), None)
        });
        boxes.push(BoxSpec {
            position: u64::from(later_opacity),
            opacity: if later_opacity { 1.0 } else { 0.99 },
            ..BoxSpec::new(Some(wrapper), (200.0, 60.0), Some(RED))
        });
        boxes.push(BoxSpec {
            margin: (0.0, -20.0),
            opacity: if later_opacity { 0.99 } else { 1.0 },
            ..BoxSpec::new(Some(wrapper), (200.0, 60.0), Some(BLUE))
        });
    }
    check("order_opacity_context", &boxes);
}

#[test]
fn css_order_transform_context_matches_chrome() {
    check(
        "order_transform_context",
        &[
            BoxSpec {
                position: 1,
                ..BoxSpec::new(None, (200.0, 100.0), Some(RED))
            },
            BoxSpec {
                margin: (0.0, -30.0),
                keyed_rotation: true,
                ..BoxSpec::new(None, (200.0, 100.0), Some(BLUE))
            },
        ],
    );
}
