use crate::mechanical_port::source::{
    factory::RuntimeFactoryHandle,
    generated::assets::script_module_asset_base::ScriptModuleAssetBase,
    signed_content_header::SignedContentHeader,
};

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    Luau = 0,
    AssemblyScript = 1,
}

#[derive(Default)]
pub struct ScriptModuleAsset {
    pub base: ScriptModuleAssetBase,
    module: Vec<u8>,
}

impl ScriptModuleAsset {
    pub fn decode(&mut self, data: &mut Vec<u8>, _factory: &RuntimeFactoryHandle) -> bool {
        self.base.base.set_verified(false);
        let header = SignedContentHeader::new(data);
        if !header.is_valid() {
            return false;
        }
        self.module = header.content().to_vec();
        true
    }

    pub fn file_extension(&self) -> &'static str {
        "wasm"
    }
    pub fn module(&self) -> &[u8] {
        &self.module
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanical_port::source::core::{CoreObject, binary_reader::BinaryReader};
    use nuxie_render_api::{PersistentFactory, RecordingFactory};

    #[test]
    fn signed_module_preserves_content_and_resets_verification() {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
        let mut asset = ScriptModuleAsset::default();
        assert!(!asset.base.verified());
        assert_eq!(asset.base.language(), Language::Luau as u32);
        assert_eq!(asset.file_extension(), "wasm");
        let module = b"\0asm\x01\0\0\0";
        let mut bytes = vec![0x80];
        bytes.extend_from_slice(&[0; 64]);
        bytes.extend_from_slice(module);
        let original = bytes.clone();
        assert!(asset.decode(&mut bytes, &factory));
        assert_eq!(asset.module(), module);
        assert_eq!(bytes, original);
        asset.base.base.set_verified(true);
        assert!(!asset.decode(&mut vec![0x80], &factory));
        assert!(!asset.base.verified());
        assert_eq!(asset.module(), module);
        assert!(!asset.decode(&mut Vec::new(), &factory));
        asset.base.base.set_verified(true);
        assert!(asset.decode(&mut vec![0, 42, 0, 43], &factory));
        assert!(!asset.base.verified());
        assert_eq!(asset.module(), &[42, 0, 43]);
    }

    #[test]
    fn module_language_deserializes_as_file_asset() {
        let mut asset = ScriptModuleAsset::default();
        assert!(asset.deserialize(1087, &mut BinaryReader::new(&[1])));
        assert_eq!(asset.base.language(), Language::AssemblyScript as u32);
        assert_eq!(asset.core_type(), 1071);
        assert!(asset.is_type_of(103));
        assert!(asset.is_type_of(99));
        assert!(!asset.is_type_of(971));
        assert!(!asset.base.verified());
        assert!(asset.module().is_empty());
    }

    #[test]
    #[should_panic(expected = "FileAsset::copyCdnUuid must never be called")]
    fn module_clone_reaches_upstream_file_asset_copy_assertion() {
        // The generated clone inherits FileAsset::copyCdnUuid's explicit
        // debug assertion; it is not a supported content-copy operation.
        let _ = ScriptModuleAsset::default().clone_boxed();
    }
}
