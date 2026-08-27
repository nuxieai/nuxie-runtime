use crate::mechanical_port::source::{
    artboard::Artboard, core::binary_reader::BinaryReader, layout_component::LayoutComponent,
};

pub trait ArtboardBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn origin_x_changed(&mut self) {}
    fn origin_y_changed(&mut self) {}
    fn default_state_machine_id_changed(&mut self) {}
    fn view_model_id_changed(&mut self) {}
}

pub struct ArtboardBase {
    pub base: LayoutComponent,
    origin_x: f32,
    origin_y: f32,
    default_state_machine_id: u32,
    view_model_id: u32,
}

impl Default for ArtboardBase {
    fn default() -> Self {
        Self {
            base: LayoutComponent::default(),
            origin_x: 0.0,
            origin_y: 0.0,
            default_state_machine_id: u32::MAX,
            view_model_id: u32::MAX,
        }
    }
}

impl ArtboardBase {
    pub const TYPE_KEY: u16 = 1;
    pub const ORIGIN_X_PROPERTY_KEY: u16 = 11;
    pub const ORIGIN_Y_PROPERTY_KEY: u16 = 12;
    pub const DEFAULT_STATE_MACHINE_ID_PROPERTY_KEY: u16 = 236;
    pub const VIEW_MODEL_ID_PROPERTY_KEY: u16 = 583;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 409 | 13 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn origin_x(&self) -> f32 {
        self.origin_x
    }
    pub fn set_origin_x(&mut self, value: f32, callbacks: &mut impl ArtboardBaseCallbacks) {
        if self.origin_x == value {
            return;
        }
        self.origin_x = value;
        callbacks.origin_x_changed();
        callbacks.notify_property_changed(Self::ORIGIN_X_PROPERTY_KEY);
    }
    pub fn origin_y(&self) -> f32 {
        self.origin_y
    }
    pub fn set_origin_y(&mut self, value: f32, callbacks: &mut impl ArtboardBaseCallbacks) {
        if self.origin_y == value {
            return;
        }
        self.origin_y = value;
        callbacks.origin_y_changed();
        callbacks.notify_property_changed(Self::ORIGIN_Y_PROPERTY_KEY);
    }
    pub fn default_state_machine_id(&self) -> u32 {
        self.default_state_machine_id
    }
    pub fn set_default_state_machine_id(
        &mut self,
        value: u32,
        callbacks: &mut impl ArtboardBaseCallbacks,
    ) {
        if self.default_state_machine_id == value {
            return;
        }
        self.default_state_machine_id = value;
        callbacks.default_state_machine_id_changed();
        callbacks.notify_property_changed(Self::DEFAULT_STATE_MACHINE_ID_PROPERTY_KEY);
    }
    pub fn view_model_id(&self) -> u32 {
        self.view_model_id
    }
    pub fn set_view_model_id(&mut self, value: u32, callbacks: &mut impl ArtboardBaseCallbacks) {
        if self.view_model_id == value {
            return;
        }
        self.view_model_id = value;
        callbacks.view_model_id_changed();
        callbacks.notify_property_changed(Self::VIEW_MODEL_ID_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl ArtboardBaseCallbacks) -> Artboard {
        let mut cloned = Artboard::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ArtboardBaseCallbacks) {
        self.origin_x = object.origin_x;
        self.origin_y = object.origin_y;
        self.default_state_machine_id = object.default_state_machine_id;
        self.view_model_id = object.view_model_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ArtboardBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::ORIGIN_X_PROPERTY_KEY => {
                self.origin_x = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::ORIGIN_Y_PROPERTY_KEY => {
                self.origin_y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::DEFAULT_STATE_MACHINE_ID_PROPERTY_KEY => {
                self.default_state_machine_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::VIEW_MODEL_ID_PROPERTY_KEY => {
                self.view_model_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
