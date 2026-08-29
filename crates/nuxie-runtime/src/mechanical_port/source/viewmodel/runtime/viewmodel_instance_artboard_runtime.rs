use crate::mechanical_port::source::{
    bindable_artboard::RuntimeBindableArtboardHandle, core::CoreHandle,
};

use super::viewmodel_instance_value_runtime::{DataType, ViewModelInstanceValueRuntime};

#[derive(Clone)]
pub struct ViewModelInstanceArtboardRuntime {
    base: ViewModelInstanceValueRuntime,
}

impl ViewModelInstanceArtboardRuntime {
    pub fn new(base: ViewModelInstanceValueRuntime) -> Option<Self> {
        (base.data_type() == DataType::Artboard).then_some(Self { base })
    }
    pub fn set_value(&self, artboard: Option<RuntimeBindableArtboardHandle>) {
        self.base.handle().with_mut(|property| {
            if let Some(property) = property.as_view_model_instance_artboard_mut() {
                property.set_bound_view_model_instance(None);
                property.set_asset(artboard);
            }
        });
    }
    pub fn set_view_model_instance(&self, instance: Option<CoreHandle>) {
        self.base.handle().with_mut(|property| {
            if let Some(property) = property.as_view_model_instance_artboard_mut() {
                property.set_bound_view_model_instance(instance);
            }
        });
    }
    pub fn artboard_name(&self) -> String {
        self.base
            .handle()
            .with(|property| {
                property
                    .as_view_model_instance_artboard()
                    .and_then(|property| {
                        property.asset().map(|asset| {
                            asset.with_artboard(|artboard| artboard.base.base.name().to_owned())
                        })
                    })
            })
            .flatten()
            .unwrap_or_default()
    }
    #[cfg(any(test, feature = "tools"))]
    pub fn testing_value(&self) -> Option<RuntimeBindableArtboardHandle> {
        self.base
            .handle()
            .with(|property| {
                property
                    .as_view_model_instance_artboard()
                    .and_then(|property| property.asset())
            })
            .flatten()
    }
    pub fn data_type(&self) -> DataType {
        DataType::Artboard
    }
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.base
    }
}
