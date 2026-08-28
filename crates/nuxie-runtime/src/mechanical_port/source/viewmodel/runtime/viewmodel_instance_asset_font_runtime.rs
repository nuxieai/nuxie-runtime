use super::viewmodel_instance_value_runtime::{DataType, ViewModelInstanceValueRuntime};
use crate::mechanical_port::source::text_engine::FontRef;

#[derive(Clone)]
pub struct ViewModelInstanceAssetFontRuntime {
    base: ViewModelInstanceValueRuntime,
}

impl ViewModelInstanceAssetFontRuntime {
    pub fn new(base: ViewModelInstanceValueRuntime) -> Option<Self> {
        (base.data_type() == DataType::AssetFont).then_some(Self { base })
    }
    pub fn set_value(&self, value: Option<FontRef>) {
        self.base.handle().with_mut(|property| {
            if let Some(property) = property.as_view_model_instance_asset_font_mut() {
                property.set_value(value);
            }
        });
    }
    #[cfg(any(test, feature = "tools"))]
    pub fn testing_value(&self) -> Option<FontRef> {
        self.base
            .handle()
            .with(|property| {
                property
                    .as_view_model_instance_asset_font()
                    .and_then(|property| property.asset().font())
            })
            .flatten()
    }
    pub fn data_type(&self) -> DataType {
        DataType::AssetFont
    }
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.base
    }
}
