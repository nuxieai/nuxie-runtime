// Mechanical translation of:
//   renderer/src/ore/metal/ore_pipeline_metal.hpp
//   renderer/src/ore/metal/ore_pipeline_metal.mm
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
//
// Copyright 2025 Rive

#![allow(non_snake_case)]

use std::any::Any;

use crate::gpu_resource::{GpuResourceManager, ResourceHandle};
use crate::pipeline::Pipeline;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use crate::types::PipelineDesc;
use crate::types::{BackendId, Pipeline as PipelineResource};

use super::MetalBackend;

#[cfg(any(target_os = "ios", target_os = "macos"))]
use objc2::rc::Retained;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use objc2::runtime::ProtocolObject;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use objc2_metal::{MTLDepthStencilState, MTLRenderPipelineState};

#[cfg(any(target_os = "ios", target_os = "macos"))]
struct RetainedMetalPipeline(Retained<ProtocolObject<dyn MTLRenderPipelineState>>);

// SAFETY: MTLRenderPipelineState is immutable after creation. The wrapper
// exposes only shared access and Objective-C retain/release operations.
#[cfg(any(target_os = "ios", target_os = "macos"))]
unsafe impl Send for RetainedMetalPipeline {}
// SAFETY: Same immutable-state invariant as `Send` above.
#[cfg(any(target_os = "ios", target_os = "macos"))]
unsafe impl Sync for RetainedMetalPipeline {}

#[cfg(any(target_os = "ios", target_os = "macos"))]
struct RetainedMetalDepthStencil(Retained<ProtocolObject<dyn MTLDepthStencilState>>);

// SAFETY: MTLDepthStencilState is immutable after creation. The wrapper
// exposes only shared access and Objective-C retain/release operations.
#[cfg(any(target_os = "ios", target_os = "macos"))]
unsafe impl Send for RetainedMetalDepthStencil {}
// SAFETY: Same immutable-state invariant as `Send` above.
#[cfg(any(target_os = "ios", target_os = "macos"))]
unsafe impl Sync for RetainedMetalDepthStencil {}

/// Concrete Metal pipeline retaining the compiled render and optional
/// depth/stencil states.
///
/// The type cannot be constructed or published without a render state. The
/// `ContextMetal` validates shader entry points and creates
/// the native states before calling [`PipelineMetal::with_native_states`];
/// this leaf intentionally does not recreate
/// `ore_context_metal.mm`'s descriptor translation or error publication.
pub struct PipelineMetal {
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    m_mtlDepthStencil: Option<RetainedMetalDepthStencil>,
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    m_mtlPipeline: RetainedMetalPipeline,
    pipeline: Pipeline,
}

impl PipelineMetal {
    /// Narrow native-state publication seam for the context unit.
    ///
    /// `ore_context_metal.mm` owns shader validation, render-pipeline
    /// descriptor construction, NSError/exception handling, and depth/stencil
    /// descriptor construction. This constructor only transfers the already
    /// successful retained states into the Metal pipeline payload.
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    pub(crate) fn with_native_states(
        desc: &PipelineDesc<'_>,
        pipeline: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
        depth_stencil: Option<Retained<ProtocolObject<dyn MTLDepthStencilState>>>,
    ) -> Self {
        Self {
            m_mtlDepthStencil: depth_stencil.map(RetainedMetalDepthStencil),
            m_mtlPipeline: RetainedMetalPipeline(pipeline),
            pipeline: Pipeline::new(desc),
        }
    }

    pub fn base(&self) -> &Pipeline {
        &self.pipeline
    }

    #[cfg_attr(
        not(any(target_os = "ios", target_os = "macos")),
        expect(dead_code, reason = "the ContextMetal factory is Apple-only")
    )]
    pub(crate) fn into_resource(self, manager: Option<GpuResourceManager>) -> ResourceHandle<Self> {
        ResourceHandle::new(manager, self)
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    pub(crate) fn mtl_pipeline(&self) -> &ProtocolObject<dyn MTLRenderPipelineState> {
        self.m_mtlPipeline.0.as_ref()
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    pub(crate) fn mtl_depth_stencil(&self) -> Option<&ProtocolObject<dyn MTLDepthStencilState>> {
        self.m_mtlDepthStencil
            .as_ref()
            .map(|depth_stencil| depth_stencil.0.as_ref())
    }
}

impl PipelineResource for PipelineMetal {
    fn backend_id(&self) -> BackendId {
        BackendId::of::<MetalBackend>()
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    use std::sync::Arc;
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    use std::sync::atomic::{AtomicBool, Ordering};

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    use objc2::rc::Weak;

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn native_states_publish_together_and_own_portable_dependencies() {
        use objc2_foundation::NSString;
        use objc2_metal::{
            MTLCreateSystemDefaultDevice, MTLDepthStencilDescriptor, MTLDevice, MTLLibrary,
            MTLPixelFormat, MTLRenderPipelineDescriptor,
        };

        let Some((pipeline_owner, depth_owner, portable_dropped)) = objc2::rc::autoreleasepool(
            |_| {
                let device = MTLCreateSystemDefaultDevice()?;
                let source = NSString::from_str(
                    "#include <metal_stdlib>\nusing namespace metal;\nvertex float4 vertex_main(uint vertex_id [[vertex_id]]) { return float4(vertex_id == 0 ? -1.0 : 1.0, -1.0, 0.0, 1.0); }",
                );
                let library = device
                    .newLibraryWithSource_options_error(&source, None)
                    .expect("compile minimal pipeline test library");
                let function = library
                    .newFunctionWithName(&NSString::from_str("vertex_main"))
                    .expect("load vertex function");
                let descriptor = MTLRenderPipelineDescriptor::new();
                descriptor.setVertexFunction(Some(&function));
                descriptor.setDepthAttachmentPixelFormat(MTLPixelFormat::Depth32Float);
                let native_pipeline = device
                    .newRenderPipelineStateWithDescriptor_error(&descriptor)
                    .expect("compile minimal render pipeline");
                let native_depth = device
                    .newDepthStencilStateWithDescriptor(&MTLDepthStencilDescriptor::new())
                    .expect("create depth-stencil state");

                let native_pipeline_pointer = Retained::as_ptr(&native_pipeline);
                let pipeline_owner = Weak::new(&*native_pipeline);
                let depth_owner = Weak::new(&*native_depth);
                let portable_dropped = Arc::new(AtomicBool::new(false));
                let module = ResourceHandle::new(
                    None,
                    DropProbe {
                        dropped: Arc::clone(&portable_dropped),
                    },
                )
                .erase();
                let desc = PipelineDesc {
                    vertexModule: Some(&module),
                    ..PipelineDesc::default()
                };
                let pipeline =
                    PipelineMetal::with_native_states(&desc, native_pipeline, Some(native_depth));
                drop(module);

                assert_eq!(pipeline.base().desc().colorCount, 1);
                assert_eq!(pipeline.base().desc().sampleCount, 1);
                let resource: &dyn PipelineResource = &pipeline;
                assert_eq!(resource.backend_id(), BackendId::of::<MetalBackend>());
                assert!(
                    resource
                        .downcast_ref::<PipelineMetal>(BackendId::of::<MetalBackend>())
                        .is_some()
                );
                enum OtherBackend {}
                assert!(
                    resource
                        .downcast_ref::<PipelineMetal>(BackendId::of::<OtherBackend>())
                        .is_none()
                );
                assert_eq!(
                    native_pipeline_pointer,
                    std::ptr::from_ref(pipeline.mtl_pipeline())
                );
                assert!(pipeline.mtl_depth_stencil().is_some());
                assert!(pipeline_owner.load().is_some());
                assert!(depth_owner.load().is_some());
                let handle = pipeline.into_resource(None);
                assert_eq!(handle.debugging_ref_count(), 1);
                drop(handle);

                Some((pipeline_owner, depth_owner, portable_dropped))
            },
        ) else {
            return;
        };

        // Metal is allowed to cache immutable state objects after our last
        // retain is released, so weak deallocation is not a stable oracle.
        drop(pipeline_owner);
        drop(depth_owner);
        assert!(portable_dropped.load(Ordering::Relaxed));
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    struct DropProbe {
        dropped: Arc<AtomicBool>,
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.dropped.store(true, Ordering::Relaxed);
        }
    }
}
