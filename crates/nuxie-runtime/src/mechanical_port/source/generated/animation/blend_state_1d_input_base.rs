use crate::mechanical_port::source::animation::blend_state_1d_input::BlendState1DInput;

use crate::mechanical_port::source::{
    animation::blend_state_1d::BlendState1D, core::binary_reader::BinaryReader,
};

pub trait BlendState1DInputBaseCallbacks:
    crate::mechanical_port::source::generated::animation::layer_state_base::LayerStateBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn input_id_changed(&mut self) {}
}

pub struct BlendState1DInputBase {
    pub base: BlendState1D,
    input_id: u32,
}

impl Default for BlendState1DInputBase {
    fn default() -> Self {
        Self {
            base: BlendState1D::default(),
            input_id: u32::MAX,
        }
    }
}

impl BlendState1DInputBase {
    pub const TYPE_KEY: u16 = 76;
    pub const INPUT_ID_PROPERTY_KEY: u16 = 167;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 527 | 72 | 60 | 66)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn input_id(&self) -> u32 {
        self.input_id
    }
    pub fn set_input_id(
        &mut self,
        value: u32,
        callbacks: &mut impl BlendState1DInputBaseCallbacks,
    ) {
        if !self.set_input_id_value(value) {
            return;
        }
        callbacks.input_id_changed();
        BlendState1DInputBaseCallbacks::notify_property_changed(
            callbacks,
            Self::INPUT_ID_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_input_id_value(&mut self, value: u32) -> bool {
        if self.input_id == value {
            return false;
        }
        self.input_id = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl BlendState1DInputBaseCallbacks,
    ) -> BlendState1DInput {
        let mut cloned = BlendState1DInput::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl BlendState1DInputBaseCallbacks) {
        self.input_id = object.input_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl BlendState1DInputBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::INPUT_ID_PROPERTY_KEY => {
                self.input_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for BlendState1DInputBase {
    type Target = BlendState1D;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for BlendState1DInputBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
