use std::rc::Rc;

use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    data_bind::data_values::{
        data_value_asset_font::FontAsset, data_value_integer::DataValueInteger,
    },
    generated::viewmodel::viewmodel_instance_asset_font_base::ViewModelInstanceAssetFontBase,
    text_engine::FontRef,
};

pub struct ViewModelInstanceAssetFont {
    pub base: ViewModelInstanceAssetFontBase,
    font_asset: Rc<FontAsset>,
}

impl Default for ViewModelInstanceAssetFont {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewModelInstanceAssetFont {
    pub fn new() -> Self {
        Self {
            base: ViewModelInstanceAssetFontBase::default(),
            font_asset: Rc::new(FontAsset::new()),
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

    pub fn set_value(&mut self, font: Option<FontRef>) {
        let previous = self.font_asset.font();
        if matches!((&previous, &font), (Some(left), Some(right)) if Rc::ptr_eq(left, right))
            || previous.is_none() && font.is_none()
        {
            self.base.set_property_value(u32::MAX);
            return;
        }
        #[cfg(feature = "tools")]
        let already_sentinel = self.base.property_value() == u32::MAX;
        self.font_asset.set_font(font);
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

    pub fn asset(&self) -> Rc<FontAsset> {
        self.font_asset.clone()
    }

    pub fn apply_value(&mut self, data_value: &DataValueInteger) {
        if let Some(asset_value) = data_value.as_asset_font() {
            let font = asset_value.font_value();
            self.set_value(font.clone());
            if font.is_some() {
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
