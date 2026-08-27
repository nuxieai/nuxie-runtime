use crate::mechanical_port::source::{assets::file_asset::FileAsset, factory::Factory};

pub trait FileAssetLoader {
    fn load_contents(
        &mut self,
        asset: &mut FileAsset,
        in_band_bytes: &[u8],
        factory: *mut Factory,
    ) -> bool;
}
