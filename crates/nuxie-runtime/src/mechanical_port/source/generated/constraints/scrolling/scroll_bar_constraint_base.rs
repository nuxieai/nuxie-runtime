use crate::mechanical_port::source::{
    constraints::draggable_constraint::DraggableConstraint,
    constraints::scrolling::scroll_bar_constraint::ScrollBarConstraint,
    core::binary_reader::BinaryReader,
};

pub trait ScrollBarConstraintBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn scroll_constraint_id_changed(&mut self) {}
    fn auto_size_changed(&mut self) {}
}

pub struct ScrollBarConstraintBase {
    pub base: DraggableConstraint,
    scroll_constraint_id: u32,
    auto_size: bool,
}

impl Default for ScrollBarConstraintBase {
    fn default() -> Self {
        Self {
            base: DraggableConstraint::default(),
            scroll_constraint_id: u32::MAX,
            auto_size: true,
        }
    }
}

impl ScrollBarConstraintBase {
    pub const TYPE_KEY: u16 = 522;
    pub const SCROLL_CONSTRAINT_ID_PROPERTY_KEY: u16 = 725;
    pub const AUTO_SIZE_PROPERTY_KEY: u16 = 734;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 520 | 79 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn scroll_constraint_id(&self) -> u32 {
        self.scroll_constraint_id
    }
    pub fn set_scroll_constraint_id(
        &mut self,
        value: u32,
        callbacks: &mut impl ScrollBarConstraintBaseCallbacks,
    ) {
        if self.scroll_constraint_id == value {
            return;
        }
        self.scroll_constraint_id = value;
        callbacks.scroll_constraint_id_changed();
        callbacks.notify_property_changed(Self::SCROLL_CONSTRAINT_ID_PROPERTY_KEY);
    }
    pub fn auto_size(&self) -> bool {
        self.auto_size
    }
    pub fn set_auto_size(
        &mut self,
        value: bool,
        callbacks: &mut impl ScrollBarConstraintBaseCallbacks,
    ) {
        if self.auto_size == value {
            return;
        }
        self.auto_size = value;
        callbacks.auto_size_changed();
        callbacks.notify_property_changed(Self::AUTO_SIZE_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl ScrollBarConstraintBaseCallbacks,
    ) -> ScrollBarConstraint {
        let mut cloned = ScrollBarConstraint::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ScrollBarConstraintBaseCallbacks) {
        self.scroll_constraint_id = object.scroll_constraint_id;
        self.auto_size = object.auto_size;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ScrollBarConstraintBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::SCROLL_CONSTRAINT_ID_PROPERTY_KEY => {
                self.scroll_constraint_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::AUTO_SIZE_PROPERTY_KEY => {
                self.auto_size = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
