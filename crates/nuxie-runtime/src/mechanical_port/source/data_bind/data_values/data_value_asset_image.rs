use super::{data_type::DataType, data_value::DataValue, data_value_integer::DataValueInteger};
use core::any::Any;
use std::{cell::RefCell, rc::Rc};
pub trait RenderImage: Any {}
pub struct ImageAsset {
    image: RefCell<Option<Rc<dyn RenderImage>>>,
}
impl ImageAsset {
    pub fn new() -> Self {
        Self {
            image: RefCell::new(None),
        }
    }
    pub fn set_render_image(&self, image: Option<Rc<dyn RenderImage>>) {
        *self.image.borrow_mut() = image
    }
    pub fn render_image(&self) -> Option<Rc<dyn RenderImage>> {
        self.image.borrow().clone()
    }
}
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
