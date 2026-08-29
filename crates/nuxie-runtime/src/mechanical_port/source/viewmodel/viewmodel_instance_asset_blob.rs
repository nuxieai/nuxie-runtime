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
    pub(crate) fn restore_host_asset(&mut self, asset: Option<Arc<RuntimeBlobAsset>>) {
        self.blob_asset = asset;
    }
    pub fn new() -> Self {
        Self {
            base: ViewModelInstanceAssetBlobBase::default(),
            blob_asset: None,
        }
    }

    fn set_property_value(&mut self, value: u32) {
        if self.base.set_property_value_value(value) {
            self.property_value_changed();
            <Self as crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_asset_base::ViewModelInstanceAssetBaseCallbacks>::notify_property_changed(self, crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_asset_base::ViewModelInstanceAssetBase::PROPERTY_VALUE_PROPERTY_KEY);
        }
    }

    pub fn property_value_changed(&mut self) {
        if let Some(owner) = crate::mechanical_port::source::core::CoreObject::core(self).handle() {
            crate::host_viewmodel::capture_native_change(
                owner,
                crate::RuntimeViewModelChangeValue::Blob(self.base.property_value() as u64),
            );
        }
        self.base.add_dirt(ComponentDirt::BINDINGS);
        #[cfg(feature = "tools")]
        if let Some(callback) = self.base.changed_callback() {
            let value = self.base.property_value();
            if !crate::view_model_cell::defer_transaction_tools_callback(self, move |owner| {
                callback(&mut owner.base.base, value);
            }) {
                callback(&mut self.base.base, value);
            }
        }
        self.base.on_value_changed();
    }

    pub fn set_value(&mut self, blob: Option<Arc<RuntimeBlobAsset>>) {
        if matches!((&self.blob_asset, &blob), (Some(left), Some(right)) if Arc::ptr_eq(left, right))
            || self.blob_asset.is_none() && blob.is_none()
        {
            self.set_property_value(u32::MAX);
            return;
        }
        let already_sentinel = self.base.property_value() == u32::MAX;
        self.blob_asset = blob;
        if already_sentinel {
            // A live asset replacement is a write even when its serialized ID
            // stays at the sentinel. ID transitions are captured by the setter.
            if let Some(owner) =
                crate::mechanical_port::source::core::CoreObject::core(self).handle()
            {
                crate::host_viewmodel::capture_native_change(
                    owner,
                    crate::RuntimeViewModelChangeValue::Blob(u32::MAX as u64),
                );
            }
        }
        #[cfg(feature = "tools")]
        if !already_sentinel {
            self.set_property_value(u32::MAX);
        } else if let Some(callback) = self.base.changed_callback() {
            let value = self.base.property_value();
            if !crate::view_model_cell::defer_transaction_tools_callback(self, move |owner| {
                callback(&mut owner.base.base, value);
            }) {
                callback(&mut self.base.base, value);
            }
        }
        #[cfg(not(feature = "tools"))]
        self.set_property_value(u32::MAX);
        self.base.add_dirt(ComponentDirt::BINDINGS);
        self.base.on_value_changed();
    }

    pub fn asset(&self) -> Option<Arc<RuntimeBlobAsset>> {
        self.blob_asset.clone()
    }

    pub fn apply_value(&mut self, data_value: &DataValueInteger) {
        self.set_property_value(data_value.value());
    }

    pub fn apply_data_value(
        &mut self,
        data_value: &dyn crate::mechanical_port::source::data_bind::data_values::data_value::DataValue,
    ) {
        if let Some(asset_value) = data_value.as_any().downcast_ref::<DataValueAssetBlob>() {
            let blob = asset_value.file_asset();
            self.set_value(blob.clone());
            if blob.is_some() {
                return;
            }
        }
        if let Some(value) = crate::mechanical_port::source::data_bind::data_values::data_value_integer::integer_value(data_value) {
            self.apply_value(&DataValueInteger::new(value));
        }
    }

    pub fn clone_value(&self) -> Box<Self> {
        let mut cloned = Box::new(Self::new());
        let mut base = std::mem::take(&mut cloned.base.base.base);
        base.copy(&self.base.base.base, &mut *cloned);
        cloned.base.base.base = base;
        for asset in self.base.assets() {
            cloned.base.add_asset(asset.clone());
        }
        cloned
    }
}
