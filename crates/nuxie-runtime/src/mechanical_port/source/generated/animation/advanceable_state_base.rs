use crate::mechanical_port::source::{
    animation::layer_state::LayerState, core::binary_reader::BinaryReader,
};

pub trait AdvanceableStateBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn speed_changed(&mut self) {}
}

pub struct AdvanceableStateBase {
    pub base: LayerState,
    speed: f32,
}

impl Default for AdvanceableStateBase {
    fn default() -> Self {
        Self {
            base: LayerState::default(),
            speed: 1.0,
        }
    }
}

impl AdvanceableStateBase {
    pub const TYPE_KEY: u16 = 145;
    pub const SPEED_PROPERTY_KEY: u16 = 292;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 60 | 66)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn speed(&self) -> f32 {
        self.speed
    }
    pub fn set_speed(&mut self, value: f32, callbacks: &mut impl AdvanceableStateBaseCallbacks) {
        if self.speed == value {
            return;
        }
        self.speed = value;
        callbacks.speed_changed();
        callbacks.notify_property_changed(Self::SPEED_PROPERTY_KEY);
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl AdvanceableStateBaseCallbacks) {
        self.speed = object.speed;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl AdvanceableStateBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::SPEED_PROPERTY_KEY => {
                self.speed = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
