use crate::mechanical_port::source::{
    animation::listener_action::ListenerAction,
    animation::listener_align_target::ListenerAlignTarget, core::binary_reader::BinaryReader,
};

pub trait ListenerAlignTargetBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn target_id_changed(&mut self) {}
    fn preserve_offset_changed(&mut self) {}
}

pub struct ListenerAlignTargetBase {
    pub base: ListenerAction,
    target_id: u32,
    preserve_offset: bool,
}

impl Default for ListenerAlignTargetBase {
    fn default() -> Self {
        Self {
            base: ListenerAction::default(),
            target_id: u32::MAX,
            preserve_offset: false,
        }
    }
}

impl ListenerAlignTargetBase {
    pub const TYPE_KEY: u16 = 126;
    pub const TARGET_ID_PROPERTY_KEY: u16 = 240;
    pub const PRESERVE_OFFSET_PROPERTY_KEY: u16 = 541;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 125)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn target_id(&self) -> u32 {
        self.target_id
    }
    pub fn set_target_id(
        &mut self,
        value: u32,
        callbacks: &mut impl ListenerAlignTargetBaseCallbacks,
    ) {
        if self.target_id == value {
            return;
        }
        self.target_id = value;
        callbacks.target_id_changed();
        callbacks.notify_property_changed(Self::TARGET_ID_PROPERTY_KEY);
    }
    pub fn preserve_offset(&self) -> bool {
        self.preserve_offset
    }
    pub fn set_preserve_offset(
        &mut self,
        value: bool,
        callbacks: &mut impl ListenerAlignTargetBaseCallbacks,
    ) {
        if self.preserve_offset == value {
            return;
        }
        self.preserve_offset = value;
        callbacks.preserve_offset_changed();
        callbacks.notify_property_changed(Self::PRESERVE_OFFSET_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl ListenerAlignTargetBaseCallbacks,
    ) -> ListenerAlignTarget {
        let mut cloned = ListenerAlignTarget::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ListenerAlignTargetBaseCallbacks) {
        self.target_id = object.target_id;
        self.preserve_offset = object.preserve_offset;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ListenerAlignTargetBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::TARGET_ID_PROPERTY_KEY => {
                self.target_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::PRESERVE_OFFSET_PROPERTY_KEY => {
                self.preserve_offset = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
