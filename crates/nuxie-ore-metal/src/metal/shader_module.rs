// Mechanical translation of:
// - renderer/src/ore/metal/ore_shader_module_metal.hpp
// - renderer/src/ore/metal/ore_shader_module_metal.mm
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
//
// Copyright 2025 Rive

#![allow(non_snake_case)]

use std::any::Any;
use std::ops::{Deref, DerefMut};

use crate::gpu_resource::{GpuResourceManager, ResourceHandle};
use crate::shader_module::ShaderModule;
use crate::types::{BackendId, ShaderModule as ShaderModuleResource};

use super::MetalBackend;

#[cfg(target_vendor = "apple")]
use objc2::rc::Retained;
#[cfg(target_vendor = "apple")]
use objc2::runtime::ProtocolObject;
#[cfg(target_vendor = "apple")]
use objc2_metal::MTLLibrary;

#[cfg(target_vendor = "apple")]
struct RetainedMetalLibrary(Retained<ProtocolObject<dyn MTLLibrary>>);

// SAFETY: MTLLibrary is immutable after compilation and supports concurrent
// retain/release and function lookup. The wrapper exposes shared access only.
#[cfg(target_vendor = "apple")]
unsafe impl Send for RetainedMetalLibrary {}
// SAFETY: Same invariant as the `Send` implementation above.
#[cfg(target_vendor = "apple")]
unsafe impl Sync for RetainedMetalLibrary {}

/// Metal-specific shader module with the exact retained MTLLibrary owner.
pub struct ShaderModuleMetal {
    shader_module: ShaderModule,
    #[cfg(target_vendor = "apple")]
    m_mtlLibrary: Option<RetainedMetalLibrary>,
}

impl ShaderModuleMetal {
    /// Default native library is nil until ContextMetal publishes a successful
    /// compilation result.
    pub fn new() -> Self {
        Self {
            shader_module: ShaderModule::new(),
            #[cfg(target_vendor = "apple")]
            m_mtlLibrary: None,
        }
    }

    #[cfg(target_vendor = "apple")]
    pub fn from_compiled_library(
        library: Option<Retained<ProtocolObject<dyn MTLLibrary>>>,
    ) -> Option<Self> {
        library.map(|library| Self {
            shader_module: ShaderModule::new(),
            m_mtlLibrary: Some(RetainedMetalLibrary(library)),
        })
    }

    #[cfg(target_vendor = "apple")]
    pub fn mtlLibrary(&self) -> Option<&ProtocolObject<dyn MTLLibrary>> {
        self.m_mtlLibrary.as_ref().map(|library| library.0.as_ref())
    }

    pub fn into_resource(self, manager: Option<GpuResourceManager>) -> ResourceHandle<Self> {
        ResourceHandle::new(manager, self)
    }
}

impl Default for ShaderModuleMetal {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for ShaderModuleMetal {
    type Target = ShaderModule;

    fn deref(&self) -> &Self::Target {
        &self.shader_module
    }
}

impl DerefMut for ShaderModuleMetal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.shader_module
    }
}

impl ShaderModuleResource for ShaderModuleMetal {
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

    #[test]
    fn shader_module_metal_starts_without_a_published_library() {
        let module = ShaderModuleMetal::new();
        assert!(module.m_bindingMap.is_empty());
        let handle = module.into_resource(None);
        assert_eq!(handle.debugging_ref_count(), 1);
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn nil_compilation_publishes_no_module_and_live_library_is_retained() {
        use objc2_foundation::NSString;
        use objc2_metal::{MTLCreateSystemDefaultDevice, MTLDevice};

        assert!(ShaderModuleMetal::from_compiled_library(None).is_none());
        let Some(device) = MTLCreateSystemDefaultDevice() else {
            crate::live_metal_test_unavailable("system Metal device");
            return;
        };
        let source = NSString::from_str(
            "#include <metal_stdlib>\nusing namespace metal;\nkernel void ownership_probe() {}",
        );
        let library = device
            .newLibraryWithSource_options_error(&source, None)
            .expect("compile minimal shader-module ownership library");
        let module = ShaderModuleMetal::from_compiled_library(Some(library))
            .expect("non-nil compilation result publishes module");
        assert!(module.mtlLibrary().is_some());
    }
}
