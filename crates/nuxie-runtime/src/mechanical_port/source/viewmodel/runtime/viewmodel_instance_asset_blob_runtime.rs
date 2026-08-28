use std::sync::Arc;

use super::viewmodel_instance_value_runtime::{DataType, ViewModelInstanceValueRuntime};
use crate::RuntimeBlobAsset;

#[derive(Clone)]
pub struct ViewModelInstanceAssetBlobRuntime {
    base: ViewModelInstanceValueRuntime,
}

impl ViewModelInstanceAssetBlobRuntime {
    pub fn new(base: ViewModelInstanceValueRuntime) -> Option<Self> {
        (base.data_type() == DataType::AssetBlob).then_some(Self { base })
    }
    pub fn set_value(&self, value: Option<Arc<RuntimeBlobAsset>>) {
        self.base.handle().with_mut(|property| {
            if let Some(property) = property.as_view_model_instance_asset_blob_mut() {
                property.set_value(value);
            }
        });
    }
    #[cfg(any(test, feature = "tools"))]
    pub fn testing_value(&self) -> Option<Arc<RuntimeBlobAsset>> {
        self.base
            .handle()
            .with(|property| {
                property
                    .as_view_model_instance_asset_blob()
                    .and_then(|property| property.asset())
            })
            .flatten()
    }
    pub fn data_type(&self) -> DataType {
        DataType::AssetBlob
    }
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.base
    }
}
