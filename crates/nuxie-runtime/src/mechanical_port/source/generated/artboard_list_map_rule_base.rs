use crate::mechanical_port::source::{
    artboard_list_map_rule::ArtboardListMapRule, component::Component,
    core::binary_reader::BinaryReader,
};

pub trait ArtboardListMapRuleBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn artboard_id_changed(&mut self) {}
    fn view_model_id_changed(&mut self) {}
}

pub struct ArtboardListMapRuleBase {
    pub base: Component,
    artboard_id: u32,
    view_model_id: u32,
}

impl Default for ArtboardListMapRuleBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            artboard_id: u32::MAX,
            view_model_id: u32::MAX,
        }
    }
}

impl ArtboardListMapRuleBase {
    pub const TYPE_KEY: u16 = 648;
    pub const ARTBOARD_ID_PROPERTY_KEY: u16 = 934;
    pub const VIEW_MODEL_ID_PROPERTY_KEY: u16 = 935;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn artboard_id(&self) -> u32 {
        self.artboard_id
    }
    pub fn set_artboard_id(
        &mut self,
        value: u32,
        callbacks: &mut impl ArtboardListMapRuleBaseCallbacks,
    ) {
        if !self.set_artboard_id_value(value) {
            return;
        }
        callbacks.artboard_id_changed();
        callbacks.notify_property_changed(Self::ARTBOARD_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_artboard_id_value(&mut self, value: u32) -> bool {
        if self.artboard_id == value {
            return false;
        }
        self.artboard_id = value;
        true
    }
    pub fn view_model_id(&self) -> u32 {
        self.view_model_id
    }
    pub fn set_view_model_id(
        &mut self,
        value: u32,
        callbacks: &mut impl ArtboardListMapRuleBaseCallbacks,
    ) {
        if !self.set_view_model_id_value(value) {
            return;
        }
        callbacks.view_model_id_changed();
        callbacks.notify_property_changed(Self::VIEW_MODEL_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_view_model_id_value(&mut self, value: u32) -> bool {
        if self.view_model_id == value {
            return false;
        }
        self.view_model_id = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl ArtboardListMapRuleBaseCallbacks,
    ) -> ArtboardListMapRule {
        let mut cloned = ArtboardListMapRule::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ArtboardListMapRuleBaseCallbacks) {
        self.artboard_id = object.artboard_id;
        self.view_model_id = object.view_model_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ArtboardListMapRuleBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::ARTBOARD_ID_PROPERTY_KEY => {
                self.artboard_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::VIEW_MODEL_ID_PROPERTY_KEY => {
                self.view_model_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for ArtboardListMapRuleBase {
    type Target = Component;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ArtboardListMapRuleBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
