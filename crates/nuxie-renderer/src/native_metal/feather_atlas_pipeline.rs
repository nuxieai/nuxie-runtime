//! Native Metal feather-atlas pipeline states.
//!
//! This is a mechanical Rust translation of the pinned upstream
//! `RenderContextMetalImpl::FeatherAtlasPipeline` implementation in
//! `renderer/src/metal/render_context_metal_impl.mm:144-239` and its
//! declaration in `renderer/include/rive/renderer/metal/render_context_metal_impl.h`.
//!
//! Pinned upstream source: `rive-runtime`, commit
//! `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
//!
//! The precompiled Metal library uses the generated entry-point tokens `RF`
//! (atlas vertex), `UE` (feathered fill), and `VE` (feathered stroke). The
//! token names are intentionally kept here rather than inferred from Rust
//! type names: they are part of the upstream shader artifact contract.

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_foundation::NSString;
use objc2_metal::{
    MTLBlendFactor, MTLBlendOperation, MTLColorWriteMask, MTLDevice, MTLFunction, MTLLibrary,
    MTLPixelFormat, MTLRenderPipelineColorAttachmentDescriptor, MTLRenderPipelineDescriptor,
    MTLRenderPipelineState,
};
use std::error::Error;
use std::fmt;

/// Generated entry point for the feather-atlas vertex shader.
pub(crate) const ATLAS_VERTEX_MAIN: &str = "RF";

/// Generated entry point for the feather-atlas fill fragment shader.
pub(crate) const ATLAS_FILL_FRAGMENT_MAIN: &str = "UE";

/// Generated entry point for the feather-atlas stroke fragment shader.
pub(crate) const ATLAS_STROKE_FRAGMENT_MAIN: &str = "VE";

/// The two pipeline variants emitted by the upstream feather-atlas pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FeatherAtlasPipelineKind {
    Fill,
    Stroke,
}

impl FeatherAtlasPipelineKind {
    fn fragment_main(self) -> &'static str {
        match self {
            Self::Fill => ATLAS_FILL_FRAGMENT_MAIN,
            Self::Stroke => ATLAS_STROKE_FRAGMENT_MAIN,
        }
    }

    fn blend_operation(self) -> MTLBlendOperation {
        match self {
            Self::Fill => MTLBlendOperation::Add,
            Self::Stroke => MTLBlendOperation::Max,
        }
    }
}

/// The exact color-attachment policy used for one atlas pipeline.
///
/// Keeping this policy as a small value makes the translation independently
/// testable without requiring a Metal device. It mirrors the assignments in
/// upstream `FeatherAtlasPipeline` rather than introducing backend-neutral
/// blend semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FeatherAtlasAttachmentPolicy {
    pub(crate) pixel_format: MTLPixelFormat,
    pub(crate) blending_enabled: bool,
    pub(crate) source_rgb_blend_factor: MTLBlendFactor,
    pub(crate) destination_rgb_blend_factor: MTLBlendFactor,
    pub(crate) rgb_blend_operation: MTLBlendOperation,
    pub(crate) source_alpha_blend_factor: MTLBlendFactor,
    pub(crate) destination_alpha_blend_factor: MTLBlendFactor,
    pub(crate) alpha_blend_operation: MTLBlendOperation,
    pub(crate) write_mask: MTLColorWriteMask,
}

/// Return the literal upstream attachment policy for the selected atlas pass.
pub(crate) fn attachment_policy(kind: FeatherAtlasPipelineKind) -> FeatherAtlasAttachmentPolicy {
    let blend_operation = kind.blend_operation();
    FeatherAtlasAttachmentPolicy {
        pixel_format: MTLPixelFormat::R16Float,
        blending_enabled: true,
        source_rgb_blend_factor: MTLBlendFactor::One,
        destination_rgb_blend_factor: MTLBlendFactor::One,
        rgb_blend_operation: blend_operation,
        source_alpha_blend_factor: MTLBlendFactor::One,
        destination_alpha_blend_factor: MTLBlendFactor::One,
        alpha_blend_operation: blend_operation,
        write_mask: MTLColorWriteMask::All,
    }
}

/// A typed failure while resolving or realizing one atlas pipeline.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FeatherAtlasPipelineError {
    MissingVertexFunction(String),
    MissingFragmentFunction(String),
    PipelineCreation {
        kind: FeatherAtlasPipelineKind,
        description: String,
    },
}

impl fmt::Display for FeatherAtlasPipelineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingVertexFunction(name) => {
                write!(
                    formatter,
                    "Metal feather-atlas vertex function is unavailable: {name}"
                )
            }
            Self::MissingFragmentFunction(name) => {
                write!(
                    formatter,
                    "Metal feather-atlas fragment function is unavailable: {name}"
                )
            }
            Self::PipelineCreation { kind, description } => write!(
                formatter,
                "Metal feather-atlas {kind:?} pipeline creation failed: {description}"
            ),
        }
    }
}

impl Error for FeatherAtlasPipelineError {}

/// One retained Metal render-pipeline state for the feather atlas.
#[derive(Clone)]
pub(crate) struct FeatherAtlasPipeline {
    pipeline_state: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    #[cfg(test)]
    kind: FeatherAtlasPipelineKind,
}

impl FeatherAtlasPipeline {
    /// Resolve the generated functions and create the exact upstream pipeline.
    pub(crate) fn new(
        gpu: &ProtocolObject<dyn MTLDevice>,
        library: &ProtocolObject<dyn MTLLibrary>,
        kind: FeatherAtlasPipelineKind,
    ) -> Result<Self, FeatherAtlasPipelineError> {
        let vertex_name = NSString::from_str(ATLAS_VERTEX_MAIN);
        let fragment_name = NSString::from_str(kind.fragment_main());
        let vertex = library.newFunctionWithName(&vertex_name).ok_or_else(|| {
            FeatherAtlasPipelineError::MissingVertexFunction(ATLAS_VERTEX_MAIN.to_owned())
        })?;
        let fragment = library.newFunctionWithName(&fragment_name).ok_or_else(|| {
            FeatherAtlasPipelineError::MissingFragmentFunction(kind.fragment_main().to_owned())
        })?;
        let descriptor = make_descriptor(&vertex, &fragment, kind);
        let creation = super::new_render_pipeline_state(gpu, &descriptor);
        if creation.error.is_some() || creation.object.is_none() {
            return Err(FeatherAtlasPipelineError::PipelineCreation {
                kind,
                description: creation.error.unwrap_or_else(|| "<nil>".to_owned()),
            });
        }
        let pipeline_state = creation.object.expect("pipeline checked nonnil");
        Ok(Self {
            pipeline_state,
            #[cfg(test)]
            kind,
        })
    }

    #[cfg(test)]
    pub(crate) fn pipeline_state(&self) -> &ProtocolObject<dyn MTLRenderPipelineState> {
        &self.pipeline_state
    }

    pub(crate) fn retained_pipeline_state(
        &self,
    ) -> Retained<ProtocolObject<dyn MTLRenderPipelineState>> {
        self.pipeline_state.clone()
    }

    #[cfg(test)]
    pub(crate) fn kind(&self) -> FeatherAtlasPipelineKind {
        self.kind
    }
}

/// Retained fill and stroke pipeline states created together, as in the
/// upstream resize path's lazy pair initialization.
#[derive(Clone)]
pub(crate) struct FeatherAtlasPipelines {
    fill: FeatherAtlasPipeline,
    stroke: FeatherAtlasPipeline,
}

impl FeatherAtlasPipelines {
    pub(crate) fn new(
        gpu: &ProtocolObject<dyn MTLDevice>,
        library: &ProtocolObject<dyn MTLLibrary>,
    ) -> Result<Self, FeatherAtlasPipelineError> {
        // Preserve upstream's fill-then-stroke construction order. If stroke
        // creation fails, the already-retained fill state is dropped and no
        // partially initialized pair escapes.
        let fill = FeatherAtlasPipeline::new(gpu, library, FeatherAtlasPipelineKind::Fill)?;
        let stroke = FeatherAtlasPipeline::new(gpu, library, FeatherAtlasPipelineKind::Stroke)?;
        Ok(Self { fill, stroke })
    }

    #[cfg(test)]
    pub(crate) fn fill(&self) -> &ProtocolObject<dyn MTLRenderPipelineState> {
        self.fill.pipeline_state()
    }

    #[cfg(test)]
    pub(crate) fn stroke(&self) -> &ProtocolObject<dyn MTLRenderPipelineState> {
        self.stroke.pipeline_state()
    }

    pub(crate) fn retained(
        &self,
        is_stroke: bool,
    ) -> Retained<ProtocolObject<dyn MTLRenderPipelineState>> {
        if is_stroke {
            self.stroke.retained_pipeline_state()
        } else {
            self.fill.retained_pipeline_state()
        }
    }

    #[cfg(test)]
    pub(crate) fn fill_pipeline(&self) -> &FeatherAtlasPipeline {
        &self.fill
    }

    #[cfg(test)]
    pub(crate) fn stroke_pipeline(&self) -> &FeatherAtlasPipeline {
        &self.stroke
    }
}

fn make_descriptor(
    vertex: &ProtocolObject<dyn MTLFunction>,
    fragment: &ProtocolObject<dyn MTLFunction>,
    kind: FeatherAtlasPipelineKind,
) -> Retained<MTLRenderPipelineDescriptor> {
    let descriptor = MTLRenderPipelineDescriptor::new();
    descriptor.setVertexFunction(Some(vertex));
    descriptor.setFragmentFunction(Some(fragment));

    // SAFETY: Metal render-pipeline descriptors expose eight color-attachment
    // slots. Upstream writes slot zero and objc2 returns its retained,
    // non-null descriptor for this scope; the typed policy contains no raw
    // pointers or unchecked values.
    let attachment = unsafe { descriptor.colorAttachments().objectAtIndexedSubscript(0) };
    apply_attachment_policy(&attachment, attachment_policy(kind));
    descriptor
}

fn apply_attachment_policy(
    attachment: &MTLRenderPipelineColorAttachmentDescriptor,
    policy: FeatherAtlasAttachmentPolicy,
) {
    attachment.setPixelFormat(policy.pixel_format);
    attachment.setBlendingEnabled(policy.blending_enabled);
    attachment.setSourceRGBBlendFactor(policy.source_rgb_blend_factor);
    attachment.setDestinationRGBBlendFactor(policy.destination_rgb_blend_factor);
    attachment.setRgbBlendOperation(policy.rgb_blend_operation);
    attachment.setSourceAlphaBlendFactor(policy.source_alpha_blend_factor);
    attachment.setDestinationAlphaBlendFactor(policy.destination_alpha_blend_factor);
    attachment.setAlphaBlendOperation(policy.alpha_blend_operation);
    attachment.setWriteMask(policy.write_mask);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_policy_matches_pinned_upstream() {
        let policy = attachment_policy(FeatherAtlasPipelineKind::Fill);
        assert_eq!(policy.pixel_format, MTLPixelFormat::R16Float);
        assert!(policy.blending_enabled);
        assert_eq!(policy.source_rgb_blend_factor, MTLBlendFactor::One);
        assert_eq!(policy.destination_rgb_blend_factor, MTLBlendFactor::One);
        assert_eq!(policy.rgb_blend_operation, MTLBlendOperation::Add);
        assert_eq!(policy.source_alpha_blend_factor, MTLBlendFactor::One);
        assert_eq!(policy.destination_alpha_blend_factor, MTLBlendFactor::One);
        assert_eq!(policy.alpha_blend_operation, MTLBlendOperation::Add);
        assert_eq!(policy.write_mask, MTLColorWriteMask::All);
    }

    #[test]
    fn stroke_policy_only_changes_blend_operation() {
        let fill = attachment_policy(FeatherAtlasPipelineKind::Fill);
        let stroke = attachment_policy(FeatherAtlasPipelineKind::Stroke);
        assert_eq!(stroke.pixel_format, MTLPixelFormat::R16Float);
        assert_eq!(stroke.source_rgb_blend_factor, MTLBlendFactor::One);
        assert_eq!(stroke.destination_rgb_blend_factor, MTLBlendFactor::One);
        assert_eq!(stroke.rgb_blend_operation, MTLBlendOperation::Max);
        assert_eq!(stroke.source_alpha_blend_factor, MTLBlendFactor::One);
        assert_eq!(stroke.destination_alpha_blend_factor, MTLBlendFactor::One);
        assert_eq!(stroke.alpha_blend_operation, MTLBlendOperation::Max);
        assert_eq!(stroke.write_mask, MTLColorWriteMask::All);
        assert_ne!(fill.rgb_blend_operation, stroke.rgb_blend_operation);
        assert_ne!(fill.alpha_blend_operation, stroke.alpha_blend_operation);
    }

    #[test]
    fn generated_entry_point_contract_matches_pinned_shader_exports() {
        assert_eq!(ATLAS_VERTEX_MAIN, "RF");
        assert_eq!(ATLAS_FILL_FRAGMENT_MAIN, "UE");
        assert_eq!(ATLAS_STROKE_FRAGMENT_MAIN, "VE");
        assert_eq!(FeatherAtlasPipelineKind::Fill.fragment_main(), "UE");
        assert_eq!(FeatherAtlasPipelineKind::Stroke.fragment_main(), "VE");
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn live_pipeline_pair_resolves_pinned_draw_metallib() {
        let Some(device) = objc2_metal::MTLCreateSystemDefaultDevice() else {
            crate::live_metal_test_unavailable("system Metal device");
            return;
        };
        let library = crate::native_metal::draw_shader::DrawShaderLibrary::load(&device).unwrap();
        let pipelines = FeatherAtlasPipelines::new(&device, library.library()).unwrap();
        assert_eq!(
            pipelines.fill_pipeline().kind(),
            FeatherAtlasPipelineKind::Fill
        );
        assert_eq!(
            pipelines.stroke_pipeline().kind(),
            FeatherAtlasPipelineKind::Stroke
        );
        assert_eq!(
            pipelines.fill_pipeline().pipeline_state() as *const _,
            pipelines.fill() as *const _
        );
        assert_eq!(
            pipelines.stroke_pipeline().pipeline_state() as *const _,
            pipelines.stroke() as *const _
        );
    }
}
