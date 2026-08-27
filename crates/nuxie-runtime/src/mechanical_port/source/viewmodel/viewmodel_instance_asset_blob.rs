use crate::mechanical_port::source::{
    assets::blob_asset::BlobAsset,
    component_dirt::ComponentDirt,
    data_bind::data_values::{
        data_value_asset_blob::DataValueAssetBlob, data_value_integer::DataValueInteger,
    },
    generated::viewmodel::viewmodel_instance_asset_blob_base::ViewModelInstanceAssetBlobBase,
    refcnt::RiveRc,
};

pub struct ViewModelInstanceAssetBlob {
    pub base: ViewModelInstanceAssetBlobBase,
    blob_asset: Option<RiveRc<BlobAsset>>,
}

impl ViewModelInstanceAssetBlob {
    pub fn new() -> Self {
        Self {
            base: ViewModelInstanceAssetBlobBase::default(),
            blob_asset: None,
        }
    }

    pub fn property_value_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::BINDINGS);
        #[cfg(feature = "rive_tools")]
        if let Some(callback) = self.base.changed_callback() {
            callback(self, self.base.property_value());
        }
        self.base.on_value_changed();
    }

    pub fn set_value(&mut self, blob: Option<RiveRc<BlobAsset>>) {
        if self.blob_asset.as_ref().map(RiveRc::as_ptr) == blob.as_ref().map(RiveRc::as_ptr) {
            self.base.set_property_value(u32::MAX);
            return;
        }
        #[cfg(feature = "rive_tools")]
        let already_sentinel = self.base.property_value() == u32::MAX;
        self.blob_asset = blob;
        #[cfg(feature = "rive_tools")]
        if !already_sentinel {
            self.base.set_property_value(u32::MAX);
        } else if let Some(callback) = self.base.changed_callback() {
            callback(self, self.base.property_value());
        }
        #[cfg(not(feature = "rive_tools"))]
        self.base.set_property_value(u32::MAX);
        self.base.add_dirt(ComponentDirt::BINDINGS);
        self.base.on_value_changed();
    }

    pub fn asset(&self) -> Option<RiveRc<BlobAsset>> {
        self.blob_asset.clone()
    }

    pub fn apply_value(&mut self, data_value: &DataValueInteger) {
        if let Some(asset_value) = data_value.as_asset_blob() {
            let blob = asset_value.blob_value();
            self.set_value(blob.clone());
            if blob.is_some() {
                return;
            }
        }
        self.base.set_property_value(data_value.value());
    }

    pub fn clone_value(&self) -> Box<Self> {
        let mut cloned = Box::new(Self {
            base: self.base.clone_base(),
            blob_asset: None,
        });
        for asset in self.base.assets() {
            cloned.base.add_asset(asset.clone());
        }
        cloned
    }
}
