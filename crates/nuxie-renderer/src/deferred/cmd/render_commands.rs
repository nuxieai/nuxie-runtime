//! renderer/cmd/render_commands.hpp at e949498e: pointer-free wire vocabulary.
use super::command_stream::{wire_pod, WirePod};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderCmd {
    MakePath,
    MakeEmptyPath,
    MakePaint,
    MakeLinearGradient,
    MakeRadialGradient,
    DecodeImage,
    MakeBuffer,
    BufferData,
    DestroyResource,
    PathRewind,
    PathFillRule,
    PathAddRawPath,
    PathAddRenderPath,
    PaintStyle,
    PaintColor,
    PaintThickness,
    PaintJoin,
    PaintCap,
    PaintFeather,
    PaintBlendMode,
    PaintShader,
    PaintInvalidateStroke,
    Save,
    Restore,
    Transform,
    DrawPath,
    ClipPath,
    DrawImage,
    DrawImageMesh,
    ModulateOpacity,
    CanvasContentBegin,
    CanvasContentEnd,
    ResourceNewVersion,
}
impl RenderCmd {
    pub fn from_byte(value: u8) -> Option<Self> {
        const COMMANDS: [RenderCmd; 33] = [
            RenderCmd::MakePath,
            RenderCmd::MakeEmptyPath,
            RenderCmd::MakePaint,
            RenderCmd::MakeLinearGradient,
            RenderCmd::MakeRadialGradient,
            RenderCmd::DecodeImage,
            RenderCmd::MakeBuffer,
            RenderCmd::BufferData,
            RenderCmd::DestroyResource,
            RenderCmd::PathRewind,
            RenderCmd::PathFillRule,
            RenderCmd::PathAddRawPath,
            RenderCmd::PathAddRenderPath,
            RenderCmd::PaintStyle,
            RenderCmd::PaintColor,
            RenderCmd::PaintThickness,
            RenderCmd::PaintJoin,
            RenderCmd::PaintCap,
            RenderCmd::PaintFeather,
            RenderCmd::PaintBlendMode,
            RenderCmd::PaintShader,
            RenderCmd::PaintInvalidateStroke,
            RenderCmd::Save,
            RenderCmd::Restore,
            RenderCmd::Transform,
            RenderCmd::DrawPath,
            RenderCmd::ClipPath,
            RenderCmd::DrawImage,
            RenderCmd::DrawImageMesh,
            RenderCmd::ModulateOpacity,
            RenderCmd::CanvasContentBegin,
            RenderCmd::CanvasContentEnd,
            RenderCmd::ResourceNewVersion,
        ];
        COMMANDS.get(value as usize).copied()
    }
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceKind {
    Path,
    Paint,
    Shader,
    Image,
    Buffer,
}

wire_pod!(ResIdPod { id: u32 });
wire_pod!(DestroyResourcePod {
    kind: u8,
    id: u32,
    generation: u32
});
wire_pod!(ResourceVersionPod {
    kind: u8,
    id: u32,
    version: u32
});
wire_pod!(MakeIdPod {
    id: u32,
    generation: u32
});
wire_pod!(MakePathPod {
    id: u32,
    generation: u32,
    blob_offset: u64,
    points_offset: u64,
    verb_count: u32,
    point_count: u32,
    fill_rule: u32,
    pad: u32
});
wire_pod!(LinearGradientPod {
    id: u32,
    generation: u32,
    sx: f32,
    sy: f32,
    ex: f32,
    ey: f32,
    blob_offset: u64,
    stops_offset: u64,
    count: u32,
    pad: u32
});
wire_pod!(RadialGradientPod {
    id: u32,
    generation: u32,
    cx: f32,
    cy: f32,
    radius: f32,
    count: u32,
    blob_offset: u64,
    stops_offset: u64
});
wire_pod!(PathFillRulePod {
    path: u32,
    fill_rule: u8
});
wire_pod!(PathRawPod {
    blob_offset: u64,
    points_offset: u64,
    path: u32,
    verb_count: u32,
    point_count: u32,
    pad: u32
});
wire_pod!(PathAddPathPod {
    path: u32,
    src: u32,
    xx: f32,
    xy: f32,
    yx: f32,
    yy: f32,
    tx: f32,
    ty: f32
});
wire_pod!(PaintU8Pod {
    paint: u32,
    value: u8
});
wire_pod!(PaintColorPod {
    paint: u32,
    color: u32
});
wire_pod!(PaintFloatPod {
    paint: u32,
    value: f32
});
wire_pod!(PaintShaderPod {
    paint: u32,
    shader: u32
});
wire_pod!(TransformPod {
    xx: f32,
    xy: f32,
    yx: f32,
    yy: f32,
    tx: f32,
    ty: f32
});
wire_pod!(DrawPathPod {
    path: u32,
    paint: u32,
    path_version: u32,
    paint_version: u32
});
wire_pod!(ClipPathPod {
    path: u32,
    version: u32
});
wire_pod!(DecodeImagePod {
    id: u32,
    generation: u32,
    blob_offset: u64,
    byte_count: u32,
    width: u32,
    height: u32,
    pad: u32
});
wire_pod!(MakeBufferPod {
    id: u32,
    generation: u32,
    buffer_type: u8,
    flags: u8,
    size_in_bytes: u32
});
wire_pod!(BufferDataPod {
    blob_offset: u64,
    buffer: u32,
    size: u32
});
wire_pod!(DrawImagePod {
    image: u32,
    wrap_x: u8,
    wrap_y: u8,
    filter: u8,
    blend_mode: u8,
    opacity: f32
});
wire_pod!(DrawImageMeshPod {
    image: u32,
    vertices: u32,
    uv_coords: u32,
    indices: u32,
    vertex_version: u32,
    uv_version: u32,
    index_version: u32,
    vertex_count: u32,
    index_count: u32,
    wrap_x: u8,
    wrap_y: u8,
    filter: u8,
    blend_mode: u8,
    opacity: f32
});
wire_pod!(OpacityPod { opacity: f32 });
wire_pod!(CanvasContentPod {
    canvas_id: u32,
    clear_color: u32
});

pub const fn payload_size_of(command: RenderCmd) -> usize {
    match command {
        RenderCmd::MakePath => MakePathPod::SIZE,
        RenderCmd::MakeEmptyPath | RenderCmd::MakePaint => MakeIdPod::SIZE,
        RenderCmd::MakeLinearGradient => LinearGradientPod::SIZE,
        RenderCmd::MakeRadialGradient => RadialGradientPod::SIZE,
        RenderCmd::DecodeImage => DecodeImagePod::SIZE,
        RenderCmd::MakeBuffer => MakeBufferPod::SIZE,
        RenderCmd::BufferData => BufferDataPod::SIZE,
        RenderCmd::DestroyResource => DestroyResourcePod::SIZE,
        RenderCmd::PathRewind | RenderCmd::PaintInvalidateStroke | RenderCmd::CanvasContentEnd => {
            ResIdPod::SIZE
        }
        RenderCmd::PathFillRule => PathFillRulePod::SIZE,
        RenderCmd::PathAddRawPath => PathRawPod::SIZE,
        RenderCmd::PathAddRenderPath => PathAddPathPod::SIZE,
        RenderCmd::PaintStyle
        | RenderCmd::PaintJoin
        | RenderCmd::PaintCap
        | RenderCmd::PaintBlendMode => PaintU8Pod::SIZE,
        RenderCmd::PaintColor => PaintColorPod::SIZE,
        RenderCmd::PaintThickness | RenderCmd::PaintFeather => PaintFloatPod::SIZE,
        RenderCmd::PaintShader => PaintShaderPod::SIZE,
        RenderCmd::Save | RenderCmd::Restore => 0,
        RenderCmd::Transform => TransformPod::SIZE,
        RenderCmd::DrawPath => DrawPathPod::SIZE,
        RenderCmd::ClipPath => ClipPathPod::SIZE,
        RenderCmd::DrawImage => DrawImagePod::SIZE,
        RenderCmd::DrawImageMesh => DrawImageMeshPod::SIZE,
        RenderCmd::ModulateOpacity => OpacityPod::SIZE,
        RenderCmd::CanvasContentBegin => CanvasContentPod::SIZE,
        RenderCmd::ResourceNewVersion => ResourceVersionPod::SIZE,
    }
}
const _: () = assert!(
    MakePathPod::SIZE == 40
        && LinearGradientPod::SIZE == 48
        && RadialGradientPod::SIZE == 40
        && PathRawPod::SIZE == 32
        && DecodeImagePod::SIZE == 32
        && BufferDataPod::SIZE == 16
);
