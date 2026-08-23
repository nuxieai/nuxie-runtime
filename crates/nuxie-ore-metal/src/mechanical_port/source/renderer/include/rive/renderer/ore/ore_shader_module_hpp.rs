/*
 * Copyright 2025 Rive
 */

// #pragma once

// #include "rive/renderer/gpu_resource.hpp"
// #include "utils/lite_rtti.hpp"
// #include "rive/renderer/ore/ore_binding_map.hpp"
// #include "rive/renderer/ore/ore_types.hpp"

// #include <cassert>
// #include <cstdint>
// #include <string>
// #include <vector>

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/ore/ore_shader_module.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use super::ore_binding_map_hpp::BindingMap;
use super::ore_types_hpp::ShaderModuleDesc;
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};

use super::super::gpu_resource_hpp::{GPUResource, GpuResourcePayload};

// `ShaderModuleDesc<'_>` is the sibling source-shaped snapshot from
// ore_types_hpp.rs. Its checked `bindingMapSize()` / `glFixupSize()` accessors
// stand in for the C++ descriptor's authored size fields without widening
// either borrowed byte-span lifetime.

// namespace rive::ore
// {

// class Context;
// The source forward declaration is retained for the friend relationships
// below. Context is owned by its own translation unit.

// class ShaderModule : public rive::gpu::GPUResource,
//                      public ENABLE_LITE_RTTI(ShaderModule)
//
// Rust has no class inheritance. The first field is the GPUResource base
// subobject, preserving source base-before-members layout and destruction
// order. The second source base, `ENABLE_LITE_RTTI(ShaderModule)`, remains a
// source-visible RTTI/downcast contract rather than a duplicate payload field.
// Lifetime authority: the module owns its reflection vectors, binding map,
// fixup names, and conditional asset id until the concrete backend module is
// released; Pipeline copies the binding map from its stage modules at build.

#[repr(C)]
pub struct ShaderModuleMembers {
    // public:
    /// Texture-sampler pair from RSTB shader reflection.
    /// Records which texture binding is paired with which sampler binding
    /// in the shader's sampling expressions. Used by the GL backend to bind
    /// sampler objects to the correct texture unit.
    // struct TextureSamplerPair
    // std::vector<TextureSamplerPair> m_textureSamplerPairs;
    pub m_textureSamplerPairs: Vec<TextureSamplerPair>,

    // (group, binding) -> per-backend native slot map for this shader's
    // resources. Parsed from the RSTB binding-map sidecar by each
    // backend's `makeShaderModule` (via `applyBindingMapFromDesc`
    // below). Consumed by `Pipeline`, which copies it from its vertex /
    // fragment modules at construction.
    // BindingMap m_bindingMap;
    pub m_bindingMap: BindingMap,

    // #ifdef TRACK_RIVE_SHADER_ID
    #[cfg(feature = "track-rive-shader-id")]
    // uint32_t m_shaderAssetId = 0;
    pub m_shaderAssetId: u32,
    // uint32_t shaderAssetId() const { return m_shaderAssetId; }
    // #endif

    // Parsed fixup table. Populated from the stage's RSTB sidecar (target
    // 14 = VS, target 15 = FS). Empty for non-GL backends, where these
    // sidecars are not part of the variant set.
    // `oreGLFixupProgramBindings` iterates both stages' tables at
    // `glLinkProgram` time.
    // std::vector<GLFixupEntry> m_glFixup;
    pub m_glFixup: Vec<GLFixupEntry>,
}

#[repr(C)]
pub struct ShaderModule {
    pub(crate) base: ManuallyDrop<GPUResource>,
    pub(crate) members: ManuallyDrop<ShaderModuleMembers>,
}

impl Deref for ShaderModule {
    type Target = ShaderModuleMembers;

    fn deref(&self) -> &Self::Target {
        &self.members
    }
}

impl DerefMut for ShaderModule {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.members
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            // Exact reverse authored member destruction before GPUResource.
            #[cfg(test)]
            super::super::gpu_resource_hpp::record_resource_drop_stage("ShaderModule.glFixup");
            core::ptr::drop_in_place(&mut self.m_glFixup);
            #[cfg(test)]
            super::super::gpu_resource_hpp::record_resource_drop_stage("ShaderModule.bindingMap");
            core::ptr::drop_in_place(&mut self.m_bindingMap);
            #[cfg(test)]
            super::super::gpu_resource_hpp::record_resource_drop_stage(
                "ShaderModule.textureSamplerPairs",
            );
            core::ptr::drop_in_place(&mut self.m_textureSamplerPairs);
            #[cfg(test)]
            super::super::gpu_resource_hpp::record_resource_drop_stage("ShaderModule.base");
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

unsafe impl GpuResourcePayload for ShaderModule {
    fn gpu_resource(&self) -> &GPUResource {
        &self.base
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        &mut self.base
    }
}

// The following records are declared inside ShaderModule in the C++ source.
// They remain source-shaped sibling records because Rust does not permit
// nested struct/enum declarations in a struct body.

/// Texture-sampler pair from the nested C++ `ShaderModule` declaration.
///
/// `repr(C)` preserves the four-byte source value layout. The four fields are
/// owned values copied from shader reflection and remain in source order.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextureSamplerPair {
    // uint8_t textureGroup;
    pub textureGroup: u8,
    // uint8_t textureBinding;
    pub textureBinding: u8,
    // uint8_t samplerGroup;
    pub samplerGroup: u8,
    // uint8_t samplerBinding;
    pub samplerBinding: u8,
}

// C++ nested `GLFixupEntry::Kind`. A transparent byte retains the
// static_cast behavior of the pinned parser for unknown forward-compatible
// discriminants while retaining the two defined values.
// enum class Kind : uint8_t
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GLFixupKind(pub u8);

impl GLFixupKind {
    pub const UBOBlock: Self = Self(0);
    pub const SamplerUniform: Self = Self(1);
}

// C++ nested-name alias retained for source-corresponding callers.
pub type Kind = GLFixupKind;

// One entry in the GL program-link fixup table: tells the runtime
// which GL binding point / texture unit each named uniform should
// land on, letting `oreGLFixupProgramBindings` call
// `glUniformBlockBinding` / `glUniform1i` without parsing the
// emitted GLSL names at runtime.
// struct GLFixupEntry
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GLFixupEntry {
    // Kind kind;
    pub kind: GLFixupKind,
    // uint8_t slot;
    pub slot: u8,
    // std::string name;
    // C++ std::string is a byte string; Vec<u8> preserves embedded NULs,
    // arbitrary non-UTF-8 names, and the source-owned value semantics.
    pub name: Vec<u8>,
}

impl ShaderModule {
    // public:

    // uint32_t shaderAssetId() const { return m_shaderAssetId; }
    // #ifdef TRACK_RIVE_SHADER_ID
    #[cfg(feature = "track-rive-shader-id")]
    pub fn shaderAssetId(&self) -> u32 {
        self.m_shaderAssetId
    }
    // #endif

    // Helper for backend `makeShaderModule` paths: parse the binding-map
    // sidecar bytes (`desc.bindingMapBytes`) into `m_bindingMap`. The
    // sidecar is mandatory; runtime backends rely on it to translate
    // `@group/@binding` to native slots.
    // void applyBindingMapFromDesc(const ShaderModuleDesc& desc)
    pub fn applyBindingMapFromDesc(&mut self, desc: &ShaderModuleDesc<'_>) {
        debug_assert!(
            desc.bindingMapSize().is_ok(),
            "ShaderModuleDesc::bindingMapSize exceeds its backing span"
        );
        let binding_map_size = desc.bindingMapSize;
        debug_assert!(
            desc.bindingMapBytes.is_some() && binding_map_size > 0,
            "ShaderModuleDesc::bindingMapBytes is required"
        );
        let ok = BindingMap::fromBlob(
            desc.bindingMapBytes,
            binding_map_size as usize,
            Some(&mut self.m_bindingMap),
        );
        debug_assert!(ok, "binding-map sidecar failed to parse");
        let _ = ok;
        self.applyGLFixupFromDesc(desc);
        // #ifdef TRACK_RIVE_SHADER_ID
        #[cfg(feature = "track-rive-shader-id")]
        {
            self.m_shaderAssetId = desc.shaderAssetId;
        }
        // #endif
    }

    // One entry in the GL program-link fixup table: tells the runtime
    // which GL binding point / texture unit each named uniform should
    // land on, letting `oreGLFixupProgramBindings` call
    // `glUniformBlockBinding` / `glUniform1i` without parsing the
    // emitted GLSL names at runtime.

    // Helper: parse `desc.glFixupBytes` (RSTB GL fixup blob format)
    // into `m_glFixup`. No-op when the sidecar is absent or malformed.
    // void applyGLFixupFromDesc(const ShaderModuleDesc& desc)
    pub fn applyGLFixupFromDesc(&mut self, desc: &ShaderModuleDesc<'_>) {
        let Some(bytes) = desc.glFixupBytes else {
            return;
        };
        let Ok(gl_fixup_size) = desc.glFixupSize() else {
            return;
        };
        if gl_fixup_size < 3 {
            return;
        }
        let Some(bytes) = bytes.get(..gl_fixup_size as usize) else {
            return;
        };
        let mut p = bytes;
        if p[0] != 1 {
            // version
            return;
        }
        p = &p[1..];
        let count = u16::from_le_bytes([p[0], p[1]]);
        p = &p[2..];
        self.m_glFixup.reserve(usize::from(count));
        for _ in 0..count {
            if p.len() < 4 {
                return;
            }
            let kind = GLFixupKind(p[0]);
            let slot = p[1];
            let name_len = usize::from(u16::from_le_bytes([p[2], p[3]]));
            p = &p[4..];
            if p.len() < name_len {
                return;
            }
            let name = p[..name_len].to_vec();
            p = &p[name_len..];
            self.m_glFixup.push(GLFixupEntry { kind, slot, name });
        }
    }

    // virtual ~ShaderModule() = default;
    // Rust's default drop glue supplies the virtual-destructor boundary for
    // the concrete resource owner; no extra state is introduced here.

    // protected:
    // friend class Context;
    // friend class Pipeline;
    // Rust has no friend declarations; these source access boundaries remain
    // visible here, and the owning translation units use crate visibility.

    // ShaderModule() : rive::gpu::GPUResource(nullptr) {}
    pub(crate) fn new() -> Self {
        Self {
            base: ManuallyDrop::new(GPUResource::new(None)),
            members: ManuallyDrop::new(ShaderModuleMembers {
                m_textureSamplerPairs: Vec::new(),
                m_bindingMap: BindingMap::default(),
                #[cfg(feature = "track-rive-shader-id")]
                m_shaderAssetId: 0,
                m_glFixup: Vec::new(),
            }),
        }
    }

    // ShaderModule(rcp<rive::gpu::GPUResourceManager> manager) :
    //     rive::gpu::GPUResource(std::move(manager))
    // {}
    // Manager ownership is carried by the concrete outer ResourceHandle.
}

// } // namespace rive::ore
#[cfg(test)]
mod tests {
    use super::*;

    fn valid_binding_map() -> Vec<u8> {
        // version=2, allocator=1, entry size=14, entry count=0
        vec![2, 1, 14, 0, 0, 0, 0, 0]
    }

    fn fixup_blob(records: &[(u8, u8, &[u8])]) -> Vec<u8> {
        let mut blob = vec![1, records.len() as u8, 0];
        for &(kind, slot, name) in records {
            blob.push(kind);
            blob.push(slot);
            blob.extend_from_slice(&(name.len() as u16).to_le_bytes());
            blob.extend_from_slice(name);
        }
        blob
    }

    #[test]
    fn shader_module_applies_binding_map_and_gl_fixup() {
        let binding_map = valid_binding_map();
        let fixup = fixup_blob(&[(0, 3, b"Globals"), (1, 5, b"tex")]);
        let desc = ShaderModuleDesc {
            bindingMapBytes: Some(&binding_map),
            bindingMapSize: binding_map.len() as u32,
            glFixupBytes: Some(&fixup),
            glFixupSize: fixup.len() as u32,
            ..ShaderModuleDesc::default()
        };
        let mut module = ShaderModule::new();
        module.applyBindingMapFromDesc(&desc);
        assert!(module.m_bindingMap.empty());
        assert_eq!(module.m_glFixup.len(), 2);
        assert_eq!(module.m_glFixup[0].kind, GLFixupKind::UBOBlock);
        assert_eq!(module.m_glFixup[0].slot, 3);
        assert_eq!(module.m_glFixup[0].name, b"Globals");
        assert_eq!(module.m_glFixup[1].kind, GLFixupKind::SamplerUniform);
    }

    #[test]
    fn shader_module_gl_fixup_absent_and_wrong_version_are_no_ops() {
        let mut module = ShaderModule::new();
        let absent = ShaderModuleDesc::default();
        module.applyGLFixupFromDesc(&absent);
        module.applyGLFixupFromDesc(&ShaderModuleDesc {
            glFixupBytes: Some(&[2, 0, 0]),
            glFixupSize: 3,
            ..ShaderModuleDesc::default()
        });
        assert!(module.m_glFixup.is_empty());
    }

    #[test]
    fn shader_module_gl_fixup_truncation_preserves_prefix_and_repeated_calls_append() {
        let first = fixup_blob(&[(0, 2, b"first"), (1, 4, b"second")]);
        let mut module = ShaderModule::new();
        module.applyGLFixupFromDesc(&ShaderModuleDesc {
            glFixupBytes: Some(&first[..first.len() - 1]),
            glFixupSize: (first.len() - 1) as u32,
            ..ShaderModuleDesc::default()
        });
        assert_eq!(module.m_glFixup.len(), 1);
        assert_eq!(module.m_glFixup[0].name, b"first");

        let second = fixup_blob(&[(1, 7, b"again")]);
        module.applyGLFixupFromDesc(&ShaderModuleDesc {
            glFixupBytes: Some(&second),
            glFixupSize: second.len() as u32,
            ..ShaderModuleDesc::default()
        });
        assert_eq!(module.m_glFixup.len(), 2);
        assert_eq!(module.m_glFixup[1].name, b"again");
    }

    #[test]
    fn shader_module_gl_fixup_preserves_non_utf8_names() {
        let fixup = fixup_blob(&[(1, 9, &[0xff, 0x00, 0x80])]);
        let mut module = ShaderModule::new();
        module.applyGLFixupFromDesc(&ShaderModuleDesc {
            glFixupBytes: Some(&fixup),
            glFixupSize: fixup.len() as u32,
            ..ShaderModuleDesc::default()
        });
        assert_eq!(module.m_glFixup[0].name, [0xff, 0x00, 0x80]);
    }

    #[test]
    fn shader_module_missing_binding_map_is_a_hard_failure() {
        let result = std::panic::catch_unwind(|| {
            ShaderModule::new().applyBindingMapFromDesc(&ShaderModuleDesc::default())
        });
        assert!(result.is_err());
    }
}
