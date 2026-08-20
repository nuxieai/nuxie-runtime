// Mechanical translation of:
//   renderer/include/rive/renderer/ore/ore_render_pass.hpp
//   renderer/include/rive/renderer/ore/ore_context.hpp
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
//
// Copyright 2025 Rive

#![allow(non_snake_case)]

use std::fmt;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use std::sync::Weak;

#[cfg(any(target_os = "ios", target_os = "macos"))]
use crate::context::ContextState;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use crate::gpu_resource::AnyResourceHandle;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use crate::metal::texture::{TextureMetal, TextureViewMetal};
#[cfg(any(target_os = "ios", target_os = "macos"))]
use crate::pipeline::Pipeline;
use crate::types::kMaxBindGroups;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use crate::types::{RenderPassDesc, TextureFormat};

/// A render-pass operation rejected before it can mutate native encoder state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderPassError {
    Finished,
    WrongBackendResource(&'static str),
    BindGroupIndexOutOfRange {
        index: u32,
    },
    BufferOffsetOutOfRange {
        offset: u32,
        size: u32,
    },
    VertexBufferSlotOutOfRange {
        slot: u32,
    },
    NativeBindingSlotOutOfRange {
        kind: &'static str,
        slot: u16,
        limit: u16,
    },
    MissingIndexBuffer,
    InvalidIndexFormat,
    IndexOffsetOverflow,
    IndexBufferOutOfRange,
    DynamicOffsetOverflow,
    PipelineIncompatible(String),
}

impl fmt::Display for RenderPassError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Finished => formatter.write_str("RenderPassMetal already finished"),
            Self::WrongBackendResource(kind) => {
                write!(formatter, "expected a Metal {kind}")
            }
            Self::BindGroupIndexOutOfRange { index } => write!(
                formatter,
                "setBindGroup: group index {index} out of range [0, {kMaxBindGroups})"
            ),
            Self::BufferOffsetOutOfRange { offset, size } => {
                write!(
                    formatter,
                    "buffer offset {offset} exceeds buffer size {size}"
                )
            }
            Self::VertexBufferSlotOutOfRange { slot } => write!(
                formatter,
                "setVertexBuffer: slot {slot} out of range [0, 15)"
            ),
            Self::NativeBindingSlotOutOfRange { kind, slot, limit } => write!(
                formatter,
                "setBindGroup: Metal {kind} slot {slot} out of range [0, {limit})"
            ),
            Self::MissingIndexBuffer => {
                formatter.write_str("Must call setIndexBuffer before drawIndexed")
            }
            Self::InvalidIndexFormat => {
                formatter.write_str("setIndexBuffer: IndexFormat::none is invalid")
            }
            Self::IndexOffsetOverflow => {
                formatter.write_str("drawIndexed: index buffer offset overflow")
            }
            Self::IndexBufferOutOfRange => {
                formatter.write_str("drawIndexed: index range exceeds the bound index buffer")
            }
            Self::DynamicOffsetOverflow => {
                formatter.write_str("setBindGroup: dynamic buffer offset overflow")
            }
            Self::PipelineIncompatible(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for RenderPassError {}

/// Portable attachment and lifetime state shared by concrete render passes.
///
/// The context is deliberately weak, matching upstream's non-owning
/// `RenderPass::m_context`. Bound groups are the only logical resource owners
/// in this base. A concrete backend owns its current pipeline separately.
#[cfg(any(target_os = "ios", target_os = "macos"))]
pub(crate) struct RenderPass {
    m_finished: bool,
    m_colorFormats: [TextureFormat; 4],
    m_colorCount: u32,
    m_depthFormat: TextureFormat,
    m_hasDepth: bool,
    m_sampleCount: u32,
    m_context: Weak<ContextState>,
    m_boundGroups: [Option<AnyResourceHandle>; kMaxBindGroups as usize],
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
impl RenderPass {
    pub(crate) fn new(context: Weak<ContextState>, desc: &RenderPassDesc<'_>) -> Self {
        let mut pass = Self {
            m_finished: false,
            m_colorFormats: [TextureFormat::r8unorm; 4],
            m_colorCount: desc.colorCount,
            m_depthFormat: TextureFormat::r8unorm,
            m_hasDepth: false,
            m_sampleCount: 1,
            m_context: context,
            m_boundGroups: std::array::from_fn(|_| None),
        };
        pass.populate_attachment_metadata(desc);
        pass
    }

    /// Populate exactly the metadata later consumed by pipeline compatibility.
    ///
    /// ContextMetal validates that descriptors contain Metal texture views
    /// before constructing the pass. Keeping the downcast here explicit avoids
    /// adding a speculative cross-platform texture interface just for this
    /// one Metal translation.
    fn populate_attachment_metadata(&mut self, desc: &RenderPassDesc<'_>) {
        for (index, attachment) in desc
            .colorAttachments
            .iter()
            .take(desc.colorCount as usize)
            .enumerate()
        {
            let Some((format, sample_count)) = texture_metadata(attachment.view) else {
                continue;
            };
            self.m_colorFormats[index] = format;
            self.m_sampleCount = sample_count;
        }

        if let Some((format, sample_count)) = texture_metadata(desc.depthStencil.view) {
            self.m_depthFormat = format;
            self.m_hasDepth = true;
            if desc.colorCount == 0 {
                self.m_sampleCount = sample_count;
            }
        }
    }

    pub(crate) fn validate(&self) -> Result<(), RenderPassError> {
        if self.m_finished {
            return Err(RenderPassError::Finished);
        }
        Ok(())
    }

    pub(crate) fn check_pipeline_compat(&self, pipeline: &Pipeline) -> Result<(), RenderPassError> {
        let desc = pipeline.desc();
        if desc.colorCount != self.m_colorCount {
            return self.incompatible(format!(
                "setPipeline: pipeline has {} color targets but render pass was begun with {}",
                desc.colorCount, self.m_colorCount
            ));
        }

        for index in 0..self.m_colorCount as usize {
            let Some((target, pass_format)) = desc
                .colorTargets
                .get(index)
                .zip(self.m_colorFormats.get(index))
            else {
                return self.incompatible(format!(
                    "setPipeline: color target count {} exceeds Ore's limit of {}",
                    self.m_colorCount,
                    self.m_colorFormats.len()
                ));
            };
            if target.format != *pass_format {
                return self.incompatible(format!(
                    "setPipeline: color target {index} format mismatch (pipeline={}, pass={})",
                    target.format as u8, *pass_format as u8
                ));
            }
        }

        if desc.sampleCount != self.m_sampleCount {
            return self.incompatible(format!(
                "setPipeline: sample count mismatch (pipeline={}, pass={})",
                desc.sampleCount, self.m_sampleCount
            ));
        }

        // rgba8unorm is the upstream no-depth sentinel.
        let pipeline_has_depth = desc.depthStencil.format != TextureFormat::rgba8unorm;
        if pipeline_has_depth != self.m_hasDepth {
            let explanation = if pipeline_has_depth {
                "pipeline expects depth but pass has none"
            } else {
                "pipeline has no depth but pass provides it"
            };
            return self.incompatible(format!(
                "setPipeline: depth attachment {explanation} (pipeline={}, pass={})",
                u8::from(pipeline_has_depth),
                u8::from(self.m_hasDepth)
            ));
        }

        if pipeline_has_depth && desc.depthStencil.format != self.m_depthFormat {
            return self.incompatible(format!(
                "setPipeline: depth format mismatch (pipeline={}, pass={})",
                desc.depthStencil.format as u8, self.m_depthFormat as u8
            ));
        }
        Ok(())
    }

    pub(crate) fn retain_bound_group(
        &mut self,
        group_index: u32,
        group: &AnyResourceHandle,
    ) -> Result<(), RenderPassError> {
        let Some(slot) = self.m_boundGroups.get_mut(group_index as usize) else {
            return self.fail(RenderPassError::BindGroupIndexOutOfRange { index: group_index });
        };
        *slot = Some(group.clone());
        Ok(())
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.m_finished
    }

    /// Mark finished and release the exact portable owners released by C++.
    pub(crate) fn finish(&mut self) {
        if self.m_finished {
            return;
        }
        self.m_finished = true;
        for group in &mut self.m_boundGroups {
            *group = None;
        }
    }

    pub(crate) fn fail<T>(&self, error: RenderPassError) -> Result<T, RenderPassError> {
        if let Some(context) = self.m_context.upgrade() {
            context.set_last_error(&error.to_string());
        }
        Err(error)
    }

    fn incompatible<T>(&self, message: String) -> Result<T, RenderPassError> {
        self.fail(RenderPassError::PipelineIncompatible(message))
    }
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
fn texture_metadata(view: Option<&AnyResourceHandle>) -> Option<(TextureFormat, u32)> {
    let view = view?.downcast_ref::<TextureViewMetal>()?;
    let texture = view.base().texture().downcast_ref::<TextureMetal>()?;
    Some((texture.base().format(), texture.base().sampleCount()))
}

#[cfg(all(test, any(target_os = "ios", target_os = "macos")))]
mod tests {
    use super::*;
    use crate::gpu_resource::ResourceHandle;
    use crate::pipeline::Pipeline;
    use crate::types::{
        ColorAttachment, ColorTargetState, DepthStencilAttachment, DepthStencilState, PipelineDesc,
        TextureAspect, TextureDesc, TextureViewDesc, TextureViewDimension,
    };
    use std::sync::Arc;

    fn texture_view(format: TextureFormat, sample_count: u32) -> AnyResourceHandle {
        let texture_desc = TextureDesc {
            width: 4,
            height: 4,
            format,
            renderTarget: true,
            sampleCount: sample_count,
            ..TextureDesc::default()
        };
        let texture = TextureMetal::new(&texture_desc).into_resource(None).erase();
        let view_desc = TextureViewDesc {
            texture: &texture,
            dimension: TextureViewDimension::texture2D,
            aspect: TextureAspect::all,
            baseMipLevel: 0,
            mipCount: 1,
            baseLayer: 0,
            layerCount: 1,
        };
        TextureViewMetal::new(&view_desc)
            .into_resource(None)
            .erase()
    }

    fn pass_for(desc: &RenderPassDesc<'_>) -> RenderPass {
        RenderPass::new(Weak::new(), desc)
    }

    #[test]
    fn attachment_metadata_drives_compatibility_in_upstream_check_order() {
        let color = texture_view(TextureFormat::bgra8unorm, 4);
        let depth = texture_view(TextureFormat::depth32float, 4);
        let pass_desc = RenderPassDesc {
            colorAttachments: [
                ColorAttachment {
                    view: Some(&color),
                    ..ColorAttachment::default()
                },
                ColorAttachment::default(),
                ColorAttachment::default(),
                ColorAttachment::default(),
            ],
            colorCount: 1,
            depthStencil: DepthStencilAttachment {
                view: Some(&depth),
                ..DepthStencilAttachment::default()
            },
            label: None,
        };
        let pass = pass_for(&pass_desc);
        let pipeline = Pipeline::new(&PipelineDesc {
            colorTargets: [ColorTargetState::default(); 4],
            colorCount: 1,
            depthStencil: DepthStencilState {
                format: TextureFormat::depth32float,
                ..DepthStencilState::default()
            },
            sampleCount: 4,
            ..PipelineDesc::default()
        });

        assert_eq!(pass.check_pipeline_compat(&pipeline), Ok(()));

        let wrong_count = Pipeline::new(&PipelineDesc {
            colorCount: 0,
            sampleCount: 1,
            ..PipelineDesc::default()
        });
        assert_eq!(
            pass.check_pipeline_compat(&wrong_count),
            Err(RenderPassError::PipelineIncompatible(
                "setPipeline: pipeline has 0 color targets but render pass was begun with 1"
                    .to_owned()
            ))
        );
    }

    #[test]
    fn depth_only_pass_takes_its_sample_count_from_depth() {
        let depth = texture_view(TextureFormat::depth16unorm, 2);
        let desc = RenderPassDesc {
            colorCount: 0,
            depthStencil: DepthStencilAttachment {
                view: Some(&depth),
                ..DepthStencilAttachment::default()
            },
            ..RenderPassDesc::default()
        };
        let pass = pass_for(&desc);
        let pipeline = Pipeline::new(&PipelineDesc {
            colorCount: 0,
            depthStencil: DepthStencilState {
                format: TextureFormat::depth16unorm,
                ..DepthStencilState::default()
            },
            sampleCount: 2,
            ..PipelineDesc::default()
        });

        assert_eq!(pass.check_pipeline_compat(&pipeline), Ok(()));
    }

    #[test]
    fn bound_groups_are_retained_by_slot_until_finish_and_finish_is_idempotent() {
        let desc = RenderPassDesc {
            colorCount: 0,
            ..RenderPassDesc::default()
        };
        let mut pass = pass_for(&desc);
        let group = ResourceHandle::new(None, 7_u32).erase();

        pass.retain_bound_group(2, &group)
            .expect("retain valid group slot");
        assert_eq!(group.debugging_ref_count(), 2);
        pass.finish();
        assert_eq!(group.debugging_ref_count(), 1);
        pass.finish();
        assert_eq!(group.debugging_ref_count(), 1);
        assert_eq!(pass.validate(), Err(RenderPassError::Finished));
    }

    #[test]
    fn invalid_group_slot_fails_closed_without_retaining() {
        let desc = RenderPassDesc {
            colorCount: 0,
            ..RenderPassDesc::default()
        };
        let mut pass = pass_for(&desc);
        let group = ResourceHandle::new(None, 9_u32).erase();

        assert_eq!(
            pass.retain_bound_group(kMaxBindGroups, &group),
            Err(RenderPassError::BindGroupIndexOutOfRange {
                index: kMaxBindGroups
            })
        );
        assert_eq!(group.debugging_ref_count(), 1);
    }

    #[test]
    fn compatibility_failure_routes_the_first_exact_error_to_the_live_context() {
        let context = ContextState::new(crate::types::Features::default(), None);
        let desc = RenderPassDesc {
            colorCount: 0,
            ..RenderPassDesc::default()
        };
        let pass = RenderPass::new(Arc::downgrade(&context), &desc);
        let pipeline = Pipeline::new(&PipelineDesc {
            colorCount: 1,
            ..PipelineDesc::default()
        });

        assert!(matches!(
            pass.check_pipeline_compat(&pipeline),
            Err(RenderPassError::PipelineIncompatible(_))
        ));
        assert_eq!(
            context.last_error(),
            "setPipeline: pipeline has 1 color targets but render pass was begun with 0"
        );
    }
}
