use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, shapes::parametric_path::ParametricPath,
    shapes::rectangle::Rectangle,
};

pub trait RectangleBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn link_corner_radius_changed(&mut self) {}
    fn corner_radius_tl_changed(&mut self) {}
    fn corner_radius_tr_changed(&mut self) {}
    fn corner_radius_bl_changed(&mut self) {}
    fn corner_radius_br_changed(&mut self) {}
}

pub struct RectangleBase {
    pub base: ParametricPath,
    link_corner_radius: bool,
    corner_radius_tl: f32,
    corner_radius_tr: f32,
    corner_radius_bl: f32,
    corner_radius_br: f32,
}

impl Default for RectangleBase {
    fn default() -> Self {
        Self {
            base: ParametricPath::default(),
            link_corner_radius: true,
            corner_radius_tl: 0.0,
            corner_radius_tr: 0.0,
            corner_radius_bl: 0.0,
            corner_radius_br: 0.0,
        }
    }
}

impl RectangleBase {
    pub const TYPE_KEY: u16 = 7;
    pub const LINK_CORNER_RADIUS_PROPERTY_KEY: u16 = 164;
    pub const CORNER_RADIUS_TL_PROPERTY_KEY: u16 = 31;
    pub const CORNER_RADIUS_TR_PROPERTY_KEY: u16 = 161;
    pub const CORNER_RADIUS_BL_PROPERTY_KEY: u16 = 162;
    pub const CORNER_RADIUS_BR_PROPERTY_KEY: u16 = 163;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 15 | 12 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn link_corner_radius(&self) -> bool {
        self.link_corner_radius
    }
    pub fn set_link_corner_radius(
        &mut self,
        value: bool,
        callbacks: &mut impl RectangleBaseCallbacks,
    ) {
        if self.link_corner_radius == value {
            return;
        }
        self.link_corner_radius = value;
        callbacks.link_corner_radius_changed();
        callbacks.notify_property_changed(Self::LINK_CORNER_RADIUS_PROPERTY_KEY);
    }
    pub fn corner_radius_tl(&self) -> f32 {
        self.corner_radius_tl
    }
    pub fn set_corner_radius_tl(
        &mut self,
        value: f32,
        callbacks: &mut impl RectangleBaseCallbacks,
    ) {
        if self.corner_radius_tl == value {
            return;
        }
        self.corner_radius_tl = value;
        callbacks.corner_radius_tl_changed();
        callbacks.notify_property_changed(Self::CORNER_RADIUS_TL_PROPERTY_KEY);
    }
    pub fn corner_radius_tr(&self) -> f32 {
        self.corner_radius_tr
    }
    pub fn set_corner_radius_tr(
        &mut self,
        value: f32,
        callbacks: &mut impl RectangleBaseCallbacks,
    ) {
        if self.corner_radius_tr == value {
            return;
        }
        self.corner_radius_tr = value;
        callbacks.corner_radius_tr_changed();
        callbacks.notify_property_changed(Self::CORNER_RADIUS_TR_PROPERTY_KEY);
    }
    pub fn corner_radius_bl(&self) -> f32 {
        self.corner_radius_bl
    }
    pub fn set_corner_radius_bl(
        &mut self,
        value: f32,
        callbacks: &mut impl RectangleBaseCallbacks,
    ) {
        if self.corner_radius_bl == value {
            return;
        }
        self.corner_radius_bl = value;
        callbacks.corner_radius_bl_changed();
        callbacks.notify_property_changed(Self::CORNER_RADIUS_BL_PROPERTY_KEY);
    }
    pub fn corner_radius_br(&self) -> f32 {
        self.corner_radius_br
    }
    pub fn set_corner_radius_br(
        &mut self,
        value: f32,
        callbacks: &mut impl RectangleBaseCallbacks,
    ) {
        if self.corner_radius_br == value {
            return;
        }
        self.corner_radius_br = value;
        callbacks.corner_radius_br_changed();
        callbacks.notify_property_changed(Self::CORNER_RADIUS_BR_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl RectangleBaseCallbacks) -> Rectangle {
        let mut cloned = Rectangle::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl RectangleBaseCallbacks) {
        self.link_corner_radius = object.link_corner_radius;
        self.corner_radius_tl = object.corner_radius_tl;
        self.corner_radius_tr = object.corner_radius_tr;
        self.corner_radius_bl = object.corner_radius_bl;
        self.corner_radius_br = object.corner_radius_br;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl RectangleBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::LINK_CORNER_RADIUS_PROPERTY_KEY => {
                self.link_corner_radius = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::CORNER_RADIUS_TL_PROPERTY_KEY => {
                self.corner_radius_tl = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::CORNER_RADIUS_TR_PROPERTY_KEY => {
                self.corner_radius_tr = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::CORNER_RADIUS_BL_PROPERTY_KEY => {
                self.corner_radius_bl = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::CORNER_RADIUS_BR_PROPERTY_KEY => {
                self.corner_radius_br = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
