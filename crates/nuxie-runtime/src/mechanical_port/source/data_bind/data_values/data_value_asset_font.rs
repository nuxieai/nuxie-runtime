use super::{data_type::DataType, data_value::DataValue, data_value_integer::DataValueInteger};
use crate::mechanical_port::source::text_engine::FontRef;
use crate::mechanical_port::source::{
    assets::font_asset::FontAsset as CoreFontAsset, core::CoreHandle,
};
use core::any::Any;
use std::{cell::RefCell, rc::Rc};
pub struct FontAsset {
    font: RefCell<Option<FontRef>>,
    core_asset: RefCell<Option<CoreHandle>>,
}
impl FontAsset {
    pub fn new() -> Self {
        Self {
            font: RefCell::new(None),
            core_asset: RefCell::new(None),
        }
    }
    pub fn set_font(&self, font: Option<FontRef>) {
        if let Some(asset) = self.core_asset.borrow().clone() {
            CoreFontAsset::set_font_occurrence(&asset, font);
        } else {
            *self.font.borrow_mut() = font;
        }
    }
    pub fn font(&self) -> Option<FontRef> {
        if let Some(asset) = self.core_asset.borrow().as_ref() {
            asset
                .with_downcast::<CoreFontAsset, _>(CoreFontAsset::font)
                .expect("retained font asset")
        } else {
            self.font.borrow().clone()
        }
    }

    pub fn core_asset(&self, context: &CoreHandle) -> CoreHandle {
        if let Some(asset) = self.core_asset.borrow().as_ref() {
            return asset.clone();
        }
        let mut asset = CoreFontAsset::default();
        asset.set_font(self.font.borrow_mut().take());
        let asset = context
            .insert_sibling(asset)
            .expect("live font asset arena");
        *self.core_asset.borrow_mut() = Some(asset.clone());
        asset
    }
}
#[derive(Clone)]
pub struct DataValueAssetFont {
    integer: DataValueInteger,
    file_asset: Rc<FontAsset>,
}
impl Default for DataValueAssetFont {
    fn default() -> Self {
        Self::new(Self::DEFAULT_VALUE)
    }
}
impl DataValueAssetFont {
    pub const TYPE_KEY: DataType = DataType::AssetFont;
    pub const DEFAULT_VALUE: u32 = u32::MAX;
    pub fn new(value: u32) -> Self {
        Self {
            integer: DataValueInteger::new(value),
            file_asset: Rc::new(FontAsset::new()),
        }
    }
    pub fn value(&self) -> u32 {
        self.integer.value()
    }
    pub fn set_value(&mut self, value: u32) {
        self.integer.set_value(value)
    }
    pub fn file_asset(&self) -> Rc<FontAsset> {
        self.file_asset.clone()
    }
    pub fn set_font_value(&self, font: Option<FontRef>) {
        self.file_asset.set_font(font)
    }
    pub fn font_value(&self) -> Option<FontRef> {
        self.file_asset.font()
    }
}
impl DataValue for DataValueAssetFont {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, t: DataType) -> bool {
        t == DataType::AssetFont || t == DataType::Integer
    }
}
