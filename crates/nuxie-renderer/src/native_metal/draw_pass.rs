//! Common Metal draw-pass binding plan.
//!
//! This is the file-first translation seam for pinned
//! `RenderContextMetalImpl::makeRenderPassForDraws` at
//! `renderer/src/metal/render_context_metal_impl.mm:1298-1393`.

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTLCommandBuffer, MTLRenderCommandEncoder, MTLRenderPassDescriptor, MTLTriangleFillMode,
    MTLViewport,
};
use smallvec::SmallVec;

use super::context::{NativeMetalContext, PreparedResourceLease};
use super::render_target::RenderTargetMetal;
use super::shader_compile_plan::{InterlockMode, ShaderMiscFlags, FIXED_FUNCTION_COLOR_OUTPUT};
use crate::RendererError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DrawPassResource {
    FlushUniforms,
    TessellationTexture,
    GaussianTexture,
    GradientTexture,
    FeatherAtlasTexture,
    Paths,
    Paints,
    PaintAux,
    Contours,
    ColorAtomicPlane,
    ClipAtomicPlane,
    CoverageAtomicPlane,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DrawPassStage {
    Vertex,
    Fragment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DrawPassBinding {
    pub(crate) resource: DrawPassResource,
    pub(crate) stage: DrawPassStage,
    pub(crate) index: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct DrawPassPlanInput {
    pub(crate) interlock_mode: InterlockMode,
    pub(crate) baseline_shader_misc_flags: ShaderMiscFlags,
    pub(crate) path_count: usize,
    pub(crate) contour_count: usize,
}

pub(crate) struct DrawPassDescriptor {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) binding_plan: DrawPassPlanInput,
    pub(crate) wireframe: bool,
}

pub(crate) fn draw_pass_binding_plan(input: DrawPassPlanInput) -> SmallVec<[DrawPassBinding; 14]> {
    use DrawPassResource as Resource;
    use DrawPassStage as Stage;

    let mut bindings = SmallVec::new();
    bindings.extend([
        binding(Resource::FlushUniforms, Stage::Vertex, 3),
        binding(Resource::FlushUniforms, Stage::Fragment, 3),
        binding(Resource::TessellationTexture, Stage::Vertex, 7),
        binding(Resource::GaussianTexture, Stage::Vertex, 9),
        binding(Resource::GradientTexture, Stage::Fragment, 8),
        binding(Resource::GaussianTexture, Stage::Fragment, 9),
        binding(Resource::FeatherAtlasTexture, Stage::Fragment, 10),
    ]);
    if input.path_count > 0 {
        bindings.push(binding(Resource::Paths, Stage::Vertex, 5));
        let paint_stage = if input.interlock_mode == InterlockMode::Atomics {
            Stage::Fragment
        } else {
            Stage::Vertex
        };
        bindings.push(binding(Resource::Paints, paint_stage, 6));
        bindings.push(binding(Resource::PaintAux, paint_stage, 7));
    }
    if input.contour_count > 0 {
        bindings.push(binding(Resource::Contours, Stage::Vertex, 8));
    }
    if input.interlock_mode == InterlockMode::Atomics {
        if input.baseline_shader_misc_flags & FIXED_FUNCTION_COLOR_OUTPUT == 0 {
            bindings.push(binding(Resource::ColorAtomicPlane, Stage::Fragment, 16));
        }
        bindings.push(binding(Resource::ClipAtomicPlane, Stage::Fragment, 17));
        bindings.push(binding(Resource::CoverageAtomicPlane, Stage::Fragment, 19));
    }
    bindings
}

const fn binding(
    resource: DrawPassResource,
    stage: DrawPassStage,
    index: usize,
) -> DrawPassBinding {
    DrawPassBinding {
        resource,
        stage,
        index,
    }
}

/// Creates and completely initializes the encoder shared by draw batches.
///
/// Upload offsets are zero because each Rust ring lease is already sliced to
/// one logical flush; this is the typed-owner adaptation of upstream's
/// `firstPath`, `firstPaint`, `firstPaintAux`, and `firstContour` offsets.
pub(crate) fn make_render_pass_for_draws(
    context: &NativeMetalContext,
    command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
    pass: &MTLRenderPassDescriptor,
    target: &mut RenderTargetMetal,
    lease: &PreparedResourceLease,
    descriptor: DrawPassDescriptor,
) -> Result<Retained<ProtocolObject<dyn MTLRenderCommandEncoder>>, RendererError> {
    let bindings = draw_pass_binding_plan(descriptor.binding_plan);
    let uses_atomic_planes = bindings.iter().any(|binding| {
        matches!(
            binding.resource,
            DrawPassResource::ColorAtomicPlane
                | DrawPassResource::ClipAtomicPlane
                | DrawPassResource::CoverageAtomicPlane
        )
    });
    let uses_color_plane = bindings
        .iter()
        .any(|binding| binding.resource == DrawPassResource::ColorAtomicPlane);
    if uses_atomic_planes {
        // Rust publishes the required atomic owner set transactionally before
        // creating an encoder; this is the fail-closed adaptation of the
        // source's individually lazy target getters.
        target.prepare_atomic_planes(uses_color_plane)?;
    }
    let encoder = command_buffer
        .renderCommandEncoderWithDescriptor(pass)
        .ok_or_else(|| {
            RendererError::NativeMetal("failed to create draw-pass encoder".to_owned())
        })?;
    encoder.setViewport(MTLViewport {
        originX: 0.0,
        originY: 0.0,
        width: descriptor.width as f64,
        height: descriptor.height as f64,
        znear: 0.0,
        zfar: 1.0,
    });

    for binding in bindings {
        apply_binding(&encoder, context, target, lease, binding)?;
    }
    if descriptor.wireframe {
        encoder.setTriangleFillMode(MTLTriangleFillMode::Lines);
    }
    Ok(encoder)
}

fn apply_binding(
    encoder: &ProtocolObject<dyn MTLRenderCommandEncoder>,
    context: &NativeMetalContext,
    target: &mut RenderTargetMetal,
    lease: &PreparedResourceLease,
    binding: DrawPassBinding,
) -> Result<(), RendererError> {
    use DrawPassResource as Resource;
    use DrawPassStage as Stage;

    let buffer = match binding.resource {
        Resource::FlushUniforms => Some(lease.flush_uniforms.as_ref()),
        Resource::Paths => Some(lease.paths.as_ref()),
        Resource::Paints => Some(lease.paints.as_ref()),
        Resource::PaintAux => Some(lease.paint_aux.as_ref()),
        Resource::Contours => Some(lease.contours.as_ref()),
        Resource::ColorAtomicPlane => target.color_atomic_buffer(),
        Resource::ClipAtomicPlane => target.clip_atomic_buffer(),
        Resource::CoverageAtomicPlane => target.coverage_atomic_buffer(),
        Resource::TessellationTexture
        | Resource::GaussianTexture
        | Resource::GradientTexture
        | Resource::FeatherAtlasTexture => None,
    };
    if let Some(buffer) = buffer {
        // SAFETY: the binding plan contains only generated Metal ABI indices;
        // every buffer is retained by the lease or target through command
        // completion, and each flush-local upload begins at aligned offset 0.
        unsafe {
            match binding.stage {
                Stage::Vertex => {
                    encoder.setVertexBuffer_offset_atIndex(Some(buffer), 0, binding.index)
                }
                Stage::Fragment => {
                    encoder.setFragmentBuffer_offset_atIndex(Some(buffer), 0, binding.index)
                }
            }
        }
        return Ok(());
    }

    if matches!(
        binding.resource,
        Resource::ColorAtomicPlane | Resource::ClipAtomicPlane | Resource::CoverageAtomicPlane
    ) {
        // Objective-C `id<MTLBuffer>` owners are nullable. Preserve the
        // source's nil binding after an allocation failure; the context and
        // flush remain alive and Metal receives the selector in source order.
        unsafe {
            match binding.stage {
                Stage::Vertex => encoder.setVertexBuffer_offset_atIndex(None, 0, binding.index),
                Stage::Fragment => encoder.setFragmentBuffer_offset_atIndex(None, 0, binding.index),
            }
        }
        return Ok(());
    }

    // SAFETY: the binding plan uses the generated texture ABI indices. Metal
    // accepts nil for optional gradient/atlas textures, and the context/lease
    // retain every non-nil texture until command completion.
    unsafe {
        match (binding.resource, binding.stage) {
            (Resource::TessellationTexture, Stage::Vertex) => {
                encoder.setVertexTexture_atIndex(Some(&lease.tessellation), binding.index)
            }
            (Resource::GaussianTexture, Stage::Vertex) => {
                encoder.setVertexTexture_atIndex(context.gaussian_integral_texture(), binding.index)
            }
            (Resource::GradientTexture, Stage::Fragment) => {
                encoder.setFragmentTexture_atIndex(lease.gradient.as_deref(), binding.index)
            }
            (Resource::GaussianTexture, Stage::Fragment) => encoder
                .setFragmentTexture_atIndex(context.gaussian_integral_texture(), binding.index),
            (Resource::FeatherAtlasTexture, Stage::Fragment) => {
                encoder.setFragmentTexture_atIndex(lease.feather_atlas.as_deref(), binding.index)
            }
            _ => {
                return Err(RendererError::NativeMetal(
                    "draw-pass binding plan selected an invalid resource stage".to_owned(),
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_metal::shader_compile_plan::FIXED_FUNCTION_COLOR_OUTPUT;

    #[test]
    fn atomic_fixed_output_common_bindings_match_pinned_order_and_stages() {
        let actual = draw_pass_binding_plan(DrawPassPlanInput {
            interlock_mode: InterlockMode::Atomics,
            baseline_shader_misc_flags: FIXED_FUNCTION_COLOR_OUTPUT,
            path_count: 2,
            contour_count: 3,
        });
        assert!(!actual.spilled());
        let expected = vec![
            binding(DrawPassResource::FlushUniforms, DrawPassStage::Vertex, 3),
            binding(DrawPassResource::FlushUniforms, DrawPassStage::Fragment, 3),
            binding(
                DrawPassResource::TessellationTexture,
                DrawPassStage::Vertex,
                7,
            ),
            binding(DrawPassResource::GaussianTexture, DrawPassStage::Vertex, 9),
            binding(
                DrawPassResource::GradientTexture,
                DrawPassStage::Fragment,
                8,
            ),
            binding(
                DrawPassResource::GaussianTexture,
                DrawPassStage::Fragment,
                9,
            ),
            binding(
                DrawPassResource::FeatherAtlasTexture,
                DrawPassStage::Fragment,
                10,
            ),
            binding(DrawPassResource::Paths, DrawPassStage::Vertex, 5),
            binding(DrawPassResource::Paints, DrawPassStage::Fragment, 6),
            binding(DrawPassResource::PaintAux, DrawPassStage::Fragment, 7),
            binding(DrawPassResource::Contours, DrawPassStage::Vertex, 8),
            binding(
                DrawPassResource::ClipAtomicPlane,
                DrawPassStage::Fragment,
                17,
            ),
            binding(
                DrawPassResource::CoverageAtomicPlane,
                DrawPassStage::Fragment,
                19,
            ),
        ];
        assert_eq!(actual.as_slice(), expected.as_slice());
    }

    #[test]
    fn atomic_advanced_output_adds_color_plane_before_clip_and_coverage() {
        let actual = draw_pass_binding_plan(DrawPassPlanInput {
            interlock_mode: InterlockMode::Atomics,
            baseline_shader_misc_flags: 0,
            path_count: 1,
            contour_count: 0,
        });
        assert_eq!(
            &actual[actual.len() - 3..],
            &[
                binding(
                    DrawPassResource::ColorAtomicPlane,
                    DrawPassStage::Fragment,
                    16,
                ),
                binding(
                    DrawPassResource::ClipAtomicPlane,
                    DrawPassStage::Fragment,
                    17,
                ),
                binding(
                    DrawPassResource::CoverageAtomicPlane,
                    DrawPassStage::Fragment,
                    19,
                ),
            ]
        );
    }

    #[test]
    fn raster_order_paints_are_vertex_bound_and_empty_tables_are_omitted() {
        let empty = draw_pass_binding_plan(DrawPassPlanInput {
            interlock_mode: InterlockMode::RasterOrdering,
            baseline_shader_misc_flags: FIXED_FUNCTION_COLOR_OUTPUT,
            path_count: 0,
            contour_count: 0,
        });
        assert_eq!(empty.len(), 7);

        let populated = draw_pass_binding_plan(DrawPassPlanInput {
            interlock_mode: InterlockMode::RasterOrdering,
            baseline_shader_misc_flags: FIXED_FUNCTION_COLOR_OUTPUT,
            path_count: 1,
            contour_count: 1,
        });
        assert!(populated.contains(&binding(DrawPassResource::Paints, DrawPassStage::Vertex, 6,)));
        assert!(populated.contains(&binding(
            DrawPassResource::PaintAux,
            DrawPassStage::Vertex,
            7,
        )));
        assert!(!populated.iter().any(|binding| matches!(
            binding.resource,
            DrawPassResource::ColorAtomicPlane
                | DrawPassResource::ClipAtomicPlane
                | DrawPassResource::CoverageAtomicPlane
        )));
    }

    #[test]
    fn clockwise_atomic_uses_vertex_paints_without_generic_atomic_planes() {
        let actual = draw_pass_binding_plan(DrawPassPlanInput {
            interlock_mode: InterlockMode::ClockwiseAtomic,
            baseline_shader_misc_flags: 0,
            path_count: 1,
            contour_count: 1,
        });
        assert!(actual.contains(&binding(DrawPassResource::Paints, DrawPassStage::Vertex, 6,)));
        assert!(actual.contains(&binding(
            DrawPassResource::PaintAux,
            DrawPassStage::Vertex,
            7,
        )));
        assert!(!actual.iter().any(|binding| matches!(
            binding.resource,
            DrawPassResource::ColorAtomicPlane
                | DrawPassResource::ClipAtomicPlane
                | DrawPassResource::CoverageAtomicPlane
        )));
    }
}
