use crate::mechanical_port::source::generated::assets::asset_base::AssetBase;

pub struct Asset {
    pub base: AssetBase,
}

impl Default for Asset {
    fn default() -> Self {
        Self {
            base: AssetBase::default(),
        }
    }
}
