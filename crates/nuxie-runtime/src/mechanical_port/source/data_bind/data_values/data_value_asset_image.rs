use super::{data_type::DataType, data_value::DataValue, data_value_integer::DataValueInteger};
use crate::mechanical_port::source::{
    assets::image_asset::ImageAsset as CoreImageAsset, core::CoreHandle,
};
use core::any::Any;
use nuxie_render_api::RenderImage;
use std::{cell::RefCell, rc::Rc};
pub struct ImageAsset {
    image: RefCell<Option<Rc<dyn RenderImage>>>,
    core_asset: RefCell<Option<CoreHandle>>,
}
impl ImageAsset {
    pub fn new() -> Self {
        Self {
            image: RefCell::new(None),
            core_asset: RefCell::new(None),
        }
    }
    pub fn set_render_image(&self, image: Option<Rc<dyn RenderImage>>) {
        if let Some(asset) = self.core_asset.borrow().clone() {
            CoreImageAsset::set_render_image_occurrence(&asset, image);
        } else {
            *self.image.borrow_mut() = image;
        }
    }
    pub fn render_image(&self) -> Option<Rc<dyn RenderImage>> {
        if let Some(asset) = self.core_asset.borrow().as_ref() {
            asset
                .with_downcast::<CoreImageAsset, _>(|asset| asset.render_image().cloned())
                .expect("retained image asset")
        } else {
            self.image.borrow().clone()
        }
    }

    pub fn core_asset(&self, context: &CoreHandle) -> CoreHandle {
        if let Some(asset) = self.core_asset.borrow().as_ref() {
            return asset.clone();
        }
        let mut asset = CoreImageAsset::default();
        // Move the retained payload into the Core occurrence; the wrapper reads
        // that same owner thereafter, so host writes cannot leave a stale copy.
        asset.set_render_image(self.image.borrow_mut().take());
        let asset = context
            .insert_sibling(asset)
            .expect("live image asset arena");
        *self.core_asset.borrow_mut() = Some(asset.clone());
        asset
    }
}
#[derive(Clone)]
pub struct DataValueAssetImage {
    integer: DataValueInteger,
    file_asset: Rc<ImageAsset>,
}
impl Default for DataValueAssetImage {
    fn default() -> Self {
        Self::new(Self::DEFAULT_VALUE)
    }
}
impl DataValueAssetImage {
    pub const TYPE_KEY: DataType = DataType::AssetImage;
    pub const DEFAULT_VALUE: u32 = u32::MAX;
    pub fn new(value: u32) -> Self {
        Self {
            integer: DataValueInteger::new(value),
            file_asset: Rc::new(ImageAsset::new()),
        }
    }
    pub fn value(&self) -> u32 {
        self.integer.value()
    }
    pub fn set_value(&mut self, value: u32) {
        self.integer.set_value(value)
    }
    pub fn file_asset(&self) -> Rc<ImageAsset> {
        self.file_asset.clone()
    }
    pub fn set_image_value(&self, image: Option<Rc<dyn RenderImage>>) {
        self.file_asset.set_render_image(image)
    }
    pub fn image_value(&self) -> Option<Rc<dyn RenderImage>> {
        self.file_asset.render_image()
    }
}
impl DataValue for DataValueAssetImage {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, t: DataType) -> bool {
        t == DataType::AssetImage || t == DataType::Integer
    }
}
