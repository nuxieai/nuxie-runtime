use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, shapes::list_path::ListPath,
    shapes::points_common_path::PointsCommonPath,
};

pub trait ListPathBaseCallbacks: crate::mechanical_port::source::generated::shapes::points_common_path_base::PointsCommonPathBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn list_source_changed(&mut self) {}
}

pub struct ListPathBase {
    pub base: PointsCommonPath,
    list_source: u32,
}

impl Default for ListPathBase {
    fn default() -> Self {
        Self {
            base: PointsCommonPath::default(),
            list_source: u32::MAX,
        }
    }
}

impl ListPathBase {
    pub const TYPE_KEY: u16 = 619;
    pub const LIST_SOURCE_PROPERTY_KEY: u16 = 874;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 620 | 12 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn list_source(&self) -> u32 {
        self.list_source
    }
    pub fn set_list_source(&mut self, value: u32, callbacks: &mut impl ListPathBaseCallbacks) {
        if !self.set_list_source_value(value) {
            return;
        }
        callbacks.list_source_changed();
        ListPathBaseCallbacks::notify_property_changed(callbacks, Self::LIST_SOURCE_PROPERTY_KEY);
    }

    pub(crate) fn set_list_source_value(&mut self, value: u32) -> bool {
        if self.list_source == value {
            return false;
        }
        self.list_source = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl ListPathBaseCallbacks) -> ListPath {
        let mut cloned = ListPath::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ListPathBaseCallbacks) {
        self.list_source = object.list_source;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ListPathBaseCallbacks,
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

impl std::ops::Deref for ListPathBase {
    type Target = PointsCommonPath;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ListPathBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
