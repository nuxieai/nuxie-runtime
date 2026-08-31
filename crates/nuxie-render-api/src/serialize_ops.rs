//! `include/utils/serialize_ops.hpp` — shared SRIV wire operations.
use crate::{PathVerb, RawPath, Vec2D};
pub use nuxie_binary::BinaryDataReader as Reader;

macro_rules! operations {
    ($($constant:ident = $variant:ident = $value:literal),* $(,)?) => {
        #[allow(non_camel_case_types)]
        #[repr(u32)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum SerializeOp { $($variant = $value),* }
        $(pub(crate) const $constant: u64 = SerializeOp::$variant as u64;)*
    };
}
operations! {
    MAKE_RENDER_BUFFER = makeRenderBuffer = 0,
    MAKE_LINEAR_GRADIENT = makeLinearGradient = 1,
    MAKE_RADIAL_GRADIENT = makeRadialGradient = 2,
    MAKE_RENDER_PATH = makeRenderPath = 3,
    MAKE_RENDER_PAINT = makeRenderPaint = 5,
    DECODE_IMAGE = decodeImage = 6,
    SAVE = save = 7,
    RESTORE = restore = 8,
    TRANSFORM = transform = 9,
    DRAW_PATH = drawPath = 10,
    CLIP_PATH = clipPath = 11,
    DRAW_IMAGE = drawImage = 12,
    DRAW_IMAGE_MESH = drawImageMesh = 13,
    SET_VERTEX_BUFFER_DATA = setVertexBufferData = 14,
    SET_INDEX_BUFFER_DATA = setIndexBufferData = 15,
    ADD_RAW_PATH = addRawPath = 16,
    REWIND = rewind = 17,
    FILL_RULE = fillRule = 18,
    STYLE = style = 20,
    COLOR = color = 21,
    THICKNESS = thickness = 22,
    JOIN = join = 23,
    CAP = cap = 24,
    FEATHER = feather = 25,
    BLEND_MODE = blendMode = 26,
    SHADER = shader = 27,
    FRAME = frame = 28,
    FRAME_SIZE = frameSize = 29,
    MODULATE_OPACITY = modulateOpacity = 30,
}

pub(crate) fn serialize_raw_path(writer: &mut crate::serializing::Writer, path: &RawPath) {
    writer.varuint(path.verbs().len() as u64);
    for verb in path.verbs() {
        writer.varuint(*verb as u64);
    }
    writer.varuint(path.points().len() as u64);
    for point in path.points() {
        writer.float(point.x);
        writer.float(point.y);
    }
}

pub fn deserialize_raw_path(reader: &mut Reader<'_>) -> RawPath {
    let mut path = RawPath::new();
    let count = reader.read_var_uint() as usize;
    let verbs = (0..count)
        .map(|_| reader.read_var_uint() as u8)
        .collect::<Vec<_>>();
    let count = reader.read_var_uint() as usize;
    let points = (0..count)
        .map(|_| Vec2D::new(reader.read_float32(), reader.read_float32()))
        .collect::<Vec<_>>();
    let mut p = 0;
    for verb in verbs {
        let needed = match verb {
            value if value == PathVerb::Move as u8 || value == PathVerb::Line as u8 => 1,
            value if value == PathVerb::Quad as u8 => 2,
            value if value == PathVerb::Cubic as u8 => 3,
            _ => 0,
        };
        if p + needed > points.len() {
            return path;
        }
        match verb {
            value if value == PathVerb::Move as u8 => path.move_to(points[p].x, points[p].y),
            value if value == PathVerb::Line as u8 => path.line_to(points[p].x, points[p].y),
            value if value == PathVerb::Quad as u8 => {
                path.quad_to(points[p].x, points[p].y, points[p + 1].x, points[p + 1].y)
            }
            value if value == PathVerb::Cubic as u8 => path.cubic_to(
                points[p].x,
                points[p].y,
                points[p + 1].x,
                points[p + 1].y,
                points[p + 2].x,
                points[p + 2].y,
            ),
            value if value == PathVerb::Close as u8 => path.close(),
            _ => {}
        }
        p += needed;
    }
    path
}
