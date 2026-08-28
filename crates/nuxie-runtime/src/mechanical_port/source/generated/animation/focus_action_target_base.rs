use crate::mechanical_port::source::{
    animation::focus_action::FocusAction, animation::focus_action_target::FocusActionTarget,
    core::binary_reader::BinaryReader,
};

pub trait FocusActionTargetBaseCallbacks: crate::mechanical_port::source::generated::animation::listener_action_base::ListenerActionBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn target_id_changed(&mut self) {}
}

pub struct FocusActionTargetBase {
    pub base: FocusAction,
    target_id: u32,
}

impl Default for FocusActionTargetBase {
    fn default() -> Self {
        Self {
            base: FocusAction::default(),
            target_id: u32::MAX,
        }
    }
}

impl FocusActionTargetBase {
    pub const TYPE_KEY: u16 = 652;
    pub const TARGET_ID_PROPERTY_KEY: u16 = 952;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 671 | 125)
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
        callbacks: &mut impl FocusActionTargetBaseCallbacks,
    ) {
        if !self.set_target_id_value(value) {
            return;
        }
        callbacks.target_id_changed();
        callbacks.notify_property_changed(Self::TARGET_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_target_id_value(&mut self, value: u32) -> bool {
        if self.target_id == value {
            return false;
        }
        self.target_id = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl FocusActionTargetBaseCallbacks,
    ) -> FocusActionTarget {
        let mut cloned = FocusActionTarget::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl FocusActionTargetBaseCallbacks) {
        self.target_id = object.target_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl FocusActionTargetBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::TARGET_ID_PROPERTY_KEY => {
                self.target_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for FocusActionTargetBase {
    type Target = FocusAction;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for FocusActionTargetBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
