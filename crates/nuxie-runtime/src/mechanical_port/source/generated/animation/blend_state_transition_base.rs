use crate::mechanical_port::source::{
    animation::blend_state_transition::BlendStateTransition,
    animation::state_transition::StateTransition, core::binary_reader::BinaryReader,
};

pub trait BlendStateTransitionBaseCallbacks: crate::mechanical_port::source::generated::animation::state_transition_base::StateTransitionBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn exit_blend_animation_id_changed(&mut self) {}
}

pub struct BlendStateTransitionBase {
    pub base: StateTransition,
    exit_blend_animation_id: u32,
}

impl Default for BlendStateTransitionBase {
    fn default() -> Self {
        Self {
            base: StateTransition::default(),
            exit_blend_animation_id: u32::MAX,
        }
    }
}

impl BlendStateTransitionBase {
    pub const TYPE_KEY: u16 = 78;
    pub const EXIT_BLEND_ANIMATION_ID_PROPERTY_KEY: u16 = 171;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 65 | 66)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn exit_blend_animation_id(&self) -> u32 {
        self.exit_blend_animation_id
    }
    pub fn set_exit_blend_animation_id(
        &mut self,
        value: u32,
        callbacks: &mut impl BlendStateTransitionBaseCallbacks,
    ) {
        if !self.set_exit_blend_animation_id_value(value) {
            return;
        }
        callbacks.exit_blend_animation_id_changed();
        BlendStateTransitionBaseCallbacks::notify_property_changed(
            callbacks,
            Self::EXIT_BLEND_ANIMATION_ID_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_exit_blend_animation_id_value(&mut self, value: u32) -> bool {
        if self.exit_blend_animation_id == value {
            return false;
        }
        self.exit_blend_animation_id = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl BlendStateTransitionBaseCallbacks,
    ) -> BlendStateTransition {
        let mut cloned = BlendStateTransition::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl BlendStateTransitionBaseCallbacks) {
        self.exit_blend_animation_id = object.exit_blend_animation_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl BlendStateTransitionBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::EXIT_BLEND_ANIMATION_ID_PROPERTY_KEY => {
                self.exit_blend_animation_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for BlendStateTransitionBase {
    type Target = StateTransition;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for BlendStateTransitionBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
