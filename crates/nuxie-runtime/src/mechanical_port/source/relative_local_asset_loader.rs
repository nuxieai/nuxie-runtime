use std::{
    fs::File,
    io::{Read, Seek},
    path::Path,
};

use crate::mechanical_port::source::{
    assets::file_asset::FileAsset, factory::Factory, file_asset_loader::FileAssetLoader,
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
        asset: &mut FileAsset,
        _in_band_bytes: &[u8],
        factory: &mut Factory,
    ) -> bool {
        let filename = format!("{}{}", self.path, asset.unique_filename());
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
            asset.decode(&mut bytes, factory);
        }
        true
    }
}
