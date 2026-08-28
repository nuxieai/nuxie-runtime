use crate::mechanical_port::source::generated::viewmodel::viewmodel_property_asset_base::ViewModelPropertyAssetBase;

#[derive(Default)]
pub struct ViewModelPropertyAsset {
    pub base: ViewModelPropertyAssetBase,
}

impl std::ops::Deref for ViewModelPropertyAsset {
    type Target = ViewModelPropertyAssetBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ViewModelPropertyAsset {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
