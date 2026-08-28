use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, drawable::Drawable, nested_artboard::NestedArtboard,
};

pub trait NestedArtboardBaseCallbacks:
    crate::mechanical_port::source::generated::drawable_base::DrawableBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn artboard_id_changed(&mut self) {}
    fn data_bind_path_ids_changed(&mut self) {}
    fn is_paused_changed(&mut self) {}
    fn speed_changed(&mut self) {}
    fn quantize_changed(&mut self) {}
    fn is_stateful_changed(&mut self) {}
    fn decode_data_bind_path_ids(&mut self, value: &[u8]);
}

pub struct NestedArtboardBase {
    pub base: Drawable,
    artboard_id: u32,
    is_paused: bool,
    speed: f32,
    quantize: f32,
    is_stateful: bool,
}

impl Default for NestedArtboardBase {
    fn default() -> Self {
        Self {
            base: Drawable::default(),
            artboard_id: u32::MAX,
            is_paused: false,
            speed: 1.0,
            quantize: -1.0,
            is_stateful: false,
        }
    }
}

impl NestedArtboardBase {
    pub const TYPE_KEY: u16 = 92;
    pub const ARTBOARD_ID_PROPERTY_KEY: u16 = 197;
    pub const DATA_BIND_PATH_IDS_PROPERTY_KEY: u16 = 582;
    pub const IS_PAUSED_PROPERTY_KEY: u16 = 895;
    pub const SPEED_PROPERTY_KEY: u16 = 907;
    pub const QUANTIZE_PROPERTY_KEY: u16 = 908;
    pub const IS_STATEFUL_PROPERTY_KEY: u16 = 1014;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 13 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn artboard_id(&self) -> u32 {
        self.artboard_id
    }
    pub fn set_artboard_id(
        &mut self,
        value: u32,
        callbacks: &mut impl NestedArtboardBaseCallbacks,
    ) {
        if !self.set_artboard_id_value(value) {
            return;
        }
        callbacks.artboard_id_changed();
        callbacks.notify_property_changed(Self::ARTBOARD_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_artboard_id_value(&mut self, value: u32) -> bool {
        if self.artboard_id == value {
            return false;
        }
        self.artboard_id = value;
        true
    }
    pub fn is_paused(&self) -> bool {
        self.is_paused
    }
    pub fn set_is_paused(&mut self, value: bool, callbacks: &mut impl NestedArtboardBaseCallbacks) {
        if !self.set_is_paused_value(value) {
            return;
        }
        callbacks.is_paused_changed();
        callbacks.notify_property_changed(Self::IS_PAUSED_PROPERTY_KEY);
    }

    pub(crate) fn set_is_paused_value(&mut self, value: bool) -> bool {
        if self.is_paused == value {
            return false;
        }
        self.is_paused = value;
        true
    }
    pub fn speed(&self) -> f32 {
        self.speed
    }
    pub fn set_speed(&mut self, value: f32, callbacks: &mut impl NestedArtboardBaseCallbacks) {
        if !self.set_speed_value(value) {
            return;
        }
        callbacks.speed_changed();
        callbacks.notify_property_changed(Self::SPEED_PROPERTY_KEY);
    }

    pub(crate) fn set_speed_value(&mut self, value: f32) -> bool {
        if self.speed == value {
            return false;
        }
        self.speed = value;
        true
    }
    pub fn quantize(&self) -> f32 {
        self.quantize
    }
    pub fn set_quantize(&mut self, value: f32, callbacks: &mut impl NestedArtboardBaseCallbacks) {
        if !self.set_quantize_value(value) {
            return;
        }
        callbacks.quantize_changed();
        callbacks.notify_property_changed(Self::QUANTIZE_PROPERTY_KEY);
    }

    pub(crate) fn set_quantize_value(&mut self, value: f32) -> bool {
        if self.quantize == value {
            return false;
        }
        self.quantize = value;
        true
    }
    pub fn is_stateful(&self) -> bool {
        self.is_stateful
    }
    pub fn set_is_stateful(
        &mut self,
        value: bool,
        callbacks: &mut impl NestedArtboardBaseCallbacks,
    ) {
        if !self.set_is_stateful_value(value) {
            return;
        }
        callbacks.is_stateful_changed();
        callbacks.notify_property_changed(Self::IS_STATEFUL_PROPERTY_KEY);
    }

    pub(crate) fn set_is_stateful_value(&mut self, value: bool) -> bool {
        if self.is_stateful == value {
            return false;
        }
        self.is_stateful = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl NestedArtboardBaseCallbacks) -> NestedArtboard {
        let mut cloned = NestedArtboard::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl NestedArtboardBaseCallbacks) {
        self.artboard_id = object.artboard_id;
        self.is_paused = object.is_paused;
        self.speed = object.speed;
        self.quantize = object.quantize;
        self.is_stateful = object.is_stateful;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl NestedArtboardBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::ARTBOARD_ID_PROPERTY_KEY => {
                self.artboard_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::IS_PAUSED_PROPERTY_KEY => {
                self.is_paused = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::SPEED_PROPERTY_KEY => {
                self.speed = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::QUANTIZE_PROPERTY_KEY => {
                self.quantize = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::IS_STATEFUL_PROPERTY_KEY => {
                self.is_stateful = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::DATA_BIND_PATH_IDS_PROPERTY_KEY => {
                let value = crate::mechanical_port::source::core::field_types::core_bytes_type::CoreBytesType::deserialize(reader);
                callbacks.decode_data_bind_path_ids(value.as_slice());
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for NestedArtboardBase {
    type Target = Drawable;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for NestedArtboardBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
