use crate::mechanical_port::source::{
    animation::advanceable_state::AdvanceableState, animation::animation_state::AnimationState,
    core::binary_reader::BinaryReader,
};

pub trait AnimationStateBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn animation_id_changed(&mut self) {}
}

pub struct AnimationStateBase {
    pub base: AdvanceableState,
    animation_id: u32,
}

impl Default for AnimationStateBase {
    fn default() -> Self {
        Self {
            base: AdvanceableState::default(),
            animation_id: u32::MAX,
        }
    }
}

impl AnimationStateBase {
    pub const TYPE_KEY: u16 = 61;
    pub const ANIMATION_ID_PROPERTY_KEY: u16 = 149;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 145 | 60 | 66)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn animation_id(&self) -> u32 {
        self.animation_id
    }
    pub fn set_animation_id(
        &mut self,
        value: u32,
        callbacks: &mut impl AnimationStateBaseCallbacks,
    ) {
        if self.animation_id == value {
            return;
        }
        self.animation_id = value;
        callbacks.animation_id_changed();
        callbacks.notify_property_changed(Self::ANIMATION_ID_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl AnimationStateBaseCallbacks) -> AnimationState {
        let mut cloned = AnimationState::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl AnimationStateBaseCallbacks) {
        self.animation_id = object.animation_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl AnimationStateBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::ANIMATION_ID_PROPERTY_KEY => {
                self.animation_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
