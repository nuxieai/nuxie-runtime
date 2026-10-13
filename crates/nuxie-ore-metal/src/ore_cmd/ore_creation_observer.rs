//! Mechanical translation of renderer/ore/cmd/ore_creation_observer.hpp.
#![allow(non_snake_case)]
use crate::{gpu_resource::AnyResourceHandle, rstb_entry_container::RstbEntryView, types::*};
use std::{
    any::Any,
    sync::{Arc, Mutex},
};

/// Synchronous notifications on the recording thread. The asset argument is
/// the runtime's borrowed CoreHandle, erased only to avoid a renderer/runtime
/// dependency cycle; observers can downcast it to inspect that exact owner.
pub trait CreationObserver: Send + Sync {
    fn onShaderAsset(&self, _asset: &dyn Any, _entries: &[RstbEntryView<'_>]) {}
    fn onShaderModule(&self, _module: &AnyResourceHandle, _desc: &ShaderModuleDesc<'_>) {}
    fn onBindGroupLayout(&self, _layout: &AnyResourceHandle, _desc: &BindGroupLayoutDesc<'_>) {}
    fn onPipeline(&self, _pipeline: &AnyResourceHandle, _desc: &PipelineDesc<'_>) {}
    fn onBindGroup(&self, _group: &AnyResourceHandle, _desc: &BindGroupDesc<'_>) {}
}

// Safe Rust replacement for the process-global atomic observer pointer. The
// mutex provides publication ordering and Arc pins the observer across calls.
// Never invoke callbacks (or drop replaced observers) under this slot lock.
static CREATION_OBSERVER: Mutex<Option<Arc<dyn CreationObserver>>> = Mutex::new(None);

pub fn creationObserver() -> Option<Arc<dyn CreationObserver>> {
    CREATION_OBSERVER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

pub fn setCreationObserver(observer: Option<Arc<dyn CreationObserver>>) {
    let previous = {
        let mut slot = CREATION_OBSERVER
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        std::mem::replace(&mut *slot, observer)
    };
    drop(previous);
}
