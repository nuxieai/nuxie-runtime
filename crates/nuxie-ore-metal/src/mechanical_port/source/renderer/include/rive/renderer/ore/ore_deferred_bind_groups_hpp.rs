//! renderer/include/rive/renderer/ore/ore_deferred_bind_groups.hpp.
#![allow(non_snake_case)]
use crate::{
    gpu_resource::AnyResourceHandle, render_pass::RenderPassApi, script_guards::kMaxDynamicOffsets,
    types::kMaxBindGroups,
};

#[derive(Default)]
struct Entry {
    group: Option<AnyResourceHandle>,
    offsets: [u32; kMaxDynamicOffsets as usize],
    offsetCount: u32,
}

/// Bindings made before the first pipeline wait for its native layout.
#[derive(Default)]
pub struct DeferredBindGroups {
    entries: [Entry; kMaxBindGroups as usize],
}

impl Drop for DeferredBindGroups {
    fn drop(&mut self) {
        // C++ destroys array elements from last to first, including groups
        // still waiting for a pipeline when the script pass is destroyed.
        for entry in self.entries.iter_mut().rev() {
            entry.group = None;
        }
    }
}

impl DeferredBindGroups {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn defer(
        &mut self,
        groupIndex: u32,
        group: Option<&AnyResourceHandle>,
        offsets: Option<&[u32]>,
        offsetCount: u32,
    ) {
        assert!(groupIndex < kMaxBindGroups && offsetCount <= kMaxDynamicOffsets);
        let entry = &mut self.entries[groupIndex as usize];
        entry.group = group.cloned();
        entry.offsetCount = offsetCount;
        if offsetCount != 0 {
            entry.offsets[..offsetCount as usize]
                .copy_from_slice(&offsets.expect("dynamic offsets")[..offsetCount as usize]);
        }
    }

    pub fn flush(&mut self, pass: &mut dyn RenderPassApi) {
        for (i, entry) in self.entries.iter_mut().enumerate() {
            if entry.group.is_some() {
                pass.setBindGroup(
                    i as u32,
                    entry.group.as_ref(),
                    (entry.offsetCount != 0).then_some(&entry.offsets[..]),
                    entry.offsetCount,
                );
                entry.group = None;
            }
        }
    }
}
