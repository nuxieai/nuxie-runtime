/*
 * Copyright 2025 Rive
 */

// #pragma once
// #include "rive/renderer/ore/ore_pipeline.hpp"
// #import <Metal/Metal.h>

// Mechanical translation of the complete pinned source header
// renderer/src/ore/metal/ore_pipeline_metal.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use super::*;
use std::mem::ManuallyDrop;

use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_resource_hpp::{
    GPUResource, GpuResourcePayload,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_types_hpp::PipelineDesc;

// `id<MTLRenderPipelineState>` is a nullable, strong Objective-C owner under
// ARC. `Retained<T>` is the corresponding strong owner; `Option` preserves
// the source `nil` state. The non-Apple stand-in keeps this source-shaped
// translation available to tools that inspect it off Apple.
#[cfg(target_vendor = "apple")]
use objc2::rc::Retained;
#[cfg(target_vendor = "apple")]
use objc2::runtime::ProtocolObject;
#[cfg(target_vendor = "apple")]
use objc2_metal::{MTLDepthStencilState, MTLRenderPipelineState};

#[cfg(target_vendor = "apple")]
type NativeMetalPipeline = Option<Retained<ProtocolObject<dyn MTLRenderPipelineState>>>;

#[cfg(not(target_vendor = "apple"))]
type NativeMetalPipeline = Option<()>;

#[cfg(target_vendor = "apple")]
type NativeMetalDepthStencil = Option<Retained<ProtocolObject<dyn MTLDepthStencilState>>>;

#[cfg(not(target_vendor = "apple"))]
type NativeMetalDepthStencil = Option<()>;

// namespace rive::ore

// class ContextMetal;
// The source forward declaration is retained for the friend relationship
// below. ContextMetal is owned by its own translation unit.

// class RenderPassMetal;
// The source forward declaration is retained for the friend relationship
// below. RenderPassMetal is owned by its own translation unit.

// class PipelineMetal : public LITE_RTTI_OVERRIDE(Pipeline, PipelineMetal)
// {
// Rust has no class inheritance. `base` is the first field to preserve the
// source Pipeline base-subobject order. `LITE_RTTI_OVERRIDE(Pipeline,
// PipelineMetal)` remains the source lite-RTTI identity/override seam and is
// not duplicated as a payload field.
#[repr(C)]
pub struct PipelineMetal {
    pub(crate) base: ManuallyDrop<Pipeline>,
    // private:
    // friend class ContextMetal;
    // friend class RenderPassMetal;
    // Rust has no friend declarations; these source access boundaries remain
    // visible here, and the owning translation units use crate visibility.
    // id<MTLRenderPipelineState> m_mtlPipeline = nil;
    // `NativeMetalPipeline` retains the non-nil Objective-C render pipeline
    // state until the enclosing logical PipelineMetal owner is dropped.
    // id<MTLDepthStencilState> m_mtlDepthStencil = nil;
    // `NativeMetalDepthStencil` retains the non-nil Objective-C depth/stencil
    // state until the enclosing logical PipelineMetal owner is dropped.
    pub(crate) m_mtlPipeline: ManuallyDrop<NativeMetalPipeline>,
    pub(crate) m_mtlDepthStencil: ManuallyDrop<NativeMetalDepthStencil>,
}

// SAFETY: Metal pipeline/depth-stencil state and the owned portable snapshot
// are immutable after publication and may be released on completion threads.
unsafe impl Send for PipelineMetal {}

unsafe impl GpuResourcePayload for PipelineMetal {
    fn gpu_resource(&self) -> &GPUResource {
        &self.base.base
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        &mut self.base.base
    }
}

impl PipelineMetal {
    // public:

    // PipelineMetal(const PipelineDesc& desc) : lite_rtti_override(desc) {}
    // The source lite-RTTI initializer delegates to the Pipeline base
    // constructor and records the concrete PipelineMetal identity.
    pub(crate) fn new(desc: &PipelineDesc<'_>) -> Option<Self> {
        Some(Self {
            base: ManuallyDrop::new(Pipeline::new(desc)?),
            m_mtlPipeline: ManuallyDrop::new(None),
            m_mtlDepthStencil: ManuallyDrop::new(None),
        })
    }

    // ~PipelineMetal() override = default; // ARC releases Metal objects
    // Rust's default drop glue releases the retained native pipeline and
    // depth/stencil owners in source declaration order before the base.
}

impl Drop for PipelineMetal {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_mtlDepthStencil);
            ManuallyDrop::drop(&mut self.m_mtlPipeline);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

// } // namespace rive::ore
#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_vendor = "apple")]
    use objc2::rc::Weak;

    #[cfg(target_vendor = "apple")]
    #[test]
    fn native_states_publish_together_and_own_portable_dependencies() {
        use objc2_foundation::NSString;
        use objc2_metal::{
            MTLCreateSystemDefaultDevice, MTLDepthStencilDescriptor, MTLDevice, MTLLibrary,
            MTLPixelFormat, MTLRenderPipelineDescriptor,
        };

        let Some((pipeline_owner, depth_owner)) = objc2::rc::autoreleasepool(|_| {
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
            let desc = PipelineDesc {
                colorCount: 0,
                ..PipelineDesc::default()
            };
            let mut pipeline = PipelineMetal::new(&desc).expect("pipeline snapshot");
            pipeline.m_mtlPipeline = ManuallyDrop::new(Some(native_pipeline));
            pipeline.m_mtlDepthStencil = ManuallyDrop::new(Some(native_depth));

            assert_eq!(pipeline.base.desc().colorCount, 0);
            assert_eq!(pipeline.base.desc().sampleCount, 1);
            assert_eq!(
                native_pipeline_pointer,
                Retained::as_ptr(pipeline.m_mtlPipeline.as_ref().unwrap(),)
            );
            assert!(pipeline.m_mtlDepthStencil.is_some());
            assert!(pipeline_owner.load().is_some());
            assert!(depth_owner.load().is_some());
            let handle = crate::gpu_resource::ResourceHandle::new(None, pipeline);
            assert_eq!(handle.debugging_refcnt(), 1);
            drop(handle);

            Some((pipeline_owner, depth_owner))
        }) else {
            return;
        };

        // Metal is allowed to cache immutable state objects after our last
        // retain is released, so weak deallocation is not a stable oracle.
        drop(pipeline_owner);
        drop(depth_owner);
    }
}
