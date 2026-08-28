use crate::mechanical_port::source::generated::assets::asset_base::{
    AssetBase, AssetBaseCallbacks,
};

pub struct Asset {
    pub base: AssetBase,
}

impl Default for Asset {
    fn default() -> Self {
        Self {
            base: AssetBase::default(),
        }
    }
}

impl Asset {
    pub fn name(&self) -> &str {
        self.base.name()
    }

    pub fn set_name(&mut self, value: String) {
        if self.base.set_name_value(value) {
            self.base
                .base
                .notify_property_changed(AssetBase::NAME_PROPERTY_KEY);
        }
    }
}

impl AssetBaseCallbacks for Asset {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base.base.notify_property_changed(property_key);
    }
}

impl std::ops::Deref for Asset {
    type Target = AssetBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Asset {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
