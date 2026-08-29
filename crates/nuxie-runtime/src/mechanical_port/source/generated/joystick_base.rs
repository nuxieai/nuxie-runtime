use crate::mechanical_port::source::{
    component::Component, core::binary_reader::BinaryReader, joystick::Joystick,
};

pub trait JoystickBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn x_changed(&mut self) {}
    fn y_changed(&mut self) {}
    fn pos_x_changed(&mut self) {}
    fn pos_y_changed(&mut self) {}
    fn origin_x_changed(&mut self) {}
    fn origin_y_changed(&mut self) {}
    fn width_changed(&mut self) {}
    fn height_changed(&mut self) {}
    fn x_id_changed(&mut self) {}
    fn y_id_changed(&mut self) {}
    fn joystick_flags_changed(&mut self) {}
    fn handle_source_id_changed(&mut self) {}
}

pub struct JoystickBase {
    pub base: Component,
    x: f32,
    y: f32,
    pos_x: f32,
    pos_y: f32,
    origin_x: f32,
    origin_y: f32,
    width: f32,
    height: f32,
    x_id: u32,
    y_id: u32,
    joystick_flags: u32,
    handle_source_id: u32,
}

impl Default for JoystickBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            x: 0.0,
            y: 0.0,
            pos_x: 0.0,
            pos_y: 0.0,
            origin_x: 0.5,
            origin_y: 0.5,
            width: 100.0,
            height: 100.0,
            x_id: u32::MAX,
            y_id: u32::MAX,
            joystick_flags: 0,
            handle_source_id: u32::MAX,
        }
    }
}

impl JoystickBase {
    pub const TYPE_KEY: u16 = 148;
    pub const X_PROPERTY_KEY: u16 = 299;
    pub const Y_PROPERTY_KEY: u16 = 300;
    pub const POS_X_PROPERTY_KEY: u16 = 303;
    pub const POS_Y_PROPERTY_KEY: u16 = 304;
    pub const ORIGIN_X_PROPERTY_KEY: u16 = 307;
    pub const ORIGIN_Y_PROPERTY_KEY: u16 = 308;
    pub const WIDTH_PROPERTY_KEY: u16 = 305;
    pub const HEIGHT_PROPERTY_KEY: u16 = 306;
    pub const X_ID_PROPERTY_KEY: u16 = 301;
    pub const Y_ID_PROPERTY_KEY: u16 = 302;
    pub const JOYSTICK_FLAGS_PROPERTY_KEY: u16 = 312;
    pub const HANDLE_SOURCE_ID_PROPERTY_KEY: u16 = 313;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn x(&self) -> f32 {
        self.x
    }
    pub fn set_x(&mut self, value: f32, callbacks: &mut impl JoystickBaseCallbacks) {
        if !self.set_x_value(value) {
            return;
        }
        callbacks.x_changed();
        JoystickBaseCallbacks::notify_property_changed(callbacks, Self::X_PROPERTY_KEY);
    }

    pub(crate) fn set_x_value(&mut self, value: f32) -> bool {
        if self.x == value {
            return false;
        }
        self.x = value;
        true
    }
    pub fn y(&self) -> f32 {
        self.y
    }
    pub fn set_y(&mut self, value: f32, callbacks: &mut impl JoystickBaseCallbacks) {
        if !self.set_y_value(value) {
            return;
        }
        callbacks.y_changed();
        JoystickBaseCallbacks::notify_property_changed(callbacks, Self::Y_PROPERTY_KEY);
    }

    pub(crate) fn set_y_value(&mut self, value: f32) -> bool {
        if self.y == value {
            return false;
        }
        self.y = value;
        true
    }
    pub fn pos_x(&self) -> f32 {
        self.pos_x
    }
    pub fn set_pos_x(&mut self, value: f32, callbacks: &mut impl JoystickBaseCallbacks) {
        if !self.set_pos_x_value(value) {
            return;
        }
        callbacks.pos_x_changed();
        JoystickBaseCallbacks::notify_property_changed(callbacks, Self::POS_X_PROPERTY_KEY);
    }

    pub(crate) fn set_pos_x_value(&mut self, value: f32) -> bool {
        if self.pos_x == value {
            return false;
        }
        self.pos_x = value;
        true
    }
    pub fn pos_y(&self) -> f32 {
        self.pos_y
    }
    pub fn set_pos_y(&mut self, value: f32, callbacks: &mut impl JoystickBaseCallbacks) {
        if !self.set_pos_y_value(value) {
            return;
        }
        callbacks.pos_y_changed();
        JoystickBaseCallbacks::notify_property_changed(callbacks, Self::POS_Y_PROPERTY_KEY);
    }

    pub(crate) fn set_pos_y_value(&mut self, value: f32) -> bool {
        if self.pos_y == value {
            return false;
        }
        self.pos_y = value;
        true
    }
    pub fn origin_x(&self) -> f32 {
        self.origin_x
    }
    pub fn set_origin_x(&mut self, value: f32, callbacks: &mut impl JoystickBaseCallbacks) {
        if !self.set_origin_x_value(value) {
            return;
        }
        callbacks.origin_x_changed();
        JoystickBaseCallbacks::notify_property_changed(callbacks, Self::ORIGIN_X_PROPERTY_KEY);
    }

    pub(crate) fn set_origin_x_value(&mut self, value: f32) -> bool {
        if self.origin_x == value {
            return false;
        }
        self.origin_x = value;
        true
    }
    pub fn origin_y(&self) -> f32 {
        self.origin_y
    }
    pub fn set_origin_y(&mut self, value: f32, callbacks: &mut impl JoystickBaseCallbacks) {
        if !self.set_origin_y_value(value) {
            return;
        }
        callbacks.origin_y_changed();
        JoystickBaseCallbacks::notify_property_changed(callbacks, Self::ORIGIN_Y_PROPERTY_KEY);
    }

    pub(crate) fn set_origin_y_value(&mut self, value: f32) -> bool {
        if self.origin_y == value {
            return false;
        }
        self.origin_y = value;
        true
    }
    pub fn width(&self) -> f32 {
        self.width
    }
    pub fn set_width(&mut self, value: f32, callbacks: &mut impl JoystickBaseCallbacks) {
        if !self.set_width_value(value) {
            return;
        }
        callbacks.width_changed();
        JoystickBaseCallbacks::notify_property_changed(callbacks, Self::WIDTH_PROPERTY_KEY);
    }

    pub(crate) fn set_width_value(&mut self, value: f32) -> bool {
        if self.width == value {
            return false;
        }
        self.width = value;
        true
    }
    pub fn height(&self) -> f32 {
        self.height
    }
    pub fn set_height(&mut self, value: f32, callbacks: &mut impl JoystickBaseCallbacks) {
        if !self.set_height_value(value) {
            return;
        }
        callbacks.height_changed();
        JoystickBaseCallbacks::notify_property_changed(callbacks, Self::HEIGHT_PROPERTY_KEY);
    }

    pub(crate) fn set_height_value(&mut self, value: f32) -> bool {
        if self.height == value {
            return false;
        }
        self.height = value;
        true
    }
    pub fn x_id(&self) -> u32 {
        self.x_id
    }
    pub fn set_x_id(&mut self, value: u32, callbacks: &mut impl JoystickBaseCallbacks) {
        if !self.set_x_id_value(value) {
            return;
        }
        callbacks.x_id_changed();
        JoystickBaseCallbacks::notify_property_changed(callbacks, Self::X_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_x_id_value(&mut self, value: u32) -> bool {
        if self.x_id == value {
            return false;
        }
        self.x_id = value;
        true
    }
    pub fn y_id(&self) -> u32 {
        self.y_id
    }
    pub fn set_y_id(&mut self, value: u32, callbacks: &mut impl JoystickBaseCallbacks) {
        if !self.set_y_id_value(value) {
            return;
        }
        callbacks.y_id_changed();
        JoystickBaseCallbacks::notify_property_changed(callbacks, Self::Y_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_y_id_value(&mut self, value: u32) -> bool {
        if self.y_id == value {
            return false;
        }
        self.y_id = value;
        true
    }
    pub fn joystick_flags(&self) -> u32 {
        self.joystick_flags
    }
    pub fn set_joystick_flags(&mut self, value: u32, callbacks: &mut impl JoystickBaseCallbacks) {
        if !self.set_joystick_flags_value(value) {
            return;
        }
        callbacks.joystick_flags_changed();
        JoystickBaseCallbacks::notify_property_changed(
            callbacks,
            Self::JOYSTICK_FLAGS_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_joystick_flags_value(&mut self, value: u32) -> bool {
        if self.joystick_flags == value {
            return false;
        }
        self.joystick_flags = value;
        true
    }
    pub fn handle_source_id(&self) -> u32 {
        self.handle_source_id
    }
    pub fn set_handle_source_id(&mut self, value: u32, callbacks: &mut impl JoystickBaseCallbacks) {
        if !self.set_handle_source_id_value(value) {
            return;
        }
        callbacks.handle_source_id_changed();
        JoystickBaseCallbacks::notify_property_changed(
            callbacks,
            Self::HANDLE_SOURCE_ID_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_handle_source_id_value(&mut self, value: u32) -> bool {
        if self.handle_source_id == value {
            return false;
        }
        self.handle_source_id = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl JoystickBaseCallbacks) -> Joystick {
        let mut cloned = Joystick::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl JoystickBaseCallbacks) {
        self.x = object.x;
        self.y = object.y;
        self.pos_x = object.pos_x;
        self.pos_y = object.pos_y;
        self.origin_x = object.origin_x;
        self.origin_y = object.origin_y;
        self.width = object.width;
        self.height = object.height;
        self.x_id = object.x_id;
        self.y_id = object.y_id;
        self.joystick_flags = object.joystick_flags;
        self.handle_source_id = object.handle_source_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl JoystickBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::X_PROPERTY_KEY => {
                self.x = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::Y_PROPERTY_KEY => {
                self.y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::POS_X_PROPERTY_KEY => {
                self.pos_x = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::POS_Y_PROPERTY_KEY => {
                self.pos_y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::ORIGIN_X_PROPERTY_KEY => {
                self.origin_x = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::ORIGIN_Y_PROPERTY_KEY => {
                self.origin_y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::WIDTH_PROPERTY_KEY => {
                self.width = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::HEIGHT_PROPERTY_KEY => {
                self.height = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::X_ID_PROPERTY_KEY => {
                self.x_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::Y_ID_PROPERTY_KEY => {
                self.y_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::JOYSTICK_FLAGS_PROPERTY_KEY => {
                self.joystick_flags = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::HANDLE_SOURCE_ID_PROPERTY_KEY => {
                self.handle_source_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for JoystickBase {
    type Target = Component;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for JoystickBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
