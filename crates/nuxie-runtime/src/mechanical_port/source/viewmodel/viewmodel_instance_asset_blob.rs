use std::sync::Arc;

use crate::RuntimeBlobAsset;

use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    data_bind::data_values::{
        data_value_asset_blob::DataValueAssetBlob, data_value_integer::DataValueInteger,
    },
    generated::viewmodel::viewmodel_instance_asset_blob_base::ViewModelInstanceAssetBlobBase,
};

pub struct ViewModelInstanceAssetBlob {
    pub base: ViewModelInstanceAssetBlobBase,
    blob_asset: Option<Arc<RuntimeBlobAsset>>,
}

impl Default for ViewModelInstanceAssetBlob {
    fn default() -> Self {
        Self::new()
    }
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
        #[cfg(feature = "tools")]
        if let Some(callback) = self.base.changed_callback() {
            callback(self, self.base.property_value());
        }
        self.base.on_value_changed();
    }

    pub fn set_value(&mut self, blob: Option<Arc<RuntimeBlobAsset>>) {
        if matches!((&self.blob_asset, &blob), (Some(left), Some(right)) if Arc::ptr_eq(left, right))
            || self.blob_asset.is_none() && blob.is_none()
        {
            self.base.set_property_value(u32::MAX);
            return;
        }
        #[cfg(feature = "tools")]
        let already_sentinel = self.base.property_value() == u32::MAX;
        self.blob_asset = blob;
        #[cfg(feature = "tools")]
        if !already_sentinel {
            self.base.set_property_value(u32::MAX);
        } else if let Some(callback) = self.base.changed_callback() {
            callback(self, self.base.property_value());
        }
        #[cfg(not(feature = "tools"))]
        self.base.set_property_value(u32::MAX);
        self.base.add_dirt(ComponentDirt::BINDINGS);
        self.base.on_value_changed();
    }

    pub fn asset(&self) -> Option<Arc<RuntimeBlobAsset>> {
        self.blob_asset.clone()
    }

    pub fn apply_value(&mut self, data_value: &DataValueInteger) {
        if let Some(asset_value) = data_value.as_asset_blob() {
            let blob = asset_value.file_asset();
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
