//! Draw binding validation translated from upstream 8920a6b.
#![allow(non_snake_case)]
use crate::{
    bind_group_layout::BindGroupLayout, gpu_resource::AnyResourceHandle, types::kMaxBindGroups,
};
pub fn bindGroupLayoutsMatch(a: &BindGroupLayout, b: &BindGroupLayout) -> bool {
    if std::ptr::eq(a, b) {
        return true;
    }
    if a.groupIndex() != b.groupIndex() || a.entries().len() != b.entries().len() {
        return false;
    }
    a.entries().iter().all(|e| {
        b.findEntry(e.binding).is_some_and(|other| {
            // repr(C) entries have explicit initialized padding, matching upstream memcmp.
            unsafe {
                std::slice::from_raw_parts(
                    std::ptr::from_ref(e).cast::<u8>(),
                    std::mem::size_of_val(e),
                ) == std::slice::from_raw_parts(
                    std::ptr::from_ref(other).cast::<u8>(),
                    std::mem::size_of_val(other),
                )
            }
        })
    })
}
#[derive(Default)]
pub struct DrawBindings {
    pipeline: Option<AnyResourceHandle>,
    groups: [Option<AnyResourceHandle>; kMaxBindGroups as usize],
}
impl DrawBindings {
    pub fn setPipeline(&mut self, pipeline: Option<&AnyResourceHandle>) {
        self.pipeline = pipeline.cloned();
    }
    pub fn setBindGroup(&mut self, index: u32, group: Option<&AnyResourceHandle>) {
        if let Some(slot) = self.groups.get_mut(index as usize) {
            *slot = group.cloned();
        }
    }
    pub fn validate(&self, mut outError: Option<&mut String>) -> bool {
        let Some(pipeline) = self
            .pipeline
            .as_ref()
            .and_then(AnyResourceHandle::pipelineBase)
        else {
            return true;
        };
        for g in 0..kMaxBindGroups {
            let Some(layout) = pipeline
                .layout(g)
                .and_then(AnyResourceHandle::bindGroupLayoutBase)
            else {
                continue;
            };
            if layout.entries().is_empty() {
                continue;
            }
            let group = self.groups[g as usize]
                .as_ref()
                .and_then(AnyResourceHandle::bindGroupBase);
            if group.is_none() && !pipeline.m_entryUseKnown {
                continue;
            }
            let problem = match group {
                None => Some("the pipeline binds this group but none is set"),
                Some(group)
                    if group
                        .layout()
                        .and_then(AnyResourceHandle::bindGroupLayoutBase)
                        .is_none_or(|b| !bindGroupLayoutsMatch(b, layout)) =>
                {
                    Some(
                        "the bind group's layout does not match the pipeline's; build it from this pipeline's getBindGroupLayout, or share layouts explicitly",
                    )
                }
                _ => None,
            };
            if let Some(problem) = problem {
                if let Some(error) = outError.as_mut() {
                    **error = format!("@group({g}): {problem}");
                }
                return false;
            }
        }
        true
    }
}
