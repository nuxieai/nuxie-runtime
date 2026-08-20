// Mechanical translation of:
//   renderer/include/rive/renderer/ore/ore_pipeline.hpp
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
//
// Copyright 2025 Rive

#![allow(non_snake_case)]

use crate::binding_map::BindingMap;
use crate::gpu_resource::{AnyResourceHandle, GpuResourceManager, ResourceHandle};
use crate::types::{
    ColorTargetState, CullMode, DepthStencilState, FaceWinding, IndexFormat, PipelineDesc,
    PrimitiveTopology, StencilFaceState, VertexAttribute, VertexStepMode,
};

/// An owned vertex-buffer layout copied from `PipelineDesc`.
///
/// C++'s `Pipeline` intentionally shallow-copies the descriptor, but its
/// Rust counterpart cannot retain borrowed slices or entry-point pointers in a
/// deferred GPU resource. This snapshot is the exact value graph consumed by
/// later render-pass code.
#[derive(Clone, Debug)]
pub struct VertexBufferLayoutSnapshot {
    pub stride: u32,
    pub stepMode: VertexStepMode,
    pub attributes: Vec<VertexAttribute>,
}

/// Owned descriptor snapshot held by [`Pipeline`].
///
/// Resource handles are erased only at this boundary so the snapshot can
/// retain the exact concrete payload supplied by the context while keeping
/// the portable pipeline independent of a native Metal type. Strings and
/// nested descriptor slices are copied exactly once at construction.
#[derive(Debug)]
pub struct PipelineSnapshot {
    pub vertexModule: Option<AnyResourceHandle>,
    pub vertexEntryPoint: Option<String>,
    pub fragmentModule: Option<AnyResourceHandle>,
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
    pub bindGroupLayouts: [Option<AnyResourceHandle>; crate::types::kMaxBindGroups as usize],
    pub bindGroupLayoutCount: u32,
    pub label: Option<String>,
}

impl PipelineSnapshot {
    /// Copy every borrowed field in a `PipelineDesc` into owned storage.
    pub fn from_desc(desc: &PipelineDesc<'_>) -> Self {
        let vertexBuffers = desc
            .vertexBuffers
            .unwrap_or(&[])
            .iter()
            .map(|layout| VertexBufferLayoutSnapshot {
                stride: layout.stride,
                stepMode: layout.stepMode,
                attributes: layout.attributes.to_vec(),
            })
            .collect();

        let mut bindGroupLayouts = std::array::from_fn(|_| None);
        let bindGroupLayoutCount = desc.bindGroupLayouts.map_or(0, |layouts| {
            u32::try_from(layouts.len()).unwrap_or(u32::MAX)
        });
        if let Some(layouts) = desc.bindGroupLayouts {
            for (destination, source) in bindGroupLayouts
                .iter_mut()
                .zip(layouts.iter().take(crate::types::kMaxBindGroups as usize))
            {
                *destination = source.as_ref().map(|handle| (*handle).clone());
            }
        }

        Self {
            vertexModule: desc.vertexModule.cloned(),
            vertexEntryPoint: desc.vertexEntryPoint.map(str::to_owned),
            fragmentModule: desc.fragmentModule.cloned(),
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
            bindGroupLayouts,
            bindGroupLayoutCount,
            label: desc.label.map(str::to_owned),
        }
    }

    /// Return the retained layout at a positional group index.
    pub fn layout(&self, group: u32) -> Option<&AnyResourceHandle> {
        self.bindGroupLayouts
            .get(group as usize)
            .and_then(Option::as_ref)
    }
}

/// Portable ORE pipeline payload.
///
/// The upstream base owns a `GPUResource` manager, a binding-map copy, a
/// fixed array of strong nullable layouts, and a shallow `PipelineDesc`.
/// `ResourceHandle<Pipeline>` owns the manager/refcount portion in Rust;
/// `PipelineSnapshot` owns every descriptor referent that would otherwise be
/// a dangling C++ pointer.
#[derive(Debug)]
pub struct Pipeline {
    m_bindingMap: BindingMap,
    m_desc: PipelineSnapshot,
}

impl Pipeline {
    /// Translate the protected unmanaged C++ constructor.
    #[cfg_attr(
        not(any(target_os = "ios", target_os = "macos")),
        expect(dead_code, reason = "the ContextMetal factory is Apple-only")
    )]
    pub(crate) fn new(desc: &PipelineDesc<'_>) -> Self {
        let snapshot = PipelineSnapshot::from_desc(desc);
        let bindingMap = binding_map_from_modules(desc);
        Self {
            m_bindingMap: bindingMap,
            m_desc: snapshot,
        }
    }

    /// Construct the payload while preserving the C++ manager-taking
    /// constructor's separation from the descriptor snapshot. The manager is
    /// owned by the returned `ResourceHandle`, not duplicated in this payload.
    pub fn into_resource(self, manager: Option<GpuResourceManager>) -> ResourceHandle<Self> {
        ResourceHandle::new(manager, self)
    }

    /// C++ `Pipeline::desc()` equivalent over the owned Rust snapshot.
    pub fn desc(&self) -> &PipelineSnapshot {
        &self.m_desc
    }

    pub fn binding_map(&self) -> &BindingMap {
        &self.m_bindingMap
    }

    /// C++ spelling retained for source-corresponding backend callers.
    pub fn bindingMap(&self) -> &BindingMap {
        self.binding_map()
    }

    /// Return the retained layout at a positional group index.
    pub fn layout(&self, group: u32) -> Option<&AnyResourceHandle> {
        // The owned snapshot is also the sole strong-layout array. Keeping a
        // second Rust array would add a logical owner that the shallow C++
        // `m_desc` does not have and would delay deferred destruction.
        self.m_desc.layout(group)
    }

    /// Test WebGPU-style exact layout identity without exposing payload types.
    pub fn has_layout(&self, group: u32, layout: &AnyResourceHandle) -> bool {
        self.layout(group)
            .is_some_and(|retained| retained.ptr_eq(layout))
    }
}

/// Select the vertex binding map, falling back to the fragment map for
/// vertex-less/blit pipelines, exactly as the upstream constructor does.
///
/// The Rust resource handle is erased, so the concrete shader payload is
/// checked at this boundary. A context factory must still validate that the
/// expected backend shader was supplied before publishing a pipeline; an
/// unknown payload yields the same empty-map state as a null shader pointer
/// and cannot create a native Metal pipeline by itself.
fn binding_map_from_modules(desc: &PipelineDesc<'_>) -> BindingMap {
    let module = desc.vertexModule.or(desc.fragmentModule);
    let Some(module) = module else {
        return BindingMap::default();
    };

    if let Some(module) = module.downcast_ref::<crate::shader_module::ShaderModule>() {
        return module.m_bindingMap.clone();
    }
    if let Some(module) = module.downcast_ref::<crate::metal::shader_module::ShaderModuleMetal>() {
        return module.m_bindingMap.clone();
    }
    BindingMap::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu_resource::ResourceHandle;
    use crate::shader_module::ShaderModule;
    use crate::types::{PipelineDesc, VertexAttribute, VertexBufferLayout};

    #[test]
    fn pipeline_snapshot_owns_nested_descriptor_values_and_handles() {
        let module = ShaderModule::new().into_resource(None).erase();
        let attrs = [VertexAttribute {
            offset: 12,
            shaderSlot: 3,
            ..VertexAttribute::default()
        }];
        let layouts = [VertexBufferLayout {
            stride: 28,
            attributes: &attrs,
            ..VertexBufferLayout::default()
        }];
        let bind_group_layout = ResourceHandle::new(None, 17_u32).erase();
        let entry = String::from("vertex");
        let label = String::from("pipeline");
        let desc = PipelineDesc {
            vertexModule: Some(&module),
            vertexEntryPoint: Some(&entry),
            vertexBuffers: Some(&layouts),
            bindGroupLayouts: Some(&[Some(&bind_group_layout)]),
            label: Some(&label),
            ..PipelineDesc::default()
        };

        let pipeline = Pipeline::new(&desc);
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
        assert_eq!(module.debugging_ref_count(), 2);
        assert_eq!(bind_group_layout.debugging_ref_count(), 2);
    }

    #[test]
    fn pipeline_copies_vertex_binding_map_and_falls_back_to_fragment() {
        let mut vertex_module = ShaderModule::new();
        let vertex_blob = binding_map_blob(0, 1);
        assert!(BindingMap::from_blob(
            &vertex_blob,
            &mut vertex_module.m_bindingMap
        ));
        let expected_vertex = vertex_module.m_bindingMap.clone();
        let vertex = vertex_module.into_resource(None).erase();

        let mut fragment_module = ShaderModule::new();
        let fragment_blob = binding_map_blob(2, 3);
        assert!(BindingMap::from_blob(
            &fragment_blob,
            &mut fragment_module.m_bindingMap
        ));
        let expected_fragment = fragment_module.m_bindingMap.clone();
        let fragment = fragment_module.into_resource(None).erase();
        let desc = PipelineDesc {
            vertexModule: Some(&vertex),
            fragmentModule: Some(&fragment),
            ..PipelineDesc::default()
        };
        let pipeline = Pipeline::new(&desc);
        assert_eq!(pipeline.binding_map(), &expected_vertex);

        let fragment_only = PipelineDesc {
            fragmentModule: Some(&fragment),
            ..PipelineDesc::default()
        };
        let pipeline = Pipeline::new(&fragment_only);
        assert_eq!(pipeline.binding_map(), &expected_fragment);
    }

    #[test]
    fn pipeline_retains_only_the_first_four_layouts_once() {
        let layout0 = ResourceHandle::new(None, 0_u32).erase();
        let layout1 = ResourceHandle::new(None, 1_u32).erase();
        let layout2 = ResourceHandle::new(None, 2_u32).erase();
        let layout3 = ResourceHandle::new(None, 3_u32).erase();
        let layout4 = ResourceHandle::new(None, 4_u32).erase();
        let layouts = [
            Some(&layout0),
            Some(&layout1),
            Some(&layout2),
            Some(&layout3),
            Some(&layout4),
        ];
        let pipeline = Pipeline::new(&PipelineDesc {
            bindGroupLayouts: Some(&layouts),
            ..PipelineDesc::default()
        });

        assert_eq!(pipeline.desc().bindGroupLayoutCount, 5);
        assert!(pipeline.has_layout(0, &layout0));
        assert!(pipeline.has_layout(3, &layout3));
        assert!(pipeline.layout(4).is_none());
        assert_eq!(layout0.debugging_ref_count(), 2);
        assert_eq!(layout3.debugging_ref_count(), 2);
        assert_eq!(layout4.debugging_ref_count(), 1);
    }

    fn binding_map_blob(group: u8, binding: u8) -> Vec<u8> {
        let mut blob = vec![2, 1, 14, 0, 1, 0, 0, 0];
        blob.extend_from_slice(&[
            group, binding, 0, 1, 0, 7, 0, 0xff, 0xff, 0xff, 0xff, 0, 0, 0,
        ]);
        blob
    }
}
