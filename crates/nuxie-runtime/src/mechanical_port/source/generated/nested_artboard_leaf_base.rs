use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, nested_artboard::NestedArtboard,
    nested_artboard_leaf::NestedArtboardLeaf,
};

pub trait NestedArtboardLeafBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn fit_changed(&mut self) {}
    fn alignment_x_changed(&mut self) {}
    fn alignment_y_changed(&mut self) {}
}

pub struct NestedArtboardLeafBase {
    pub base: NestedArtboard,
    fit: u32,
    alignment_x: f32,
    alignment_y: f32,
}

impl Default for NestedArtboardLeafBase {
    fn default() -> Self {
        Self {
            base: NestedArtboard::default(),
            fit: 0,
            alignment_x: 0.0,
            alignment_y: 0.0,
        }
    }
}

impl NestedArtboardLeafBase {
    pub const TYPE_KEY: u16 = 451;
    pub const FIT_PROPERTY_KEY: u16 = 538;
    pub const ALIGNMENT_X_PROPERTY_KEY: u16 = 644;
    pub const ALIGNMENT_Y_PROPERTY_KEY: u16 = 645;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 92 | 13 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn fit(&self) -> u32 {
        self.fit
    }
    pub fn set_fit(&mut self, value: u32, callbacks: &mut impl NestedArtboardLeafBaseCallbacks) {
        if self.fit == value {
            return;
        }
        self.fit = value;
        callbacks.fit_changed();
        callbacks.notify_property_changed(Self::FIT_PROPERTY_KEY);
    }
    pub fn alignment_x(&self) -> f32 {
        self.alignment_x
    }
    pub fn set_alignment_x(
        &mut self,
        value: f32,
        callbacks: &mut impl NestedArtboardLeafBaseCallbacks,
    ) {
        if self.alignment_x == value {
            return;
        }
        self.alignment_x = value;
        callbacks.alignment_x_changed();
        callbacks.notify_property_changed(Self::ALIGNMENT_X_PROPERTY_KEY);
    }
    pub fn alignment_y(&self) -> f32 {
        self.alignment_y
    }
    pub fn set_alignment_y(
        &mut self,
        value: f32,
        callbacks: &mut impl NestedArtboardLeafBaseCallbacks,
    ) {
        if self.alignment_y == value {
            return;
        }
        self.alignment_y = value;
        callbacks.alignment_y_changed();
        callbacks.notify_property_changed(Self::ALIGNMENT_Y_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl NestedArtboardLeafBaseCallbacks,
    ) -> NestedArtboardLeaf {
        let mut cloned = NestedArtboardLeaf::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl NestedArtboardLeafBaseCallbacks) {
        self.fit = object.fit;
        self.alignment_x = object.alignment_x;
        self.alignment_y = object.alignment_y;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl NestedArtboardLeafBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::FIT_PROPERTY_KEY => {
                self.fit = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::ALIGNMENT_X_PROPERTY_KEY => {
                self.alignment_x = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::ALIGNMENT_Y_PROPERTY_KEY => {
                self.alignment_y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
