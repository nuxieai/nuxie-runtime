use crate::mechanical_port::source::{core::binary_reader::BinaryReader, shapes::path::Path};

pub trait PointsCommonPathBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn is_closed_changed(&mut self) {}
}

pub struct PointsCommonPathBase {
    pub base: Path,
    is_closed: bool,
}

impl Default for PointsCommonPathBase {
    fn default() -> Self {
        Self {
            base: Path::default(),
            is_closed: false,
        }
    }
}

impl PointsCommonPathBase {
    pub const TYPE_KEY: u16 = 620;
    pub const IS_CLOSED_PROPERTY_KEY: u16 = 32;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 12 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn is_closed(&self) -> bool {
        self.is_closed
    }
    pub fn set_is_closed(
        &mut self,
        value: bool,
        callbacks: &mut impl PointsCommonPathBaseCallbacks,
    ) {
        if self.is_closed == value {
            return;
        }
        self.is_closed = value;
        callbacks.is_closed_changed();
        callbacks.notify_property_changed(Self::IS_CLOSED_PROPERTY_KEY);
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl PointsCommonPathBaseCallbacks) {
        self.is_closed = object.is_closed;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl PointsCommonPathBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::IS_CLOSED_PROPERTY_KEY => {
                self.is_closed = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
