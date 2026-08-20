// Mechanical translation of:
//   renderer/include/rive/renderer/ore/ore_shader_module.hpp
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
//
// Copyright 2025 Rive

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use crate::binding_map::BindingMap;
use crate::gpu_resource::{GpuResourceManager, ResourceHandle};
use crate::types::ShaderModuleDesc;

/// Texture-sampler pair from RSTB shader reflection.
///
/// Records which texture binding is paired with which sampler binding in the
/// shader's sampling expressions. Used by the GL backend to bind sampler
/// objects to the correct texture unit.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextureSamplerPair {
    pub textureGroup: u8,
    pub textureBinding: u8,
    pub samplerGroup: u8,
    pub samplerBinding: u8,
}

/// One entry in the GL program-link fixup table.
///
/// The table tells the runtime which GL binding point / texture unit each
/// named uniform should land on, letting the GL backend call
/// `glUniformBlockBinding` / `glUniform1i` without parsing emitted GLSL names
/// at runtime.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GLFixupEntry {
    pub kind: GLFixupKind,
    pub slot: u8,
    pub name: Vec<u8>,
}

/// Raw `GLFixupEntry::Kind` byte.
///
/// The pinned C++ parser uses `static_cast` and therefore retains unknown kind
/// values. A transparent byte preserves that forward-compatible behavior while
/// retaining the two defined constants.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GLFixupKind(pub u8);

impl GLFixupKind {
    pub const UBOBlock: Self = Self(0);
    pub const SamplerUniform: Self = Self(1);
}

/// C++ nested-name alias retained for source-corresponding callers.
pub type Kind = GLFixupKind;

/// ORE shader module state shared by concrete backends.
///
/// C++ derives this class from `GPUResource`. Rust keeps the payload separate
/// from its intrusive lifetime owner: use [`ShaderModule::into_resource`] to
/// create a [`ResourceHandle<ShaderModule>`] when a manager owns the module.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ShaderModule {
    pub m_textureSamplerPairs: Vec<TextureSamplerPair>,

    // (group, binding) -> per-backend native slot map for this shader's
    // resources. Parsed from the RSTB binding-map sidecar by each backend's
    // `makeShaderModule` and consumed by Pipeline.
    pub m_bindingMap: BindingMap,

    #[cfg(feature = "tools")]
    pub m_shaderAssetId: u32,

    // Parsed fixup table. Populated from the stage's RSTB sidecar (target
    // 14 = VS, target 15 = FS). Empty for non-GL backends.
    pub m_glFixup: Vec<GLFixupEntry>,
}

impl ShaderModule {
    /// C++ default constructor equivalent. The upstream constructor is
    /// protected; this public value constructor is the Rust payload boundary.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adopt this payload into the translated `GPUResource` lifetime owner.
    pub fn into_resource(self, manager: Option<GpuResourceManager>) -> ResourceHandle<Self> {
        ResourceHandle::new(manager, self)
    }

    /// Parse the mandatory binding-map sidecar and apply the optional GL
    /// fixups. The C++ path asserts on a missing or malformed binding map, so
    /// this translation preserves the hard failure rather than returning a
    /// fallback module.
    pub fn apply_binding_map_from_desc(&mut self, desc: &ShaderModuleDesc<'_>) {
        let bytes = desc
            .bindingMapBytes
            .expect("ShaderModuleDesc::bindingMapBytes is required");
        assert!(
            !bytes.is_empty(),
            "ShaderModuleDesc::bindingMapBytes is required"
        );
        let ok = BindingMap::from_blob(bytes, &mut self.m_bindingMap);
        assert!(ok, "binding-map sidecar failed to parse");
        self.apply_gl_fixup_from_desc(desc);
        #[cfg(feature = "tools")]
        {
            self.m_shaderAssetId = desc.shaderAssetId;
        }
    }

    /// C++ spelling retained for source-corresponding callers.
    pub fn applyBindingMapFromDesc(&mut self, desc: &ShaderModuleDesc<'_>) {
        self.apply_binding_map_from_desc(desc);
    }

    /// Parse `desc.glFixupBytes` into `m_glFixup`.
    ///
    /// Absent, wrong-version, and malformed blobs return without an error,
    /// matching upstream. Entries successfully decoded before truncation stay
    /// appended; repeated calls append to the existing vector because the C++
    /// helper never clears `m_glFixup`.
    pub fn apply_gl_fixup_from_desc(&mut self, desc: &ShaderModuleDesc<'_>) {
        let Some(bytes) = desc.glFixupBytes else {
            return;
        };
        if bytes.len() < 3 {
            return;
        }
        let mut input = &bytes[..];
        if input[0] != 1 {
            return;
        }
        input = &input[1..];
        let count = u16::from_le_bytes([input[0], input[1]]);
        input = &input[2..];
        self.m_glFixup.reserve(usize::from(count));
        for _ in 0..count {
            if input.len() < 4 {
                return;
            }
            let kind = GLFixupKind(input[0]);
            let slot = input[1];
            let name_len = usize::from(u16::from_le_bytes([input[2], input[3]]));
            input = &input[4..];
            if input.len() < name_len {
                return;
            }
            // `std::string` is a byte string. Preserve arbitrary payloads;
            // GL authored names happen to be UTF-8 but the parser does not
            // impose that stronger invariant.
            let name = input[..name_len].to_vec();
            input = &input[name_len..];
            self.m_glFixup.push(GLFixupEntry { kind, slot, name });
        }
    }

    /// C++ spelling retained for source-corresponding callers.
    pub fn applyGLFixupFromDesc(&mut self, desc: &ShaderModuleDesc<'_>) {
        self.apply_gl_fixup_from_desc(desc);
    }

    #[cfg(feature = "tools")]
    pub fn shaderAssetId(&self) -> u32 {
        self.m_shaderAssetId
    }
}

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
            glFixupBytes: Some(&fixup),
            ..ShaderModuleDesc::default()
        };
        let mut module = ShaderModule::new();
        module.apply_binding_map_from_desc(&desc);
        assert!(module.m_bindingMap.is_empty());
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
        module.apply_gl_fixup_from_desc(&absent);
        module.apply_gl_fixup_from_desc(&ShaderModuleDesc {
            glFixupBytes: Some(&[2, 0, 0]),
            ..ShaderModuleDesc::default()
        });
        assert!(module.m_glFixup.is_empty());
    }

    #[test]
    fn shader_module_gl_fixup_truncation_preserves_prefix_and_repeated_calls_append() {
        let first = fixup_blob(&[(0, 2, b"first"), (1, 4, b"second")]);
        let mut module = ShaderModule::new();
        module.apply_gl_fixup_from_desc(&ShaderModuleDesc {
            glFixupBytes: Some(&first[..first.len() - 1]),
            ..ShaderModuleDesc::default()
        });
        assert_eq!(module.m_glFixup.len(), 1);
        assert_eq!(module.m_glFixup[0].name, b"first");

        module.apply_gl_fixup_from_desc(&ShaderModuleDesc {
            glFixupBytes: Some(&fixup_blob(&[(1, 7, b"again")])),
            ..ShaderModuleDesc::default()
        });
        assert_eq!(module.m_glFixup.len(), 2);
        assert_eq!(module.m_glFixup[1].name, b"again");
    }

    #[test]
    fn shader_module_gl_fixup_preserves_non_utf8_names() {
        let fixup = fixup_blob(&[(1, 9, &[0xff, 0x00, 0x80])]);
        let mut module = ShaderModule::new();
        module.apply_gl_fixup_from_desc(&ShaderModuleDesc {
            glFixupBytes: Some(&fixup),
            ..ShaderModuleDesc::default()
        });
        assert_eq!(module.m_glFixup[0].name, [0xff, 0x00, 0x80]);
    }

    #[test]
    fn shader_module_missing_binding_map_is_a_hard_failure() {
        let result = std::panic::catch_unwind(|| {
            ShaderModule::new().apply_binding_map_from_desc(&ShaderModuleDesc::default())
        });
        assert!(result.is_err());
    }
}
