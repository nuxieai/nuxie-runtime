use crate::mechanical_port::source::{
    assets::font_asset::FontAsset, component_dirt::ComponentDirt,
    data_bind::data_values::data_value_integer::DataValueInteger,
    generated::viewmodel::viewmodel_instance_asset_font_base::ViewModelInstanceAssetFontBase,
    refcnt::RiveRc, text_engine::FontRef,
};

pub struct ViewModelInstanceAssetFont {
    pub base: ViewModelInstanceAssetFontBase,
    font_asset: RiveRc<FontAsset>,
}

impl ViewModelInstanceAssetFont {
    pub fn new() -> Self {
        Self {
            base: ViewModelInstanceAssetFontBase::default(),
            font_asset: RiveRc::new(FontAsset::default()),
        }
    }

    pub fn property_value_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::BINDINGS);
        #[cfg(feature = "rive_tools")]
        if let Some(callback) = self.base.changed_callback() {
            callback(self, self.base.property_value());
        }
        self.base.on_value_changed();
    }

    pub fn set_value(&mut self, font: Option<FontRef>) {
        if self.font_asset.font().as_ref().map(FontRef::as_ptr)
            == font.as_ref().map(FontRef::as_ptr)
        {
            self.base.set_property_value(u32::MAX);
            return;
        }
        #[cfg(feature = "rive_tools")]
        let already_sentinel = self.base.property_value() == u32::MAX;
        self.font_asset.set_font_direct(font);
        #[cfg(feature = "rive_tools")]
        if !already_sentinel {
            self.base.set_property_value(u32::MAX);
        } else if let Some(callback) = self.base.changed_callback() {
            callback(self, self.base.property_value());
        }
        #[cfg(not(feature = "rive_tools"))]
        self.base.set_property_value(u32::MAX);
        self.base.add_dirt(ComponentDirt::BINDINGS);
        self.base.on_value_changed();
    }

    pub fn asset(&self) -> RiveRc<FontAsset> {
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
