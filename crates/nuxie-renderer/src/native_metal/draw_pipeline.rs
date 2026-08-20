//! Native Metal draw-pipeline realization.
//!
//! This is a mechanical Rust translation of the pinned upstream implementation
//! in `renderer/src/metal/render_context_metal_impl.mm`, specifically
//! `make_pipeline_state` (lines 45-57) and `DrawPipeline` (lines 263-401), at
//! source SHA `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
//!
//! The two-value interlock enum is intentionally local while the platform
//! neutral renderer seam is being translated. The upstream Metal code only
//! constructs raster-ordering and atomic pipelines here; the other upstream
//! interlock modes remain unreachable at this seam.

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_foundation::NSString;
use objc2_metal::{
    MTLBlendFactor, MTLBlendOperation, MTLColorWriteMask, MTLDevice, MTLLibrary, MTLPixelFormat,
    MTLRenderPipelineDescriptor, MTLRenderPipelineState,
};
use std::error::Error;
use std::fmt;

use super::super::gpu::DrawType;
use super::pipeline_names::FIXED_FUNCTION_COLOR_OUTPUT;

/// The Metal interlock modes realized by the translated draw pipeline.
///
/// This temporary local type mirrors the two cases handled by the upstream
/// `DrawPipeline` constructor. It deliberately does not import a shared
/// interlock enum whose values may include modes that Metal does not realize.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MetalInterlockMode {
    RasterOrdering,
    Atomics,
}

/// A typed failure from Metal function lookup or pipeline realization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DrawPipelineError {
    /// The shader compilation/library stage failed before pipeline creation.
    MissingLibrary,
    /// The requested vertex entry point was not present in the library.
    MissingVertexFunction(String),
    /// The requested fragment entry point was not present in the library.
    MissingFragmentFunction(String),
    /// Metal rejected one of the two format-specialized descriptors.
    PipelineCreation {
        pixel_format: MTLPixelFormat,
        description: String,
    },
    /// A draw was attempted after pipeline creation failed or was incomplete.
    PipelineUnavailable,
    /// The caller selected a format outside the upstream routing table.
    UnsupportedPixelFormat(MTLPixelFormat),
}

impl fmt::Display for DrawPipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLibrary => f.write_str("Metal draw pipeline library is unavailable"),
            Self::MissingVertexFunction(name) => {
                write!(f, "Metal draw vertex function is unavailable: {name}")
            }
            Self::MissingFragmentFunction(name) => {
                write!(f, "Metal draw fragment function is unavailable: {name}")
            }
            Self::PipelineCreation {
                pixel_format,
                description,
            } => write!(
                f,
                "Metal draw pipeline creation failed for {pixel_format:?}: {description}"
            ),
            Self::PipelineUnavailable => f.write_str("Metal draw pipeline is unavailable"),
            Self::UnsupportedPixelFormat(pixel_format) => {
                write!(f, "unsupported Metal draw pixel format: {pixel_format:?}")
            }
        }
    }
}

impl Error for DrawPipelineError {}

/// The format-specialized state selected by `pipeline_state`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PipelineSlot {
    Rgba8,
    Bgra8,
}

/// Fixed-function color policy from the upstream atomic-mode switch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BlendPolicy {
    /// Leave Metal's descriptor defaults untouched (raster-ordering mode).
    Default,
    /// The shader expects src-over blending in the framebuffer attachment.
    SrcOver,
    /// Store no blended color in the framebuffer attachment.
    Disabled,
}

/// Explicitness of the upstream color-write-mask assignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WriteMaskPolicy {
    Default,
    All,
    None,
}

/// Pure attachment policy used by both descriptor realization and tests.
///
/// `framebuffer` is the logical color plane (`COLOR_PLANE_IDX` in upstream).
/// The remaining planes are `None` in atomic mode because atomics access them
/// through device buffers rather than color attachments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AttachmentPolicy {
    framebuffer: MTLPixelFormat,
    clip_plane: Option<MTLPixelFormat>,
    scratch_color_plane: Option<MTLPixelFormat>,
    coverage_plane: Option<MTLPixelFormat>,
    blend: BlendPolicy,
    write_mask: WriteMaskPolicy,
}

/// Return the exact attachment and fixed-function policy selected upstream.
///
/// Source: `renderer/src/metal/render_context_metal_impl.mm:300-359`, pinned
/// at `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
fn attachment_policy(
    pixel_format: MTLPixelFormat,
    draw_type: DrawType,
    interlock_mode: MetalInterlockMode,
    shader_misc_flags: u32,
) -> AttachmentPolicy {
    match interlock_mode {
        MetalInterlockMode::RasterOrdering => AttachmentPolicy {
            framebuffer: pixel_format,
            // In rasterOrdering mode, the PLS planes are accessed as color
            // attachments.
            clip_plane: Some(MTLPixelFormat::R32Uint),
            scratch_color_plane: Some(pixel_format),
            coverage_plane: Some(MTLPixelFormat::R32Uint),
            blend: BlendPolicy::Default,
            write_mask: WriteMaskPolicy::Default,
        },
        MetalInterlockMode::Atomics => {
            // In atomic mode, the PLS planes are accessed as device buffers;
            // only the framebuffer attachment is configured above.
            if shader_misc_flags & FIXED_FUNCTION_COLOR_OUTPUT != 0 {
                AttachmentPolicy {
                    framebuffer: pixel_format,
                    clip_plane: None,
                    scratch_color_plane: None,
                    coverage_plane: None,
                    blend: BlendPolicy::SrcOver,
                    write_mask: WriteMaskPolicy::All,
                }
            } else if draw_type == DrawType::RenderPassResolve {
                AttachmentPolicy {
                    framebuffer: pixel_format,
                    clip_plane: None,
                    scratch_color_plane: None,
                    coverage_plane: None,
                    blend: BlendPolicy::Disabled,
                    write_mask: WriteMaskPolicy::All,
                }
            } else {
                AttachmentPolicy {
                    framebuffer: pixel_format,
                    clip_plane: None,
                    scratch_color_plane: None,
                    coverage_plane: None,
                    blend: BlendPolicy::Disabled,
                    write_mask: WriteMaskPolicy::None,
                }
            }
        }
    }
}

/// Map a render-target format to the upstream RGBA8/BGRA8 pipeline pair.
///
/// Source: `renderer/src/metal/render_context_metal_impl.mm:378-395`, pinned
/// at `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
fn pipeline_slot_for_pixel_format(
    pixel_format: MTLPixelFormat,
) -> Result<PipelineSlot, DrawPipelineError> {
    match pixel_format {
        MTLPixelFormat::RGBA8Unorm
        | MTLPixelFormat::RGBA8Unorm_sRGB
        | MTLPixelFormat::RGBA16Float => Ok(PipelineSlot::Rgba8),
        MTLPixelFormat::BGRA8Unorm | MTLPixelFormat::BGRA8Unorm_sRGB => Ok(PipelineSlot::Bgra8),
        _ => Err(DrawPipelineError::UnsupportedPixelFormat(pixel_format)),
    }
}

/// Compile one descriptor, preserving the upstream helper's error boundary.
///
/// Source: `renderer/src/metal/render_context_metal_impl.mm:45-57`, pinned at
/// `4ac7b32798da0482e441ef09304dc3b480ed3ee5`. Objective-C's nil return is
/// represented as a typed `Err` so a failed state cannot be used for drawing.
fn make_pipeline_state(
    gpu: &ProtocolObject<dyn MTLDevice>,
    descriptor: &MTLRenderPipelineDescriptor,
    pixel_format: MTLPixelFormat,
) -> Result<Retained<ProtocolObject<dyn MTLRenderPipelineState>>, DrawPipelineError> {
    gpu.newRenderPipelineStateWithDescriptor_error(descriptor)
        .map_err(|error| DrawPipelineError::PipelineCreation {
            pixel_format,
            description: error.localizedDescription().to_string(),
        })
}

fn apply_attachment_policy(descriptor: &MTLRenderPipelineDescriptor, policy: AttachmentPolicy) {
    let attachments = descriptor.colorAttachments();
    // SAFETY: `MTLRenderPipelineDescriptor.colorAttachments` owns eight
    // non-null descriptors. Index 0 is upstream's `COLOR_PLANE_IDX`, and
    // objc2 returns a retained descriptor that remains owned for this scope.
    let framebuffer = unsafe { attachments.objectAtIndexedSubscript(0) };
    framebuffer.setPixelFormat(policy.framebuffer);

    if let Some(pixel_format) = policy.clip_plane {
        // SAFETY: Index 1 is upstream's `CLIP_PLANE_IDX`, within Metal's
        // eight-entry attachment array. objc2 returns a retained, non-null
        // descriptor, and `pixel_format` is a typed `MTLPixelFormat` value.
        let clip_plane = unsafe { attachments.objectAtIndexedSubscript(1) };
        clip_plane.setPixelFormat(pixel_format);
    }
    if let Some(pixel_format) = policy.scratch_color_plane {
        // SAFETY: Index 2 is upstream's `SCRATCH_COLOR_PLANE_IDX`, within
        // Metal's eight-entry attachment array. objc2 retains the non-null
        // descriptor for this scope; the pixel-format enum is passed by value.
        let scratch_color_plane = unsafe { attachments.objectAtIndexedSubscript(2) };
        scratch_color_plane.setPixelFormat(pixel_format);
    }
    if let Some(pixel_format) = policy.coverage_plane {
        // SAFETY: Index 3 is upstream's `COVERAGE_PLANE_IDX`, within Metal's
        // eight-entry attachment array. objc2 retains the non-null descriptor
        // for this scope; the pixel-format enum is passed by value.
        let coverage_plane = unsafe { attachments.objectAtIndexedSubscript(3) };
        coverage_plane.setPixelFormat(pixel_format);
    }

    match policy.blend {
        BlendPolicy::Default => {}
        BlendPolicy::SrcOver => {
            framebuffer.setBlendingEnabled(true);
            framebuffer.setSourceRGBBlendFactor(MTLBlendFactor::One);
            framebuffer.setDestinationRGBBlendFactor(MTLBlendFactor::OneMinusSourceAlpha);
            framebuffer.setRgbBlendOperation(MTLBlendOperation::Add);
            framebuffer.setSourceAlphaBlendFactor(MTLBlendFactor::One);
            framebuffer.setDestinationAlphaBlendFactor(MTLBlendFactor::OneMinusSourceAlpha);
            framebuffer.setAlphaBlendOperation(MTLBlendOperation::Add);
        }
        BlendPolicy::Disabled => framebuffer.setBlendingEnabled(false),
    }

    match policy.write_mask {
        WriteMaskPolicy::Default => {}
        WriteMaskPolicy::All => framebuffer.setWriteMask(MTLColorWriteMask::All),
        WriteMaskPolicy::None => framebuffer.setWriteMask(MTLColorWriteMask::None),
    }
}

/// A pair of Metal draw states specialized for RGBA and BGRA framebuffer data.
#[derive(Clone)]
pub(crate) struct DrawPipeline {
    pipeline_state_rgba8: Option<Retained<ProtocolObject<dyn MTLRenderPipelineState>>>,
    pipeline_state_bgra8: Option<Retained<ProtocolObject<dyn MTLRenderPipelineState>>>,
}

impl DrawPipeline {
    /// Create the pair of format-specialized draw states.
    ///
    /// A missing library/function or either failed state returns an error and
    /// drops the other state, preserving the upstream fail-closed nil-pair
    /// behavior without allowing a partial pair to escape.
    pub(crate) fn new(
        gpu: &ProtocolObject<dyn MTLDevice>,
        library: Option<&ProtocolObject<dyn MTLLibrary>>,
        vertex_function_name: &NSString,
        fragment_function_name: &NSString,
        draw_type: DrawType,
        interlock_mode: MetalInterlockMode,
        shader_misc_flags: u32,
    ) -> Result<Self, DrawPipelineError> {
        let Some(library) = library else {
            return Err(DrawPipelineError::MissingLibrary);
        };

        let vertex_main = library
            .newFunctionWithName(vertex_function_name)
            .ok_or_else(|| {
                DrawPipelineError::MissingVertexFunction(vertex_function_name.to_string())
            })?;
        let fragment_main = library
            .newFunctionWithName(fragment_function_name)
            .ok_or_else(|| {
                DrawPipelineError::MissingFragmentFunction(fragment_function_name.to_string())
            })?;

        let make_state = |pixel_format: MTLPixelFormat| {
            let descriptor = MTLRenderPipelineDescriptor::new();
            descriptor.setVertexFunction(Some(&vertex_main));
            descriptor.setFragmentFunction(Some(&fragment_main));
            apply_attachment_policy(
                &descriptor,
                attachment_policy(pixel_format, draw_type, interlock_mode, shader_misc_flags),
            );
            make_pipeline_state(gpu, &descriptor, pixel_format)
        };

        // Keep the source order: RGBA first, then BGRA. If either fails, the
        // `?` drops the first state and no incomplete pair is returned.
        let pipeline_state_rgba8 = make_state(MTLPixelFormat::RGBA8Unorm)?;
        let pipeline_state_bgra8 = make_state(MTLPixelFormat::BGRA8Unorm)?;
        Ok(Self {
            pipeline_state_rgba8: Some(pipeline_state_rgba8),
            pipeline_state_bgra8: Some(pipeline_state_bgra8),
        })
    }

    /// Construct the nil-pair state used when a caller already recorded a
    /// shader-compilation failure and wants draws to remain fail-closed.
    pub(crate) const fn unavailable() -> Self {
        Self {
            pipeline_state_rgba8: None,
            pipeline_state_bgra8: None,
        }
    }

    /// Whether both format-specialized states are available.
    pub(crate) fn valid(&self) -> bool {
        self.pipeline_state_rgba8.is_some() && self.pipeline_state_bgra8.is_some()
    }

    /// Select the state matching the framebuffer's channel ordering.
    pub(crate) fn pipeline_state(
        &self,
        pixel_format: MTLPixelFormat,
    ) -> Result<&ProtocolObject<dyn MTLRenderPipelineState>, DrawPipelineError> {
        if !self.valid() {
            return Err(DrawPipelineError::PipelineUnavailable);
        }

        match pipeline_slot_for_pixel_format(pixel_format)? {
            PipelineSlot::Rgba8 => self
                .pipeline_state_rgba8
                .as_deref()
                .ok_or(DrawPipelineError::PipelineUnavailable),
            PipelineSlot::Bgra8 => self
                .pipeline_state_bgra8
                .as_deref()
                .ok_or(DrawPipelineError::PipelineUnavailable),
        }
    }

    pub(crate) fn retained_pipeline_state(
        &self,
        pixel_format: MTLPixelFormat,
    ) -> Result<Retained<ProtocolObject<dyn MTLRenderPipelineState>>, DrawPipelineError> {
        if !self.valid() {
            return Err(DrawPipelineError::PipelineUnavailable);
        }
        match pipeline_slot_for_pixel_format(pixel_format)? {
            PipelineSlot::Rgba8 => self
                .pipeline_state_rgba8
                .clone()
                .ok_or(DrawPipelineError::PipelineUnavailable),
            PipelineSlot::Bgra8 => self
                .pipeline_state_bgra8
                .clone()
                .ok_or(DrawPipelineError::PipelineUnavailable),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attachment_policy_table_matches_raster_ordering_planes() {
        let policy = attachment_policy(
            MTLPixelFormat::RGBA8Unorm,
            DrawType::MidpointFanPatches,
            MetalInterlockMode::RasterOrdering,
            0,
        );

        assert_eq!(policy.framebuffer, MTLPixelFormat::RGBA8Unorm);
        assert_eq!(policy.clip_plane, Some(MTLPixelFormat::R32Uint));
        assert_eq!(policy.scratch_color_plane, Some(MTLPixelFormat::RGBA8Unorm));
        assert_eq!(policy.coverage_plane, Some(MTLPixelFormat::R32Uint));
        assert_eq!(policy.blend, BlendPolicy::Default);
        assert_eq!(policy.write_mask, WriteMaskPolicy::Default);
    }

    #[test]
    fn attachment_policy_table_matches_atomic_fixed_function_output() {
        let policy = attachment_policy(
            MTLPixelFormat::BGRA8Unorm,
            DrawType::ImageMesh,
            MetalInterlockMode::Atomics,
            FIXED_FUNCTION_COLOR_OUTPUT,
        );

        assert_eq!(policy.framebuffer, MTLPixelFormat::BGRA8Unorm);
        assert_eq!(policy.clip_plane, None);
        assert_eq!(policy.scratch_color_plane, None);
        assert_eq!(policy.coverage_plane, None);
        assert_eq!(policy.blend, BlendPolicy::SrcOver);
        assert_eq!(policy.write_mask, WriteMaskPolicy::All);
    }

    #[test]
    fn attachment_policy_table_matches_atomic_resolve_and_offscreen_output() {
        let resolve = attachment_policy(
            MTLPixelFormat::RGBA16Float,
            DrawType::RenderPassResolve,
            MetalInterlockMode::Atomics,
            0,
        );
        assert_eq!(resolve.blend, BlendPolicy::Disabled);
        assert_eq!(resolve.write_mask, WriteMaskPolicy::All);

        let offscreen = attachment_policy(
            MTLPixelFormat::RGBA16Float,
            DrawType::InteriorTriangulation,
            MetalInterlockMode::Atomics,
            0,
        );
        assert_eq!(offscreen.blend, BlendPolicy::Disabled);
        assert_eq!(offscreen.write_mask, WriteMaskPolicy::None);
    }

    #[test]
    fn pixel_format_routing_matches_upstream_pair_selection() {
        for pixel_format in [
            MTLPixelFormat::RGBA8Unorm,
            MTLPixelFormat::RGBA8Unorm_sRGB,
            MTLPixelFormat::RGBA16Float,
        ] {
            assert_eq!(
                pipeline_slot_for_pixel_format(pixel_format),
                Ok(PipelineSlot::Rgba8)
            );
        }
        for pixel_format in [MTLPixelFormat::BGRA8Unorm, MTLPixelFormat::BGRA8Unorm_sRGB] {
            assert_eq!(
                pipeline_slot_for_pixel_format(pixel_format),
                Ok(PipelineSlot::Bgra8)
            );
        }
        assert_eq!(
            pipeline_slot_for_pixel_format(MTLPixelFormat::Invalid),
            Err(DrawPipelineError::UnsupportedPixelFormat(
                MTLPixelFormat::Invalid
            ))
        );
    }

    #[test]
    fn unavailable_pipeline_is_invalid_and_fail_closed() {
        let pipeline = DrawPipeline::unavailable();
        assert!(!pipeline.valid());
        assert_eq!(
            pipeline.pipeline_state(MTLPixelFormat::RGBA8Unorm),
            Err(DrawPipelineError::PipelineUnavailable)
        );
    }
}
