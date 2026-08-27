use std::collections::HashMap;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, factory::Factory,
    generated::assets::manifest_asset_base::ManifestAssetBase,
};

pub struct ManifestAsset {
    pub base: ManifestAssetBase,
    names: HashMap<i32, String>,
    paths: HashMap<i32, Vec<u32>>,
}

impl Default for ManifestAsset {
    fn default() -> Self {
        Self {
            base: ManifestAssetBase::default(),
            names: HashMap::new(),
            paths: HashMap::new(),
        }
    }
}

impl ManifestAsset {
    fn decode_names(&mut self, reader: &mut BinaryReader<'_>) -> bool {
        let count = reader.read_var_u64();
        if reader.has_error() {
            return false;
        }
        for _ in 0..count {
            let id = reader.read_var_u64() as i32;
            if reader.has_error() {
                return false;
            }
            let value = reader.read_string();
            if reader.has_error() {
                return false;
            }
            self.names.insert(id, value);
        }
        true
    }

    fn decode_paths(&mut self, reader: &mut BinaryReader<'_>) -> bool {
        let count = reader.read_var_u64();
        if reader.has_error() {
            return false;
        }
        for _ in 0..count {
            let id = reader.read_var_u64() as i32;
            if reader.has_error() {
                return false;
            }
            let path_length = reader.read_var_u64() as i32;
            if reader.has_error() {
                return false;
            }
            let mut path = Vec::new();
            // C++ compares uint64_t j with the signed pathLength, so the
            // signed value is converted back to u64 for the loop bound.
            for _ in 0..(path_length as u64) {
                path.push(reader.read_var_u64() as u32);
            }
            self.paths.insert(id, path);
            if reader.has_error() {
                return false;
            }
        }
        true
    }

    pub fn decode(&mut self, bytes: &[u8], _factory: &mut Factory) -> bool {
        if bytes.is_empty() {
            return true;
        }
        let mut reader = BinaryReader::new(bytes);
        while !reader.reached_end() {
            let section_value = reader.read_var_u64();
            if reader.has_error() {
                return false;
            }
            let section_size = reader.read_var_u64();
            if reader.has_error() {
                return false;
            }
            let section_start = reader.position();
            match section_value as u8 {
                0 => {
                    if !self.decode_names(&mut reader) {
                        return false;
                    }
                }
                1 => {
                    if !self.decode_paths(&mut reader) {
                        return false;
                    }
                }
                _ => {
                    reader.read_bytes(section_size as usize);
                    if reader.has_error() {
                        return false;
                    }
                    continue;
                }
            }
            if reader.position() - section_start != section_size as usize {
                return false;
            }
        }
        true
    }

    pub fn file_extension(&self) -> &'static str {
        "man"
    }

    pub fn resolve_name(&self, id: i32) -> &str {
        self.names.get(&id).map_or("", String::as_str)
    }

    pub fn resolve_path(&self, id: i32) -> &[u32] {
        self.paths.get(&id).map_or(&[], Vec::as_slice)
    }

    pub fn adds_to_backboard(&self) -> bool {
        false
    }
}
