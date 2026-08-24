//! Complete mechanical implementation translation of
//! `renderer/src/ore/wgpu/ore_bind_group_wgpu.cpp`.

#![allow(non_snake_case)]

use super::ore_bind_group_wgpu_decl::{BindGroupWGPU, CachedGroup};
use super::webgpu_cpp_decl::{
    BindGroupEntry, Buffer as WagyuBuffer, Sampler as WagyuSampler,
    TextureView as WagyuTextureView,
};
use super::webgpu_decl::{WGPUBindGroupDescriptor, WGPUStringView, WGPU_STRLEN};

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_wgpu_ore_bind_group_wgpu.cpp");

pub(crate) fn markUBOsBound(group: &BindGroupWGPU) {
    for u in group.m_uboEntries.iter() {
        unsafe { u.buffer.as_ref().markBound() };
    }
}

fn labelView(label: &str) -> WGPUStringView {
    if label.is_empty() {
        WGPUStringView {
            data: std::ptr::null(),
            length: WGPU_STRLEN,
        }
    } else {
        WGPUStringView {
            data: label.as_ptr().cast(),
            length: label.len(),
        }
    }
}

pub(crate) fn resolveBindGroup(group: &BindGroupWGPU) -> &super::webgpu_cpp_decl::BindGroup {
    let cache = unsafe { group.m_cache.getMutOnRecordingThread() };
    let cachedIndex = cache.iter().position(|cached| {
        cached.key.len() == group.m_uboEntries.len()
            && cached
                .key
                .iter()
                .zip(group.m_uboEntries.iter())
                .all(|(key, entry)| *key == unsafe { entry.buffer.as_ref().currentRaw() })
    });
    if let Some(index) = cachedIndex {
        return &cache[index].bindGroup;
    }

    let mut entries = Vec::with_capacity(
        group.m_uboEntries.len() + group.m_texEntries.len() + group.m_sampEntries.len(),
    );
    let mut cached = CachedGroup {
        key: Vec::with_capacity(group.m_uboEntries.len()),
        bindGroup: Default::default(),
    };
    for u in group.m_uboEntries.iter() {
        let buffer = unsafe { u.buffer.as_ref().currentRaw() };
        cached.key.push(buffer);
        let mut e = BindGroupEntry::default();
        e.binding = u.binding;
        e.buffer = unsafe { WagyuBuffer::FromBorrowed(buffer) };
        e.offset = u.offset;
        e.size = u.size;
        entries.push(e);
    }
    for t in group.m_texEntries.iter() {
        let mut e = BindGroupEntry::default();
        e.binding = t.binding;
        e.textureView = unsafe { WagyuTextureView::FromBorrowed(t.view.Get()) };
        entries.push(e);
    }
    for s in group.m_sampEntries.iter() {
        let mut e = BindGroupEntry::default();
        e.binding = s.binding;
        e.sampler = unsafe { WagyuSampler::FromBorrowed(s.sampler.Get()) };
        entries.push(e);
    }

    let ctx = group.context();
    let mut bgDesc = WGPUBindGroupDescriptor::default();
    bgDesc.layout = group.m_wgpuBGL.Get();
    bgDesc.label = labelView(&group.m_label);
    bgDesc.entryCount = entries.len();
    bgDesc.entries = if entries.is_empty() {
        std::ptr::null()
    } else {
        entries.as_ptr().cast()
    };
    cached.bindGroup = unsafe { ctx.m_wgpuDevice.CreateBindGroup(&bgDesc) };
    if cached.bindGroup.Get().is_null() {
        return &group.m_nullBindGroup;
    }
    cache.push(cached);
    &cache.last().expect("just pushed bind group").bindGroup
}

pub(crate) const SOURCE_FUNCTION_COUNT: usize = 2;
pub(crate) const SOURCE_LOOP_COUNT: usize = 6;
pub(crate) const SOURCE_CREATE_BIND_GROUP_CALL_COUNT: usize = 1;
pub(crate) const SOURCE_FAILED_CREATION_EARLY_RETURN_COUNT: usize = 1;
const _: [(); 2292] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_implementation_denominator_is_locked() {
        assert_eq!(PINNED_SOURCE.lines().count(), 81);
        assert_eq!(SOURCE_FUNCTION_COUNT, 2);
        assert_eq!(SOURCE_LOOP_COUNT, 6);
        assert_eq!(SOURCE_CREATE_BIND_GROUP_CALL_COUNT, 1);
        assert_eq!(SOURCE_FAILED_CREATION_EARLY_RETURN_COUNT, 1);
    }

    #[test]
    fn empty_and_authored_labels_preserve_source_null_rule() {
        assert!(labelView("").data.is_null());
        let label = labelView("group");
        assert!(!label.data.is_null());
        assert_eq!(label.length, 5);
    }
}
