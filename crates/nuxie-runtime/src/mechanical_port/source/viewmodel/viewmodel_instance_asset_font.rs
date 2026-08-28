use std::rc::Rc;

use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    data_bind::data_values::{
        data_value_asset_font::{DataValueAssetFont, FontAsset},
        data_value_integer::DataValueInteger,
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
                crate::RuntimeViewModelChangeValue::Font(self.base.property_value() as u64),
            );
        }
        self.base.add_dirt(ComponentDirt::BINDINGS);
        #[cfg(feature = "tools")]
        if let Some(callback) = self.base.changed_callback() {
            let value = self.base.property_value();
            callback(&mut self.base.base, value);
        }
        self.base.on_value_changed();
    }

    pub fn set_value(&mut self, font: Option<FontRef>) {
        let previous = self.font_asset.font();
        if matches!((&previous, &font), (Some(left), Some(right)) if Rc::ptr_eq(left, right))
            || previous.is_none() && font.is_none()
        {
            self.set_property_value(u32::MAX);
            return;
        }
        #[cfg(feature = "tools")]
        let already_sentinel = self.base.property_value() == u32::MAX;
        self.font_asset.set_font(font);
        #[cfg(feature = "tools")]
        if !already_sentinel {
            self.set_property_value(u32::MAX);
        } else if let Some(callback) = self.base.changed_callback() {
            let value = self.base.property_value();
            callback(&mut self.base.base, value);
        }
        #[cfg(not(feature = "tools"))]
        self.set_property_value(u32::MAX);
        self.base.add_dirt(ComponentDirt::BINDINGS);
        self.base.on_value_changed();
    }

    pub fn asset(&self) -> Rc<FontAsset> {
        self.font_asset.clone()
    }

    pub fn apply_value(&mut self, data_value: &DataValueInteger) {
        self.set_property_value(data_value.value());
    }

    pub fn apply_data_value(
        &mut self,
        data_value: &dyn crate::mechanical_port::source::data_bind::data_values::data_value::DataValue,
    ) {
        if let Some(asset_value) = data_value.as_any().downcast_ref::<DataValueAssetFont>() {
            let font = asset_value.font_value();
            self.set_value(font.clone());
            if font.is_some() {
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
