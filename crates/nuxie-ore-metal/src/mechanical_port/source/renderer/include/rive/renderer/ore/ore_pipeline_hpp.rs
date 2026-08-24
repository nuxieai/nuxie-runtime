/*
 * Copyright 2025 Rive
 */

// #pragma once
// #include "rive/renderer/gpu_resource.hpp"
// #include "utils/lite_rtti.hpp"
// #include "rive/renderer/ore/ore_types.hpp"
// #include "rive/renderer/ore/ore_bind_group_layout.hpp"
// #include "rive/renderer/ore/ore_shader_module.hpp"

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/ore/ore_pipeline.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use super::*;

use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

use super::super::gpu_resource_hpp::{AnyResourceHandle, GPUResource, GpuResourcePayload};
use super::ore_binding_map_hpp::BindingMap;
use super::ore_types_hpp::{
    ColorTargetState, CullMode, DepthStencilState, FaceWinding, IndexFormat, PipelineDesc,
    PrimitiveTopology, StencilFaceState, VertexAttribute, VertexStepMode, kMaxBindGroups,
};
#[cfg(target_vendor = "apple")]
use crate::mechanical_port::source::renderer::src::ore::metal::ore_shader_module_metal_hpp::ShaderModuleMetal;

#[derive(Clone)]
pub struct VertexBufferLayoutSnapshot {
    pub stride: u32,
    pub stepMode: VertexStepMode,
    pub attributes: Vec<VertexAttribute>,
}

/// Owned spelling of the source shallow `PipelineDesc` member. The C++
/// descriptor points into shader modules non-owningly and into layouts that
/// Pipeline explicitly retains. Rust copies strings/slices and retains only
/// those layouts; binding maps are copied before this snapshot is built.
pub struct PipelineSnapshot {
    pub vertexEntryPoint: Option<String>,
    pub fragmentEntryPoint: Option<String>,
    pub vertexBuffers: Vec<VertexBufferLayoutSnapshot>,
    pub topology: PrimitiveTopology,
    pub indexFormat: IndexFormat,
    pub cullMode: CullMode,
    pub winding: FaceWinding,
    pub colorTargets: [ColorTargetState; 4],
    pub colorCount: u32,
    pub depthStencil: DepthStencilState,
    pub stencilFront: StencilFaceState,
    pub stencilBack: StencilFaceState,
    pub stencilReadMask: u8,
    pub stencilWriteMask: u8,
    pub sampleCount: u32,
    pub bindGroupLayoutCount: u32,
    pub label: Option<String>,
}

impl PipelineSnapshot {
    pub fn from_desc(desc: &PipelineDesc<'_>) -> Option<Self> {
        let vertex_buffers = desc.vertexBuffers.unwrap_or(&[]);
        let vertex_buffers = vertex_buffers.get(..desc.vertexBufferCount as usize)?;
        let vertexBuffers = vertex_buffers
            .iter()
            .map(|layout| {
                let attributes = layout
                    .attributes
                    .get(..layout.attributeCount as usize)?
                    .to_vec();
                Some(VertexBufferLayoutSnapshot {
                    stride: layout.stride,
                    stepMode: layout.stepMode,
                    attributes,
                })
            })
            .collect::<Option<Vec<_>>>()?;

        let layouts = desc.bindGroupLayouts.unwrap_or(&[]);
        let _ = layouts.get(..desc.bindGroupLayoutCount as usize)?;

        Some(Self {
            vertexEntryPoint: desc.vertexEntryPoint.map(str::to_owned),
            fragmentEntryPoint: desc.fragmentEntryPoint.map(str::to_owned),
            vertexBuffers,
            topology: desc.topology,
            indexFormat: desc.indexFormat,
            cullMode: desc.cullMode,
            winding: desc.winding,
            colorTargets: desc.colorTargets,
            colorCount: desc.colorCount,
            depthStencil: desc.depthStencil,
            stencilFront: desc.stencilFront,
            stencilBack: desc.stencilBack,
            stencilReadMask: desc.stencilReadMask,
            stencilWriteMask: desc.stencilWriteMask,
            sampleCount: desc.sampleCount,
            bindGroupLayoutCount: desc.bindGroupLayoutCount,
            label: desc.label.map(str::to_owned),
        })
    }
}

// class Pipeline : public rive::gpu::GPUResource,
//                  public ENABLE_LITE_RTTI(Pipeline)
#[repr(C)]
pub struct PipelineMembers {
    pub m_bindingMap: ManuallyDrop<BindingMap>,
    pub(crate) m_layouts: ManuallyDrop<[Option<AnyResourceHandle>; kMaxBindGroups as usize]>,
    pub(crate) m_desc: ManuallyDrop<PipelineSnapshot>,
}

#[repr(C)]
pub struct Pipeline {
    pub(crate) base: ManuallyDrop<GPUResource>,
    pub(crate) members: ManuallyDrop<PipelineMembers>,
}

impl Deref for Pipeline {
    type Target = PipelineMembers;

    fn deref(&self) -> &Self::Target {
        &self.members
    }
}

impl DerefMut for Pipeline {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.members
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            // Pinned reverse destruction: m_desc, m_layouts, m_bindingMap,
            // then the exact inherited GPUResource base.
            #[cfg(test)]
            super::super::gpu_resource_hpp::record_resource_drop_stage("Pipeline.desc");
            ManuallyDrop::drop(&mut self.m_desc);
            #[cfg(test)]
            super::super::gpu_resource_hpp::record_resource_drop_stage("Pipeline.layouts");
            ManuallyDrop::drop(&mut self.m_layouts);
            #[cfg(test)]
            super::super::gpu_resource_hpp::record_resource_drop_stage("Pipeline.bindingMap");
            ManuallyDrop::drop(&mut self.m_bindingMap);
            #[cfg(test)]
            super::super::gpu_resource_hpp::record_resource_drop_stage("Pipeline.base");
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

unsafe impl GpuResourcePayload for Pipeline {
    fn gpu_resource(&self) -> &GPUResource {
        &self.base
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        &mut self.base
    }
}

impl Pipeline {
    pub(crate) fn new(desc: &PipelineDesc<'_>) -> Option<Self> {
        let mut m_bindingMap = BindingMap::default();
        let module = desc.vertexModule.or(desc.fragmentModule);
        if let Some(module) = module {
            #[cfg(target_vendor = "apple")]
            if let Some(module) = module.downcast_ref::<ShaderModuleMetal>() {
                m_bindingMap = module.base.m_bindingMap.clone();
            } else if let Some(module) = module.downcast_ref::<ShaderModule>() {
                m_bindingMap = module.m_bindingMap.clone();
            }
            #[cfg(not(target_vendor = "apple"))]
            if let Some(module) = module.downcast_ref::<ShaderModule>() {
                m_bindingMap = module.m_bindingMap.clone();
            }
        }
        let m_desc = PipelineSnapshot::from_desc(desc)?;
        let mut m_layouts = std::array::from_fn(|_| None);
        let layouts = desc.bindGroupLayouts.unwrap_or(&[]);
        for (destination, source) in m_layouts
            .iter_mut()
            .zip(layouts.get(..desc.bindGroupLayoutCount as usize)?)
        {
            *destination = source.map(Clone::clone);
        }
        Some(Self {
            base: ManuallyDrop::new(GPUResource::new(None)),
            members: ManuallyDrop::new(PipelineMembers {
                m_bindingMap: ManuallyDrop::new(m_bindingMap),
                m_layouts: ManuallyDrop::new(m_layouts),
                m_desc: ManuallyDrop::new(m_desc),
            }),
        })
    }

    pub fn desc(&self) -> &PipelineSnapshot {
        &self.m_desc
    }

    pub fn layout(&self, group: u32) -> Option<&AnyResourceHandle> {
        self.m_layouts.get(group as usize).and_then(Option::as_ref)
    }

    pub fn has_layout(&self, group: u32, layout: &AnyResourceHandle) -> bool {
        self.layout(group)
            .is_some_and(|retained| retained.ptr_eq(layout))
    }
}

// The optional GPUResourceManager is owned by the outer concrete
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_resource_hpp::ResourceHandle;
    use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_shader_module_hpp::ShaderModule;
    use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_types_hpp::{
        PipelineDesc, VertexAttribute, VertexBufferLayout,
    };

    #[test]
    fn pipeline_snapshot_owns_nested_descriptor_values_and_handles() {
        let module = ResourceHandle::new(None, ShaderModule::new()).erase();
        let attrs = [VertexAttribute {
            offset: 12,
            shaderSlot: 3,
            ..VertexAttribute::default()
        }];
        let layouts = [VertexBufferLayout {
            stride: 28,
            attributes: &attrs,
            attributeCount: attrs.len() as u32,
            ..VertexBufferLayout::default()
        }];
        let bind_group_layout =
            ResourceHandle::new(None, crate::gpu_resource::TestGPUResource::new(17_u32)).erase();
        let entry = String::from("vertex");
        let label = String::from("pipeline");
        let desc = PipelineDesc {
            vertexModule: Some(&module),
            vertexEntryPoint: Some(&entry),
            vertexBuffers: Some(&layouts),
            vertexBufferCount: layouts.len() as u32,
            bindGroupLayouts: Some(&[Some(&bind_group_layout)]),
            bindGroupLayoutCount: 1,
            label: Some(&label),
            ..PipelineDesc::default()
        };

        let pipeline = Pipeline::new(&desc).expect("valid pipeline snapshot");
        drop(entry);
        drop(label);
        assert_eq!(pipeline.desc().vertexEntryPoint.as_deref(), Some("vertex"));
        assert_eq!(pipeline.desc().label.as_deref(), Some("pipeline"));
        assert_eq!(
            pipeline
                .desc()
                .vertexBuffers
                .first()
                .and_then(|layout| layout.attributes.first())
                .map(|attribute| attribute.offset),
            Some(12)
        );
        assert_eq!(pipeline.desc().bindGroupLayoutCount, 1);
        assert!(
            pipeline
                .layout(0)
                .is_some_and(|layout| layout.ptr_eq(&bind_group_layout))
        );
        assert_eq!(module.debugging_refcnt(), 1);
        assert_eq!(bind_group_layout.debugging_refcnt(), 2);
    }

    #[test]
    fn pipeline_copies_vertex_binding_map_and_falls_back_to_fragment() {
        let mut vertex_module = ShaderModule::new();
        let vertex_blob = binding_map_blob(0, 1);
        assert!(BindingMap::fromBlob(
            Some(&vertex_blob),
            vertex_blob.len(),
            Some(&mut vertex_module.m_bindingMap),
        ));
        let expected_vertex = vertex_module.m_bindingMap.clone();
        let vertex = ResourceHandle::new(None, vertex_module).erase();

        let mut fragment_module = ShaderModule::new();
        let fragment_blob = binding_map_blob(2, 3);
        assert!(BindingMap::fromBlob(
            Some(&fragment_blob),
            fragment_blob.len(),
            Some(&mut fragment_module.m_bindingMap),
        ));
        let expected_fragment = fragment_module.m_bindingMap.clone();
        let fragment = ResourceHandle::new(None, fragment_module).erase();
        let desc = PipelineDesc {
            vertexModule: Some(&vertex),
            fragmentModule: Some(&fragment),
            ..PipelineDesc::default()
        };
        let pipeline = Pipeline::new(&desc).expect("vertex pipeline");
        assert_eq!(&*pipeline.m_bindingMap, &expected_vertex);

        let fragment_only = PipelineDesc {
            fragmentModule: Some(&fragment),
            ..PipelineDesc::default()
        };
        let pipeline = Pipeline::new(&fragment_only).expect("fragment pipeline");
        assert_eq!(&*pipeline.m_bindingMap, &expected_fragment);
    }

    #[test]
    fn pipeline_retains_only_the_first_four_layouts_once() {
        let layout0 =
            ResourceHandle::new(None, crate::gpu_resource::TestGPUResource::new(0_u32)).erase();
        let layout1 =
            ResourceHandle::new(None, crate::gpu_resource::TestGPUResource::new(1_u32)).erase();
        let layout2 =
            ResourceHandle::new(None, crate::gpu_resource::TestGPUResource::new(2_u32)).erase();
        let layout3 =
            ResourceHandle::new(None, crate::gpu_resource::TestGPUResource::new(3_u32)).erase();
        let layout4 =
            ResourceHandle::new(None, crate::gpu_resource::TestGPUResource::new(4_u32)).erase();
        let layouts = [
            Some(&layout0),
            Some(&layout1),
            Some(&layout2),
            Some(&layout3),
            Some(&layout4),
        ];
        let pipeline = Pipeline::new(&PipelineDesc {
            bindGroupLayouts: Some(&layouts),
            bindGroupLayoutCount: layouts.len() as u32,
            ..PipelineDesc::default()
        })
        .expect("layout snapshot");

        assert_eq!(pipeline.desc().bindGroupLayoutCount, 5);
        assert!(pipeline.has_layout(0, &layout0));
        assert!(pipeline.has_layout(3, &layout3));
        assert!(pipeline.layout(4).is_none());
        assert_eq!(layout0.debugging_refcnt(), 2);
        assert_eq!(layout3.debugging_refcnt(), 2);
        assert_eq!(layout4.debugging_refcnt(), 1);
    }

    fn binding_map_blob(group: u8, binding: u8) -> Vec<u8> {
        let mut blob = vec![2, 1, 14, 0, 1, 0, 0, 0];
        blob.extend_from_slice(&[
            group, binding, 0, 1, 0, 7, 0, 0xff, 0xff, 0xff, 0xff, 0, 0, 0,
        ]);
        blob
    }
}

// `ResourceHandle<PipelineMetal>`, not duplicated in this payload.
