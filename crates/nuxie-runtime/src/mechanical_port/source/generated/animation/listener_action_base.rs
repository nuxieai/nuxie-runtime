use crate::mechanical_port::source::{core::Core, core::binary_reader::BinaryReader};

pub trait ListenerActionBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn flags_changed(&mut self) {}
}

pub struct ListenerActionBase {
    pub base: Core,
    flags: u32,
}

impl Default for ListenerActionBase {
    fn default() -> Self {
        Self {
            base: Core::default(),
            flags: 0,
        }
    }
}

impl ListenerActionBase {
    pub const TYPE_KEY: u16 = 125;
    pub const FLAGS_PROPERTY_KEY: u16 = 980;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn flags(&self) -> u32 {
        self.flags
    }
    pub fn set_flags(&mut self, value: u32, callbacks: &mut impl ListenerActionBaseCallbacks) {
        if self.flags == value {
            return;
        }
        self.flags = value;
        callbacks.flags_changed();
        callbacks.notify_property_changed(Self::FLAGS_PROPERTY_KEY);
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ListenerActionBaseCallbacks) {
        self.flags = object.flags;
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ListenerActionBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::FLAGS_PROPERTY_KEY => {
                self.flags = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => false,
        }
    }
}
