use std::{
    fs::File,
    io::{Read, Seek},
    path::Path,
};

use crate::mechanical_port::source::{
    core::CoreHandle, factory::RuntimeFactoryHandle, file_asset_loader::FileAssetLoader,
    generated::core_registry::CoreCapabilities,
};

pub struct RelativeLocalAssetLoader {
    path: String,
}

impl RelativeLocalAssetLoader {
    pub fn new(filename: String) -> Self {
        let path = filename.rfind('/').map_or_else(String::new, |final_slash| {
            filename[..=final_slash].to_owned()
        });
        Self { path }
    }
}

impl FileAssetLoader for RelativeLocalAssetLoader {
    fn load_contents(
        &mut self,
        asset: CoreHandle,
        _in_band_bytes: &[u8],
        factory: &RuntimeFactoryHandle,
    ) -> bool {
        let Some(unique_filename) = asset
            .with(|asset| {
                CoreCapabilities::as_file_asset(asset).map(|asset| {
                    asset
                        .file_asset_base()
                        .unique_filename(asset.file_extension())
                })
            })
            .flatten()
        else {
            return false;
        };
        let filename = format!("{}{}", self.path, unique_filename);
        let path = Path::new(&filename);
        let Ok(mut file) = File::open(path) else {
            eprintln!("Failed to find file at {filename}");
            return false;
        };
        let Ok(length) = file.seek(std::io::SeekFrom::End(0)) else {
            return true;
        };
        if file.seek(std::io::SeekFrom::Start(0)).is_err() {
            return true;
        }
        let mut bytes = vec![0; length as usize];
        if file.read_exact(&mut bytes).is_ok() {
            asset.with_mut(|asset| {
                CoreCapabilities::as_file_asset_mut(asset)
                    .is_some_and(|asset| asset.file_asset_decode(&mut bytes, factory))
            });
        }
        true
    }
}
