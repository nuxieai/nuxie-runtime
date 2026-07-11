//! Host-side GPU contracts translated from Rive's `renderer/gpu.hpp`.
#![allow(dead_code)]

use bytemuck::{Pod, Zeroable};
use nuxie_render_api::{BlendMode, ColorInt, Mat2D};

pub(crate) const PARAMETRIC_PRECISION: u32 = 4;
pub(crate) const POLAR_PRECISION: u32 = 8;
pub(crate) const MAX_PARAMETRIC_SEGMENTS: u32 = 1023;
pub(crate) const MAX_POLAR_SEGMENTS: u32 = 1023;
pub(crate) const MIP_MAP_LOD_BIAS: f32 = -0.5;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct GradientSpan {
    pub horizontal_span: u32,
    pub y_with_flags: u32,
    pub color0: u32,
    pub color1: u32,
}

impl GradientSpan {
    pub(crate) fn new(
        x0_fixed: u32,
        x1_fixed: u32,
        y: u32,
        flags: u32,
        color0: ColorInt,
        color1: ColorInt,
    ) -> Self {
        assert!(x0_fixed < 65_536 && x1_fixed < 65_536);
        Self {
            horizontal_span: x1_fixed << 16 | x0_fixed,
            y_with_flags: flags | y,
            color0,
            color1,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct TessVertexSpan {
    pub points: [[f32; 2]; 4],
    pub join_tangent: [f32; 2],
    pub y: f32,
    pub reflection_y: f32,
    pub x0_x1: i32,
    pub reflection_x0_x1: i32,
    pub segment_counts: u32,
    pub contour_id_with_flags: u32,
}

impl TessVertexSpan {
    pub(crate) fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 16,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 32,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32x4,
                    offset: 48,
                    shader_location: 3,
                },
            ],
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        points: [[f32; 2]; 4],
        join_tangent: [f32; 2],
        y: f32,
        x0: i32,
        x1: i32,
        reflection_y: f32,
        reflection_x0: i32,
        reflection_x1: i32,
        parametric_segments: u32,
        polar_segments: u32,
        join_segments: u32,
        contour_id_with_flags: u32,
    ) -> Self {
        assert!((-32_768..=32_767).contains(&x0));
        assert!((-32_768..=32_767).contains(&x1));
        assert!((-32_768..=32_767).contains(&reflection_x0));
        assert!((-32_768..=32_767).contains(&reflection_x1));
        assert!(parametric_segments <= MAX_PARAMETRIC_SEGMENTS);
        assert!(polar_segments <= MAX_POLAR_SEGMENTS);
        assert!(join_segments < 1 << 12);
        Self {
            points,
            join_tangent,
            y,
            reflection_y,
            x0_x1: pack_i16_pair(x0, x1),
            reflection_x0_x1: pack_i16_pair(reflection_x0, reflection_x1),
            segment_counts: join_segments << 20 | polar_segments << 10 | parametric_segments,
            contour_id_with_flags,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn without_reflection(
        points: [[f32; 2]; 4],
        join_tangent: [f32; 2],
        y: f32,
        x0: i32,
        x1: i32,
        parametric_segments: u32,
        polar_segments: u32,
        join_segments: u32,
        contour_id_with_flags: u32,
    ) -> Self {
        Self::new(
            points,
            join_tangent,
            y,
            x0,
            x1,
            f32::NAN,
            -1,
            -1,
            parametric_segments,
            polar_segments,
            join_segments,
            contour_id_with_flags,
        )
    }
}

const fn pack_i16_pair(low: i32, high: i32) -> i32 {
    ((high as u32) << 16 | (low as u16 as u32)) as i32
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PaintType {
    ClipUpdate = 0,
    SolidColor = 1,
    LinearGradient = 2,
    RadialGradient = 3,
    Image = 4,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct PatchVertex {
    pub local_vertex_id: f32,
    pub outset: f32,
    pub fill_coverage: f32,
    pub params: i32,
    pub mirrored_vertex_id: f32,
    pub mirrored_outset: f32,
    pub mirrored_fill_coverage: f32,
    pub padding: i32,
}

impl PatchVertex {
    pub(crate) fn new(local_vertex_id: f32, outset: f32, fill_coverage: f32, params: i32) -> Self {
        Self {
            local_vertex_id,
            outset,
            fill_coverage,
            params,
            mirrored_vertex_id: local_vertex_id,
            mirrored_outset: outset,
            mirrored_fill_coverage: fill_coverage,
            padding: 0,
        }
    }

    pub(crate) fn set_mirrored_position(
        &mut self,
        local_vertex_id: f32,
        outset: f32,
        fill_coverage: f32,
    ) {
        self.mirrored_vertex_id = local_vertex_id;
        self.mirrored_outset = outset;
        self.mirrored_fill_coverage = fill_coverage;
    }
}

pub(crate) const MIDPOINT_FAN_PATCH_SEGMENT_SPAN: usize = 8;
pub(crate) const OUTER_CURVE_PATCH_SEGMENT_SPAN: usize = 17;
pub(crate) const MIDPOINT_FAN_PATCH_VERTEX_COUNT: usize = 42;
pub(crate) const MIDPOINT_FAN_PATCH_INDEX_COUNT: usize = 72;
pub(crate) const MIDPOINT_FAN_CENTER_AA_PATCH_VERTEX_COUNT: usize = 74;
pub(crate) const MIDPOINT_FAN_CENTER_AA_PATCH_INDEX_COUNT: usize = 120;
pub(crate) const OUTER_CURVE_PATCH_VERTEX_COUNT: usize = 153;
pub(crate) const OUTER_CURVE_PATCH_INDEX_COUNT: usize = 249;
pub(crate) const PATCH_VERTEX_BUFFER_COUNT: usize = 269;
pub(crate) const PATCH_INDEX_BUFFER_COUNT: usize = 441;
pub(crate) const CONTOUR_ID_MASK: u32 = 0xffff;

#[derive(Clone, Copy, PartialEq, Eq)]
enum PatchType {
    MidpointFan,
    MidpointFanCenterAa,
    OuterCurves,
}

pub(crate) fn generate_patch_buffer_data() -> (Vec<PatchVertex>, Vec<u16>) {
    let mut vertices = Vec::with_capacity(PATCH_VERTEX_BUFFER_COUNT);
    let mut indices = Vec::with_capacity(PATCH_INDEX_BUFFER_COUNT);
    generate_patch(PatchType::MidpointFan, &mut vertices, &mut indices);
    generate_patch(PatchType::MidpointFanCenterAa, &mut vertices, &mut indices);
    generate_patch(PatchType::OuterCurves, &mut vertices, &mut indices);
    debug_assert_eq!(vertices.len(), PATCH_VERTEX_BUFFER_COUNT);
    debug_assert_eq!(indices.len(), PATCH_INDEX_BUFFER_COUNT);
    (vertices, indices)
}

fn generate_patch(patch_type: PatchType, vertices: &mut Vec<PatchVertex>, indices: &mut Vec<u16>) {
    const STROKE_VERTEX: i32 = 0;
    const FAN_VERTEX: i32 = 1;
    const FAN_MIDPOINT_VERTEX: i32 = 2;
    let base_vertex = vertices.len() as u16;
    let base_index = indices.len();
    let segment_span = if patch_type == PatchType::OuterCurves {
        OUTER_CURVE_PATCH_SEGMENT_SPAN
    } else {
        MIDPOINT_FAN_PATCH_SEGMENT_SPAN
    };
    let params = |vertex_type| ((segment_span as i32) << 2) | vertex_type;

    for segment in 0..segment_span {
        let left = segment as f32;
        let right = left + 1.0;
        let start = vertices.len();
        match patch_type {
            PatchType::OuterCurves | PatchType::MidpointFanCenterAa => {
                vertices.extend([
                    PatchVertex::new(left, 0.0, 0.5, params(STROKE_VERTEX)),
                    PatchVertex::new(left, 1.0, 0.0, params(STROKE_VERTEX)),
                    PatchVertex::new(right, 0.0, 0.5, params(STROKE_VERTEX)),
                    PatchVertex::new(right, 1.0, 0.0, params(STROKE_VERTEX)),
                ]);
                let shift = if patch_type == PatchType::OuterCurves {
                    0.0
                } else {
                    -1.0
                };
                vertices[start].set_mirrored_position(right + shift, 0.0, 0.5);
                vertices[start + 1].set_mirrored_position(left + shift, 0.0, 0.5);
                vertices[start + 2].set_mirrored_position(right + shift, 1.0, 0.0);
                vertices[start + 3].set_mirrored_position(left + shift, 1.0, 0.0);
            }
            PatchType::MidpointFan => {
                vertices.extend([
                    PatchVertex::new(left, -1.0, 1.0, params(STROKE_VERTEX)),
                    PatchVertex::new(left, 1.0, 0.0, params(STROKE_VERTEX)),
                    PatchVertex::new(right, -1.0, 1.0, params(STROKE_VERTEX)),
                    PatchVertex::new(right, 1.0, 0.0, params(STROKE_VERTEX)),
                ]);
                vertices[start].set_mirrored_position(right - 1.0, -1.0, 1.0);
                vertices[start + 1].set_mirrored_position(left - 1.0, -1.0, 1.0);
                vertices[start + 2].set_mirrored_position(right - 1.0, 1.0, 0.0);
                vertices[start + 3].set_mirrored_position(left - 1.0, 1.0, 0.0);
            }
        }
    }

    if patch_type != PatchType::MidpointFan {
        for segment in 0..segment_span {
            let left = segment as f32;
            let right = left + 1.0;
            let start = vertices.len();
            vertices.extend([
                PatchVertex::new(left, -0.0, 0.5, params(STROKE_VERTEX)),
                PatchVertex::new(right, -0.0, 0.5, params(STROKE_VERTEX)),
                PatchVertex::new(left, -1.0, 0.0, params(STROKE_VERTEX)),
                PatchVertex::new(right, -1.0, 0.0, params(STROKE_VERTEX)),
            ]);
            let shift = if patch_type == PatchType::OuterCurves {
                0.0
            } else {
                -1.0
            };
            vertices[start].set_mirrored_position(right + shift, -0.0, 0.5);
            vertices[start + 1].set_mirrored_position(right + shift, -1.0, 0.0);
            vertices[start + 2].set_mirrored_position(left + shift, -0.0, 0.5);
            vertices[start + 3].set_mirrored_position(left + shift, -1.0, 0.0);
        }
    }

    let fan_vertices = vertices.len() as u16;
    let fan_segment_span = if patch_type == PatchType::OuterCurves {
        segment_span - 1
    } else {
        segment_span
    };
    for segment in 0..=fan_segment_span {
        let local_id = segment as f32;
        let outset = if patch_type == PatchType::MidpointFan {
            -1.0
        } else {
            0.0
        };
        let mut vertex = PatchVertex::new(local_id, outset, 1.0, params(FAN_VERTEX));
        if patch_type != PatchType::OuterCurves {
            vertex.set_mirrored_position(local_id - 1.0, outset, 1.0);
        }
        vertices.push(vertex);
    }
    let midpoint = vertices.len() as u16;
    if patch_type != PatchType::OuterCurves {
        vertices.push(PatchVertex::new(0.0, 0.0, 1.0, params(FAN_MIDPOINT_VERTEX)));
    }

    const BORDER: [u16; 6] = [0, 1, 2, 2, 1, 3];
    const NEGATIVE_BORDER: [u16; 6] = [0, 2, 1, 1, 2, 3];
    let mut edge_vertex = base_vertex;
    for _ in 0..segment_span {
        indices.extend(BORDER.map(|index| edge_vertex + index));
        edge_vertex += 4;
    }
    if patch_type != PatchType::MidpointFan {
        for _ in 0..segment_span {
            indices.extend(NEGATIVE_BORDER.map(|index| edge_vertex + index));
            edge_vertex += 4;
        }
    }
    debug_assert_eq!(edge_vertex, fan_vertices);

    let mut step = 1;
    while step < fan_segment_span {
        for segment in (0..fan_segment_span).step_by(step * 2) {
            indices.extend([
                fan_vertices + segment as u16,
                fan_vertices + (segment + step) as u16,
                fan_vertices + (segment + step * 2) as u16,
            ]);
        }
        step *= 2;
    }
    if patch_type != PatchType::OuterCurves {
        indices.extend([
            fan_vertices,
            fan_vertices + fan_segment_span as u16,
            midpoint,
        ]);
    }

    let expected = match patch_type {
        PatchType::MidpointFan => (
            MIDPOINT_FAN_PATCH_VERTEX_COUNT,
            MIDPOINT_FAN_PATCH_INDEX_COUNT,
        ),
        PatchType::MidpointFanCenterAa => (
            MIDPOINT_FAN_CENTER_AA_PATCH_VERTEX_COUNT,
            MIDPOINT_FAN_CENTER_AA_PATCH_INDEX_COUNT,
        ),
        PatchType::OuterCurves => (
            OUTER_CURVE_PATCH_VERTEX_COUNT,
            OUTER_CURVE_PATCH_INDEX_COUNT,
        ),
    };
    debug_assert_eq!(vertices.len() - base_vertex as usize, expected.0);
    debug_assert_eq!(indices.len() - base_index, expected.1);
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DrawType {
    MidpointFanPatches = 0,
    MidpointFanCenterAaPatches,
    OuterCurvePatches,
    InteriorTriangulation,
    AtlasBlit,
    ImageRect,
    ImageMesh,
    MsaaStrokes,
    MsaaMidpointFanBorrowedCoverage,
    MsaaMidpointFans,
    MsaaMidpointFanStencilReset,
    MsaaMidpointFanPathsStencil,
    MsaaMidpointFanPathsCover,
    MsaaOuterCubics,
    ClipReset,
    RenderPassInitialize,
    RenderPassResolve,
}

impl DrawType {
    pub(crate) const fn is_image_draw(self) -> bool {
        matches!(self, Self::ImageRect | Self::ImageMesh)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct FlushUniforms {
    pub inverse_viewports: [f32; 4],
    pub render_target_width: u32,
    pub render_target_height: u32,
    pub color_clear_value: u32,
    pub coverage_clear_value: u32,
    pub render_target_update_bounds: [i32; 4],
    pub atlas_texture_inverse_size: [f32; 2],
    pub atlas_content_inverse_viewport: [f32; 2],
    pub coverage_buffer_prefix: u32,
    pub epsilon_for_pseudo_memory_barrier: f32,
    pub path_id_granularity: u32,
    pub vertex_discard_value: f32,
    pub mip_map_lod_bias: f32,
    pub max_path_id: u32,
    pub dither_scale: f32,
    pub dither_bias: f32,
    pub dither_conversion_to_rgb10: f32,
    pub wireframe_enabled: u32,
    pub padding: [u8; 152],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct AtlasTransform {
    pub scale_factor: f32,
    pub translate_x: f32,
    pub translate_y: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct CoverageBufferRange {
    pub offset: u32,
    pub pitch: u32,
    pub offset_x: f32,
    pub offset_y: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct PathData {
    pub matrix: [f32; 6],
    pub stroke_radius: f32,
    pub feather_radius: f32,
    pub z_index: u32,
    pub atlas_transform: AtlasTransform,
    pub coverage_buffer_range: CoverageBufferRange,
}

impl PathData {
    pub(crate) fn new(
        matrix: Mat2D,
        stroke_radius: f32,
        feather_radius: f32,
        z_index: u32,
        atlas_transform: AtlasTransform,
        coverage_buffer_range: CoverageBufferRange,
    ) -> Self {
        Self {
            matrix: matrix.0,
            stroke_radius,
            feather_radius,
            z_index,
            atlas_transform,
            coverage_buffer_range,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct PaintData {
    pub params: u32,
    pub value: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct PaintAuxData {
    pub matrix: [f32; 6],
    pub paint_value: [f32; 2],
    pub clip_rect_inverse_matrix: [f32; 6],
    pub inverse_fwidth: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct ContourData {
    pub midpoint: [f32; 2],
    pub path_id: u32,
    pub vertex_index0: u32,
}

impl ContourData {
    pub(crate) const fn new(midpoint: [f32; 2], path_id: u32, vertex_index0: u32) -> Self {
        Self {
            midpoint,
            path_id,
            vertex_index0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct TriangleVertex {
    pub point: [f32; 2],
    pub weight_path_id: i32,
}

impl TriangleVertex {
    pub(crate) const fn new(point: [f32; 2], weight: i16, path_id: u16) -> Self {
        Self {
            point,
            weight_path_id: ((weight as i32) << 16) | path_id as i32,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct ImageDrawUniforms {
    pub matrix: [f32; 6],
    pub opacity: f32,
    pub padding0: f32,
    pub clip_rect_inverse_matrix: [f32; 6],
    pub clip_id: u32,
    pub blend_mode: u32,
    pub z_index: u32,
    pub padding: [u8; 188],
}

pub(crate) const fn swizzle_rive_color_to_rgba(color: ColorInt) -> u32 {
    (color & 0xff00_ff00) | (color.rotate_left(16) & 0x00ff_00ff)
}

pub(crate) fn swizzle_rive_color_to_rgba_premul(color: ColorInt) -> u32 {
    let [alpha, red, green, blue] = color.to_be_bytes();
    let premul = |channel: u8| u32::from(channel) * u32::from(alpha) / 255;
    premul(red) | premul(green) << 8 | premul(blue) << 16 | u32::from(alpha) << 24
}

pub(crate) const fn blend_mode_id(mode: BlendMode) -> u32 {
    match mode {
        BlendMode::SrcOver => 0,
        BlendMode::Screen => 1,
        BlendMode::Overlay => 2,
        BlendMode::Darken => 3,
        BlendMode::Lighten => 4,
        BlendMode::ColorDodge => 5,
        BlendMode::ColorBurn => 6,
        BlendMode::HardLight => 7,
        BlendMode::SoftLight => 8,
        BlendMode::Difference => 9,
        BlendMode::Exclusion => 10,
        BlendMode::Multiply => 11,
        BlendMode::Hue => 12,
        BlendMode::Saturation => 13,
        BlendMode::Color => 14,
        BlendMode::Luminosity => 15,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, offset_of, size_of};

    #[test]
    fn gpu_upload_records_match_cpp_abi() {
        assert_eq!(size_of::<GradientSpan>(), 16);
        assert_eq!(size_of::<TessVertexSpan>(), 64);
        assert_eq!(size_of::<PatchVertex>(), 32);
        assert_eq!(size_of::<FlushUniforms>(), 256);
        assert_eq!(size_of::<AtlasTransform>(), 12);
        assert_eq!(size_of::<CoverageBufferRange>(), 16);
        assert_eq!(size_of::<PathData>(), 64);
        assert_eq!(size_of::<PaintData>(), 8);
        assert_eq!(size_of::<PaintAuxData>(), 64);
        assert_eq!(size_of::<ContourData>(), 16);
        assert_eq!(size_of::<TriangleVertex>(), 12);
        assert_eq!(size_of::<ImageDrawUniforms>(), 256);
        assert_eq!(size_of::<PaintType>(), 4);
        assert_eq!(size_of::<DrawType>(), 1);
        assert_eq!(align_of::<FlushUniforms>(), 4);

        assert_eq!(offset_of!(PathData, atlas_transform), 36);
        assert_eq!(offset_of!(PathData, coverage_buffer_range), 48);
        assert_eq!(offset_of!(PaintAuxData, clip_rect_inverse_matrix), 32);
        assert_eq!(offset_of!(ImageDrawUniforms, matrix) % 16, 0);
        assert_eq!(
            offset_of!(ImageDrawUniforms, clip_rect_inverse_matrix) % 16,
            0
        );
    }

    #[test]
    fn tessellation_span_packs_cpp_bitfields() {
        let span = TessVertexSpan::new(
            [[0.0; 2]; 4],
            [0.0; 2],
            3.0,
            -2,
            7,
            4.0,
            -3,
            8,
            9,
            10,
            11,
            0x8000_0012,
        );
        assert_eq!(span.x0_x1 as u32, 0x0007_fffe);
        assert_eq!(span.reflection_x0_x1 as u32, 0x0008_fffd);
        assert_eq!(span.segment_counts, 11 << 20 | 10 << 10 | 9);
        assert_eq!(span.contour_id_with_flags, 0x8000_0012);
    }

    #[test]
    fn color_and_blend_helpers_match_shader_encoding() {
        assert_eq!(swizzle_rive_color_to_rgba(0x8040_2010), 0x8010_2040);
        assert_eq!(swizzle_rive_color_to_rgba_premul(0x8040_2010), 0x8008_1020);
        assert_eq!(blend_mode_id(BlendMode::SrcOver), 0);
        assert_eq!(blend_mode_id(BlendMode::Luminosity), 15);
    }

    #[test]
    fn constructors_match_cpp_record_encoding() {
        let gradient = GradientSpan::new(2, 5, 7, 0x4000_0000, 11, 13);
        assert_eq!(gradient.horizontal_span, 0x0005_0002);
        assert_eq!(gradient.y_with_flags, 0x4000_0007);

        let triangle = TriangleVertex::new([1.0, 2.0], -2, 7);
        assert_eq!(triangle.weight_path_id as u32, 0xfffe_0007);

        let mut patch = PatchVertex::new(1.0, 2.0, 0.5, 12);
        patch.set_mirrored_position(0.0, -2.0, -0.5);
        assert_eq!(patch.mirrored_vertex_id, 0.0);
        assert_eq!(patch.mirrored_outset, -2.0);
        assert_eq!(patch.mirrored_fill_coverage, -0.5);
    }

    #[test]
    fn patch_buffers_match_cpp_counts_and_topology() {
        let (vertices, indices) = generate_patch_buffer_data();
        assert_eq!(vertices.len(), PATCH_VERTEX_BUFFER_COUNT);
        assert_eq!(indices.len(), PATCH_INDEX_BUFFER_COUNT);
        assert_eq!(&indices[..6], &[0, 1, 2, 2, 1, 3]);
        assert_eq!(
            indices[MIDPOINT_FAN_PATCH_INDEX_COUNT - 3..MIDPOINT_FAN_PATCH_INDEX_COUNT],
            [32, 40, 41]
        );
        assert_eq!(
            indices[MIDPOINT_FAN_PATCH_INDEX_COUNT..][..6],
            [42, 43, 44, 44, 43, 45]
        );
        assert!(indices.iter().all(|index| *index < vertices.len() as u16));
    }
}
