// Mechanical translation of:
//   renderer/include/rive/renderer/ore/ore_bind_group.hpp
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
//
// Copyright 2025 Rive

#![allow(non_snake_case)]
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the pending context/render-pass unit constructs and consumes bind groups"
    )
)]

use crate::bind_group_layout::BindGroupLayout;
use crate::gpu_resource::AnyResourceHandle;

/// Pre-baked resource bindings that can be reused across draw calls.
///
/// C++ derives this object from `GPUResource`. Rust stores the payload here
/// and puts the manager/refcount owner in
/// [`ResourceHandle`](crate::gpu_resource::ResourceHandle). The vectors are
/// the one logical strong-owner graph for accepted bindings. Metal records
/// refer back to these vectors by index instead of retaining the same resource
/// a second time merely to replace the C++ raw `BufferMetal*` pointer.
pub struct BindGroup {
    m_retainedSamplers: Vec<AnyResourceHandle>,
    m_retainedViews: Vec<AnyResourceHandle>,
    m_retainedBuffers: Vec<AnyResourceHandle>,
    m_layoutRef: Option<AnyResourceHandle>,
    m_dynamicOffsetCount: u32,
    // The pinned header stores a non-owning Context* for a deferred Lua-GC
    // destruction route. There is no safe Rust Context/ContextMetal owner
    // topology yet, so this dead back-pointer is deliberately not recreated.
    // The pending context unit must establish that token/defer seam before a
    // bind group can be published through the public context API.
}

impl BindGroup {
    /// Translate the protected C++ payload construction.
    ///
    /// The context factory remains responsible for null-layout rejection,
    /// backend checked-downcasts, slot resolution, and deciding which input
    /// entries are accepted. This constructor only adopts the already accepted
    /// strong handles and scalar state.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the pending ContextMetal unit publishes accepted bindings"
        )
    )]
    pub(crate) fn from_parts(
        dynamic_offset_count: u32,
        layout: Option<AnyResourceHandle>,
        retained_buffers: Vec<AnyResourceHandle>,
        retained_views: Vec<AnyResourceHandle>,
        retained_samplers: Vec<AnyResourceHandle>,
    ) -> Self {
        Self {
            m_retainedSamplers: retained_samplers,
            m_retainedViews: retained_views,
            m_retainedBuffers: retained_buffers,
            m_layoutRef: layout,
            m_dynamicOffsetCount: dynamic_offset_count,
        }
    }

    pub fn dynamic_offset_count(&self) -> u32 {
        self.m_dynamicOffsetCount
    }

    /// C++ spelling retained for source-corresponding callers.
    pub fn dynamicOffsetCount(&self) -> u32 {
        self.dynamic_offset_count()
    }

    pub fn group_index(&self) -> u32 {
        self.m_layoutRef
            .as_ref()
            .and_then(|layout| layout.downcast_ref::<BindGroupLayout>())
            .map_or(0, BindGroupLayout::group_index)
    }

    /// C++ spelling retained for source-corresponding callers.
    pub fn groupIndex(&self) -> u32 {
        self.group_index()
    }

    pub fn layout(&self) -> Option<&AnyResourceHandle> {
        self.m_layoutRef.as_ref()
    }

    pub(crate) fn retained_buffer(&self, index: usize) -> Option<&AnyResourceHandle> {
        self.m_retainedBuffers.get(index)
    }

    pub(crate) fn retained_view(&self, index: usize) -> Option<&AnyResourceHandle> {
        self.m_retainedViews.get(index)
    }

    pub(crate) fn retained_sampler(&self, index: usize) -> Option<&AnyResourceHandle> {
        self.m_retainedSamplers.get(index)
    }

    pub(crate) fn retained_buffer_count(&self) -> usize {
        self.m_retainedBuffers.len()
    }

    pub(crate) fn retained_view_count(&self) -> usize {
        self.m_retainedViews.len()
    }

    pub(crate) fn retained_sampler_count(&self) -> usize {
        self.m_retainedSamplers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu_resource::ResourceHandle;
    use crate::types::{BindGroupLayoutEntry, BindingKind, StageVisibility};
    use std::sync::{Arc, Mutex};

    #[test]
    fn bind_group_adopts_each_strong_resource_once_and_reports_layout_identity() {
        let layout = ResourceHandle::new(
            None,
            BindGroupLayout::from_context_entries(
                2,
                &[BindGroupLayoutEntry {
                    binding: 4,
                    kind: BindingKind::uniformBuffer,
                    visibility: StageVisibility::default(),
                    ..BindGroupLayoutEntry::default()
                }],
            ),
        )
        .erase();
        let buffer = ResourceHandle::new(None, 11_u32).erase();
        let view = ResourceHandle::new(None, 12_u32).erase();
        let sampler = ResourceHandle::new(None, 13_u32).erase();

        let group = BindGroup::from_parts(
            1,
            Some(layout.clone()),
            vec![buffer.clone()],
            vec![view.clone()],
            vec![sampler.clone()],
        );

        assert_eq!(group.dynamicOffsetCount(), 1);
        assert_eq!(group.groupIndex(), 2);
        assert!(group.layout().is_some_and(|value| value.ptr_eq(&layout)));
        assert!(
            group
                .retained_buffer(0)
                .is_some_and(|value| value.ptr_eq(&buffer))
        );
        assert!(
            group
                .retained_view(0)
                .is_some_and(|value| value.ptr_eq(&view))
        );
        assert!(
            group
                .retained_sampler(0)
                .is_some_and(|value| value.ptr_eq(&sampler))
        );
        assert_eq!(layout.debugging_ref_count(), 2);
        assert_eq!(buffer.debugging_ref_count(), 2);
        assert_eq!(view.debugging_ref_count(), 2);
        assert_eq!(sampler.debugging_ref_count(), 2);
        assert_eq!(group.retained_buffer_count(), 1);
        assert_eq!(group.retained_view_count(), 1);
        assert_eq!(group.retained_sampler_count(), 1);
    }

    #[test]
    fn missing_layout_preserves_cxx_zero_group_fallback() {
        let group = BindGroup::from_parts(0, None, vec![], vec![], vec![]);
        assert_eq!(group.group_index(), 0);
        assert!(group.layout().is_none());
    }

    #[test]
    fn logical_resources_drop_in_cxx_reverse_member_order() {
        struct DropTag {
            tag: &'static str,
            order: Arc<Mutex<Vec<&'static str>>>,
        }

        impl Drop for DropTag {
            fn drop(&mut self) {
                self.order
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(self.tag);
            }
        }

        let order = Arc::new(Mutex::new(Vec::new()));
        let owner = |tag| {
            ResourceHandle::new(
                None,
                DropTag {
                    tag,
                    order: Arc::clone(&order),
                },
            )
            .erase()
        };
        let group = BindGroup::from_parts(
            0,
            Some(owner("layout")),
            vec![owner("buffer")],
            vec![owner("view")],
            vec![owner("sampler")],
        );

        drop(group);

        assert_eq!(
            *order
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            ["sampler", "view", "buffer", "layout"]
        );
    }
}
