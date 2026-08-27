use crate::mechanical_port::source::{
    container_component::ContainerComponent, core::binary_reader::BinaryReader,
    shapes::paint::linear_gradient::LinearGradient,
};

pub trait LinearGradientBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn start_x_changed(&mut self) {}
    fn start_y_changed(&mut self) {}
    fn end_x_changed(&mut self) {}
    fn end_y_changed(&mut self) {}
    fn opacity_changed(&mut self) {}
}

pub struct LinearGradientBase {
    pub base: ContainerComponent,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
    opacity: f32,
}

impl Default for LinearGradientBase {
    fn default() -> Self {
        Self {
            base: ContainerComponent::default(),
            start_x: 0.0,
            start_y: 0.0,
            end_x: 0.0,
            end_y: 0.0,
            opacity: 1.0,
        }
    }
}

impl LinearGradientBase {
    pub const TYPE_KEY: u16 = 22;
    pub const START_X_PROPERTY_KEY: u16 = 42;
    pub const START_Y_PROPERTY_KEY: u16 = 33;
    pub const END_X_PROPERTY_KEY: u16 = 34;
    pub const END_Y_PROPERTY_KEY: u16 = 35;
    pub const OPACITY_PROPERTY_KEY: u16 = 46;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn start_x(&self) -> f32 {
        self.start_x
    }
    pub fn set_start_x(&mut self, value: f32, callbacks: &mut impl LinearGradientBaseCallbacks) {
        if self.start_x == value {
            return;
        }
        self.start_x = value;
        callbacks.start_x_changed();
        callbacks.notify_property_changed(Self::START_X_PROPERTY_KEY);
    }
    pub fn start_y(&self) -> f32 {
        self.start_y
    }
    pub fn set_start_y(&mut self, value: f32, callbacks: &mut impl LinearGradientBaseCallbacks) {
        if self.start_y == value {
            return;
        }
        self.start_y = value;
        callbacks.start_y_changed();
        callbacks.notify_property_changed(Self::START_Y_PROPERTY_KEY);
    }
    pub fn end_x(&self) -> f32 {
        self.end_x
    }
    pub fn set_end_x(&mut self, value: f32, callbacks: &mut impl LinearGradientBaseCallbacks) {
        if self.end_x == value {
            return;
        }
        self.end_x = value;
        callbacks.end_x_changed();
        callbacks.notify_property_changed(Self::END_X_PROPERTY_KEY);
    }
    pub fn end_y(&self) -> f32 {
        self.end_y
    }
    pub fn set_end_y(&mut self, value: f32, callbacks: &mut impl LinearGradientBaseCallbacks) {
        if self.end_y == value {
            return;
        }
        self.end_y = value;
        callbacks.end_y_changed();
        callbacks.notify_property_changed(Self::END_Y_PROPERTY_KEY);
    }
    pub fn opacity(&self) -> f32 {
        self.opacity
    }
    pub fn set_opacity(&mut self, value: f32, callbacks: &mut impl LinearGradientBaseCallbacks) {
        if self.opacity == value {
            return;
        }
        self.opacity = value;
        callbacks.opacity_changed();
        callbacks.notify_property_changed(Self::OPACITY_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl LinearGradientBaseCallbacks) -> LinearGradient {
        let mut cloned = LinearGradient::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl LinearGradientBaseCallbacks) {
        self.start_x = object.start_x;
        self.start_y = object.start_y;
        self.end_x = object.end_x;
        self.end_y = object.end_y;
        self.opacity = object.opacity;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl LinearGradientBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::START_X_PROPERTY_KEY => {
                self.start_x = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::START_Y_PROPERTY_KEY => {
                self.start_y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::END_X_PROPERTY_KEY => {
                self.end_x = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::END_Y_PROPERTY_KEY => {
                self.end_y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::OPACITY_PROPERTY_KEY => {
                self.opacity = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
