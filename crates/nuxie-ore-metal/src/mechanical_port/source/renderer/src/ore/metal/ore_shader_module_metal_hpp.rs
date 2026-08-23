/*
 * Copyright 2025 Rive
 */

// #pragma once
// #include "rive/renderer/ore/ore_shader_module.hpp"
// #import <Metal/Metal.h>

// Mechanical translation of the complete pinned source header
// renderer/src/ore/metal/ore_shader_module_metal.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use super::*;
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_resource_hpp::{
    GPUResource, GpuResourcePayload,
};
use std::mem::ManuallyDrop;

// `id<MTLLibrary>` is a nullable, strong Objective-C owner under ARC. Rust's
// `Retained<T>` is the corresponding strong owner; `Option` preserves the
// source `nil` state while the library is being constructed. The non-Apple
// stand-in keeps this source-shaped translation available to tools that
// inspect it off Apple.
#[cfg(target_vendor = "apple")]
use objc2::rc::Retained;
#[cfg(target_vendor = "apple")]
use objc2::runtime::ProtocolObject;
#[cfg(target_vendor = "apple")]
use objc2_metal::MTLLibrary;

#[cfg(target_vendor = "apple")]
type NativeMetalLibrary = Option<Retained<ProtocolObject<dyn MTLLibrary>>>;

#[cfg(not(target_vendor = "apple"))]
type NativeMetalLibrary = Option<()>;

// namespace rive::ore

// class ContextMetal;
// The source forward declaration is retained for the friend relationship
// below. ContextMetal is owned by its own translation unit.

// class ShaderModuleMetal
//     : public LITE_RTTI_OVERRIDE(ShaderModule, ShaderModuleMetal)
// {
// Rust has no class inheritance. `base` is the first field to preserve the
// source ShaderModule base-subobject order. `LITE_RTTI_OVERRIDE(ShaderModule,
// ShaderModuleMetal)` remains the source lite-RTTI identity/override seam and
// is not duplicated as a payload field.
#[repr(C)]
pub struct ShaderModuleMetal {
    pub(crate) base: ManuallyDrop<ShaderModule>,
    // private:
    // friend class ContextMetal;
    // Rust has no friend declarations; this source access boundary remains
    // visible here, and the owning translation unit uses crate visibility.
    // id<MTLLibrary> m_mtlLibrary = nil;
    // `NativeMetalLibrary` retains the non-nil Objective-C library until the
    // enclosing logical ShaderModuleMetal owner is dropped.
    pub(crate) m_mtlLibrary: ManuallyDrop<NativeMetalLibrary>,
}

// SAFETY: MTLLibrary and the portable reflection data are immutable after
// publication; completion-thread final release may drop their retains.
unsafe impl Send for ShaderModuleMetal {}

unsafe impl GpuResourcePayload for ShaderModuleMetal {
    fn gpu_resource(&self) -> &GPUResource {
        &self.base.base
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        &mut self.base.base
    }
}

impl ShaderModuleMetal {
    // public:

    // ShaderModuleMetal() = default;
    pub(crate) fn new() -> Self {
        Self {
            base: ManuallyDrop::new(ShaderModule::new()),
            m_mtlLibrary: ManuallyDrop::new(None),
        }
    }

    /// Product inspection seam over the source friend-owned native library.
    /// The returned borrow cannot outlive this exact ShaderModuleMetal owner.
    #[cfg(target_vendor = "apple")]
    pub fn mtlLibrary(&self) -> Option<&ProtocolObject<dyn objc2_metal::MTLLibrary>> {
        self.m_mtlLibrary.as_deref()
    }

    /// Public inherited-source view of ShaderModule::m_bindingMap.
    pub fn bindingMap(&self) -> &BindingMap {
        &self.base.m_bindingMap
    }

    // ~ShaderModuleMetal() override = default; // ARC releases m_mtlLibrary
    // Rust's default drop glue releases the retained native library owner
    // before the remaining source-shaped fields.
}

impl Drop for ShaderModuleMetal {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_mtlLibrary);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

// } // namespace rive::ore
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shader_module_metal_starts_without_a_published_library() {
        let module = ShaderModuleMetal::new();
        assert!(module.base.m_bindingMap.empty());
        let handle = crate::gpu_resource::ResourceHandle::new(None, module);
        assert_eq!(handle.debugging_refcnt(), 1);
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn nil_compilation_publishes_no_module_and_live_library_is_retained() {
        use objc2_foundation::NSString;
        use objc2_metal::{MTLCreateSystemDefaultDevice, MTLDevice};

        let empty = ShaderModuleMetal::new();
        assert!(empty.mtlLibrary().is_none());
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
        let mut module = ShaderModuleMetal::new();
        module.m_mtlLibrary = ManuallyDrop::new(Some(library));
        assert!(module.mtlLibrary().is_some());
    }
}
