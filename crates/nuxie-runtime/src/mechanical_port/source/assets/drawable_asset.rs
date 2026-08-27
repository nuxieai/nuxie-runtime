use crate::mechanical_port::source::generated::assets::drawable_asset_base::DrawableAssetBase;

pub struct DrawableAsset {
    pub base: DrawableAssetBase,
}

impl Default for DrawableAsset {
    fn default() -> Self {
        Self {
            base: DrawableAssetBase::default(),
        }
    }
}
