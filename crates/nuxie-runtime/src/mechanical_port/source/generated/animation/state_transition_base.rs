use crate::mechanical_port::source::{
    animation::state_machine_layer_component::StateMachineLayerComponent,
    animation::state_transition::StateTransition, core::binary_reader::BinaryReader,
};

pub trait StateTransitionBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn state_to_id_changed(&mut self) {}
    fn flags_changed(&mut self) {}
    fn duration_changed(&mut self) {}
    fn exit_time_changed(&mut self) {}
    fn interpolation_type_changed(&mut self) {}
    fn interpolator_id_changed(&mut self) {}
    fn random_weight_changed(&mut self) {}
}

pub struct StateTransitionBase {
    pub base: StateMachineLayerComponent,
    state_to_id: u32,
    flags: u32,
    duration: u32,
    exit_time: u32,
    interpolation_type: u32,
    interpolator_id: u32,
    random_weight: u32,
}

impl Default for StateTransitionBase {
    fn default() -> Self {
        Self {
            base: StateMachineLayerComponent::default(),
            state_to_id: u32::MAX,
            flags: 0,
            duration: 0,
            exit_time: 0,
            interpolation_type: 1,
            interpolator_id: u32::MAX,
            random_weight: 1,
        }
    }
}

impl StateTransitionBase {
    pub const TYPE_KEY: u16 = 65;
    pub const STATE_TO_ID_PROPERTY_KEY: u16 = 151;
    pub const FLAGS_PROPERTY_KEY: u16 = 152;
    pub const DURATION_PROPERTY_KEY: u16 = 158;
    pub const EXIT_TIME_PROPERTY_KEY: u16 = 160;
    pub const INTERPOLATION_TYPE_PROPERTY_KEY: u16 = 349;
    pub const INTERPOLATOR_ID_PROPERTY_KEY: u16 = 350;
    pub const RANDOM_WEIGHT_PROPERTY_KEY: u16 = 537;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 66)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn state_to_id(&self) -> u32 {
        self.state_to_id
    }
    pub fn set_state_to_id(
        &mut self,
        value: u32,
        callbacks: &mut impl StateTransitionBaseCallbacks,
    ) {
        if !self.set_state_to_id_value(value) {
            return;
        }
        callbacks.state_to_id_changed();
        callbacks.notify_property_changed(Self::STATE_TO_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_state_to_id_value(&mut self, value: u32) -> bool {
        if self.state_to_id == value {
            return false;
        }
        self.state_to_id = value;
        true
    }
    pub fn flags(&self) -> u32 {
        self.flags
    }
    pub fn set_flags(&mut self, value: u32, callbacks: &mut impl StateTransitionBaseCallbacks) {
        if !self.set_flags_value(value) {
            return;
        }
        callbacks.flags_changed();
        callbacks.notify_property_changed(Self::FLAGS_PROPERTY_KEY);
    }

    pub(crate) fn set_flags_value(&mut self, value: u32) -> bool {
        if self.flags == value {
            return false;
        }
        self.flags = value;
        true
    }
    pub fn duration(&self) -> u32 {
        self.duration
    }
    pub fn set_duration(&mut self, value: u32, callbacks: &mut impl StateTransitionBaseCallbacks) {
        if !self.set_duration_value(value) {
            return;
        }
        callbacks.duration_changed();
        callbacks.notify_property_changed(Self::DURATION_PROPERTY_KEY);
    }

    pub(crate) fn set_duration_value(&mut self, value: u32) -> bool {
        if self.duration == value {
            return false;
        }
        self.duration = value;
        true
    }
    pub fn exit_time(&self) -> u32 {
        self.exit_time
    }
    pub fn set_exit_time(&mut self, value: u32, callbacks: &mut impl StateTransitionBaseCallbacks) {
        if !self.set_exit_time_value(value) {
            return;
        }
        callbacks.exit_time_changed();
        callbacks.notify_property_changed(Self::EXIT_TIME_PROPERTY_KEY);
    }

    pub(crate) fn set_exit_time_value(&mut self, value: u32) -> bool {
        if self.exit_time == value {
            return false;
        }
        self.exit_time = value;
        true
    }
    pub fn interpolation_type(&self) -> u32 {
        self.interpolation_type
    }
    pub fn set_interpolation_type(
        &mut self,
        value: u32,
        callbacks: &mut impl StateTransitionBaseCallbacks,
    ) {
        if !self.set_interpolation_type_value(value) {
            return;
        }
        callbacks.interpolation_type_changed();
        callbacks.notify_property_changed(Self::INTERPOLATION_TYPE_PROPERTY_KEY);
    }

    pub(crate) fn set_interpolation_type_value(&mut self, value: u32) -> bool {
        if self.interpolation_type == value {
            return false;
        }
        self.interpolation_type = value;
        true
    }
    pub fn interpolator_id(&self) -> u32 {
        self.interpolator_id
    }
    pub fn set_interpolator_id(
        &mut self,
        value: u32,
        callbacks: &mut impl StateTransitionBaseCallbacks,
    ) {
        if !self.set_interpolator_id_value(value) {
            return;
        }
        callbacks.interpolator_id_changed();
        callbacks.notify_property_changed(Self::INTERPOLATOR_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_interpolator_id_value(&mut self, value: u32) -> bool {
        if self.interpolator_id == value {
            return false;
        }
        self.interpolator_id = value;
        true
    }
    pub fn random_weight(&self) -> u32 {
        self.random_weight
    }
    pub fn set_random_weight(
        &mut self,
        value: u32,
        callbacks: &mut impl StateTransitionBaseCallbacks,
    ) {
        if !self.set_random_weight_value(value) {
            return;
        }
        callbacks.random_weight_changed();
        callbacks.notify_property_changed(Self::RANDOM_WEIGHT_PROPERTY_KEY);
    }

    pub(crate) fn set_random_weight_value(&mut self, value: u32) -> bool {
        if self.random_weight == value {
            return false;
        }
        self.random_weight = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl StateTransitionBaseCallbacks) -> StateTransition {
        let mut cloned = StateTransition::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl StateTransitionBaseCallbacks) {
        self.state_to_id = object.state_to_id;
        self.flags = object.flags;
        self.duration = object.duration;
        self.exit_time = object.exit_time;
        self.interpolation_type = object.interpolation_type;
        self.interpolator_id = object.interpolator_id;
        self.random_weight = object.random_weight;
        self.base.copy(&object.base);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl StateTransitionBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::STATE_TO_ID_PROPERTY_KEY => {
                self.state_to_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::FLAGS_PROPERTY_KEY => {
                self.flags = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::DURATION_PROPERTY_KEY => {
                self.duration = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::EXIT_TIME_PROPERTY_KEY => {
                self.exit_time = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::INTERPOLATION_TYPE_PROPERTY_KEY => {
                self.interpolation_type = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::INTERPOLATOR_ID_PROPERTY_KEY => {
                self.interpolator_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::RANDOM_WEIGHT_PROPERTY_KEY => {
                self.random_weight = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader),
        }
    }
}

impl std::ops::Deref for StateTransitionBase {
    type Target = StateMachineLayerComponent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for StateTransitionBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
