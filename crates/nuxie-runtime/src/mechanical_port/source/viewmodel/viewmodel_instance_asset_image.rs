use std::rc::Rc;

use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    data_bind::data_values::{
        data_value_asset_image::ImageAsset, data_value_integer::DataValueInteger,
    },
    generated::viewmodel::viewmodel_instance_asset_image_base::ViewModelInstanceAssetImageBase,
};
use nuxie_render_api::RenderImage;

pub struct ViewModelInstanceAssetImage {
    pub base: ViewModelInstanceAssetImageBase,
    image_asset: Rc<ImageAsset>,
}

impl Default for ViewModelInstanceAssetImage {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewModelInstanceAssetImage {
    pub fn new() -> Self {
        Self {
            base: ViewModelInstanceAssetImageBase::default(),
            image_asset: Rc::new(ImageAsset::new()),
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

    pub fn set_value(&mut self, image: Option<Rc<dyn RenderImage>>) {
        let previous = self.image_asset.render_image();
        if matches!((&previous, &image), (Some(left), Some(right)) if Rc::ptr_eq(left, right))
            || previous.is_none() && image.is_none()
        {
            self.base.set_property_value(u32::MAX);
            return;
        }
        #[cfg(feature = "tools")]
        let already_sentinel = self.base.property_value() == u32::MAX;
        self.image_asset.set_render_image(image);
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

    pub fn asset(&self) -> Rc<ImageAsset> {
        self.image_asset.clone()
    }

    pub fn apply_value(&mut self, data_value: &DataValueInteger) {
        if let Some(asset_value) = data_value.as_asset_image() {
            let image = asset_value.image_value();
            self.set_value(image.clone());
            if image.is_some() {
                return;
            }
        }
        self.base.set_property_value(data_value.value());
    }

    pub fn clone_value(&self) -> Box<Self> {
        let mut cloned = Box::new(Self::new());
        cloned.base.copy_from(&self.base);
        for asset in self.base.assets() {
            cloned.base.add_asset(asset.clone());
        }
        cloned
    }
}
