use std::rc::Rc;

use nuxie_render_api::RenderImage;

use super::viewmodel_instance_value_runtime::{DataType, ViewModelInstanceValueRuntime};

#[derive(Clone)]
pub struct ViewModelInstanceAssetImageRuntime {
    base: ViewModelInstanceValueRuntime,
}

impl ViewModelInstanceAssetImageRuntime {
    pub fn new(base: ViewModelInstanceValueRuntime) -> Option<Self> {
        (base.data_type() == DataType::AssetImage).then_some(Self { base })
    }
    pub fn set_value(&self, value: Option<Rc<dyn RenderImage>>) {
        self.base.handle().with_mut(|property| {
            if let Some(property) = property.as_view_model_instance_asset_image_mut() {
                property.set_value(value);
            }
        });
    }
    #[cfg(any(test, feature = "tools"))]
    pub fn testing_value(&self) -> Option<Rc<dyn RenderImage>> {
        self.base
            .handle()
            .with(|property| {
                property
                    .as_view_model_instance_asset_image()
                    .and_then(|property| property.asset().render_image())
            })
            .flatten()
    }
    pub fn data_type(&self) -> DataType {
        DataType::AssetImage
    }
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.base
    }
}
