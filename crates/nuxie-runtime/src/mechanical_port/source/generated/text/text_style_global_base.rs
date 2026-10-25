use crate::source::{
    core::binary_reader::BinaryReader, text::text_style_global::TextStyleGlobal,
    text::text_style_reference::TextStyleReference,
};
pub trait TextStyleGlobalBaseCallbacks:
    crate::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn artboard_id_changed(&mut self) {}
}
pub struct TextStyleGlobalBase {
    pub base: TextStyleReference,
    artboard_id: u32,
}
impl Default for TextStyleGlobalBase {
    fn default() -> Self {
        Self {
            base: TextStyleReference::default(),
            artboard_id: u32::MAX,
        }
    }
}
impl TextStyleGlobalBase {
    pub const TYPE_KEY: u16 = 1101;
    pub const ARTBOARD_ID_PROPERTY_KEY: u16 = 493;
    pub fn is_type_of(type_key: u16) -> bool {
        type_key == Self::TYPE_KEY
            || crate::source::generated::core_type_tree::has_ancestor(Self::TYPE_KEY, type_key)
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
        callbacks: &mut impl TextStyleGlobalBaseCallbacks,
    ) {
        if self.set_artboard_id_value(value) {
            callbacks.artboard_id_changed();
            TextStyleGlobalBaseCallbacks::notify_property_changed(
                callbacks,
                Self::ARTBOARD_ID_PROPERTY_KEY,
            );
        }
    }
    pub(crate) fn set_artboard_id_value(&mut self, value: u32) -> bool {
        if self.artboard_id == value {
            return false;
        }
        self.artboard_id = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl TextStyleGlobalBaseCallbacks) -> TextStyleGlobal {
        let mut cloned = TextStyleGlobal::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl TextStyleGlobalBaseCallbacks) {
        self.artboard_id = object.artboard_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TextStyleGlobalBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::ARTBOARD_ID_PROPERTY_KEY => {
                self.artboard_id =
                    crate::source::core::field_types::core_id_type::CoreIdType::runtime_deserialize(
                        reader,
                    );
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
impl std::ops::Deref for TextStyleGlobalBase {
    type Target = TextStyleReference;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for TextStyleGlobalBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
