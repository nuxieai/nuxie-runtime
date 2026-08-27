use crate::mechanical_port::source::{
    artboard_component_list::ArtboardComponentList, core::binary_reader::BinaryReader,
    drawable::Drawable,
};

pub trait ArtboardComponentListBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn list_source_changed(&mut self) {}
}

pub struct ArtboardComponentListBase {
    pub base: Drawable,
    list_source: u32,
}

impl Default for ArtboardComponentListBase {
    fn default() -> Self {
        Self {
            base: Drawable::default(),
            list_source: u32::MAX,
        }
    }
}

impl ArtboardComponentListBase {
    pub const TYPE_KEY: u16 = 559;
    pub const LIST_SOURCE_PROPERTY_KEY: u16 = 800;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 13 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn list_source(&self) -> u32 {
        self.list_source
    }
    pub fn set_list_source(
        &mut self,
        value: u32,
        callbacks: &mut impl ArtboardComponentListBaseCallbacks,
    ) {
        if self.list_source == value {
            return;
        }
        self.list_source = value;
        callbacks.list_source_changed();
        callbacks.notify_property_changed(Self::LIST_SOURCE_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl ArtboardComponentListBaseCallbacks,
    ) -> ArtboardComponentList {
        let mut cloned = ArtboardComponentList::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ArtboardComponentListBaseCallbacks) {
        self.list_source = object.list_source;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ArtboardComponentListBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::LIST_SOURCE_PROPERTY_KEY => {
                self.list_source = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
