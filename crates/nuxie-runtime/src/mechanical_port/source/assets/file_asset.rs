use crate::mechanical_port::source::{
    core::CoreHandle, generated::assets::file_asset_base::FileAssetBase,
    importers::import_stack::ImportStack, status_code::StatusCode,
};

pub struct FileAsset {
    pub base: FileAssetBase,
    cdn_uuid: Vec<u8>,
    file_asset_referencers: Vec<CoreHandle>,
}

impl Default for FileAsset {
    fn default() -> Self {
        Self {
            base: FileAssetBase::default(),
            cdn_uuid: Vec::new(),
            file_asset_referencers: Vec::new(),
        }
    }
}

impl FileAsset {
    pub fn import(
        &mut self,
        this: CoreHandle,
        adds_to_backboard: bool,
        import_stack: &mut ImportStack,
    ) -> StatusCode {
        if adds_to_backboard {
            let Some(backboard_importer) = import_stack.latest_backboard_importer() else {
                return StatusCode::MissingObject;
            };
            backboard_importer.add_file_asset(this);
        }
        self.base.import(import_stack)
    }

    pub fn unique_name(&self) -> String {
        let name = self.base.name();
        let stem = name.rsplit_once('.').map_or(name, |(stem, _)| stem);
        format!("{stem}-{}", self.base.asset_id())
    }

    pub fn unique_filename(&self, file_extension: &str) -> String {
        format!("{}.{}", self.unique_name(), file_extension)
    }

    pub fn copy_cdn_uuid(&mut self, _object: &FileAssetBase) {
        panic!("FileAsset::copyCdnUuid must never be called");
    }

    pub fn decode_cdn_uuid(&mut self, value: &[u8]) {
        self.cdn_uuid = value.to_vec();
    }

    pub fn cdn_uuid(&self) -> &[u8] {
        &self.cdn_uuid
    }

    pub fn cdn_uuid_str(&self) -> String {
        if self.cdn_uuid.len() != 16 {
            return String::new();
        }
        let indices = [3usize, 2, 1, 0, 5, 4, 7, 6, 9, 8, 15, 14, 13, 12, 11, 10];
        let mut result = String::with_capacity(36);
        for index in indices {
            use std::fmt::Write;
            write!(&mut result, "{:02x}", self.cdn_uuid[index])
                .expect("writing to String cannot fail");
            if matches!(index, 0 | 4 | 6 | 8) {
                result.push('-');
            }
        }
        result
    }

    pub fn file_asset_referencers(&self) -> &[CoreHandle] {
        &self.file_asset_referencers
    }

    pub fn add_file_asset_referencer(&mut self, referencer: CoreHandle) {
        self.file_asset_referencers.push(referencer);
    }

    pub fn remove_file_asset_referencer(&mut self, referencer: CoreHandle) {
        self.file_asset_referencers
            .retain(|candidate| *candidate != referencer);
    }
}
