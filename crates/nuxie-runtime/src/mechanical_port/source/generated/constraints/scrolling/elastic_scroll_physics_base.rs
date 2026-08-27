use crate::mechanical_port::source::{
    constraints::scrolling::elastic_scroll_physics::ElasticScrollPhysics,
    constraints::scrolling::scroll_physics::ScrollPhysics, core::binary_reader::BinaryReader,
};

pub trait ElasticScrollPhysicsBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn friction_changed(&mut self) {}
    fn speed_multiplier_changed(&mut self) {}
    fn elastic_factor_changed(&mut self) {}
}

pub struct ElasticScrollPhysicsBase {
    pub base: ScrollPhysics,
    friction: f32,
    speed_multiplier: f32,
    elastic_factor: f32,
}

impl Default for ElasticScrollPhysicsBase {
    fn default() -> Self {
        Self {
            base: ScrollPhysics::default(),
            friction: 8.0,
            speed_multiplier: 1.0,
            elastic_factor: 0.66,
        }
    }
}

impl ElasticScrollPhysicsBase {
    pub const TYPE_KEY: u16 = 525;
    pub const FRICTION_PROPERTY_KEY: u16 = 728;
    pub const SPEED_MULTIPLIER_PROPERTY_KEY: u16 = 729;
    pub const ELASTIC_FACTOR_PROPERTY_KEY: u16 = 730;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 523 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn friction(&self) -> f32 {
        self.friction
    }
    pub fn set_friction(
        &mut self,
        value: f32,
        callbacks: &mut impl ElasticScrollPhysicsBaseCallbacks,
    ) {
        if self.friction == value {
            return;
        }
        self.friction = value;
        callbacks.friction_changed();
        callbacks.notify_property_changed(Self::FRICTION_PROPERTY_KEY);
    }
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier
    }
    pub fn set_speed_multiplier(
        &mut self,
        value: f32,
        callbacks: &mut impl ElasticScrollPhysicsBaseCallbacks,
    ) {
        if self.speed_multiplier == value {
            return;
        }
        self.speed_multiplier = value;
        callbacks.speed_multiplier_changed();
        callbacks.notify_property_changed(Self::SPEED_MULTIPLIER_PROPERTY_KEY);
    }
    pub fn elastic_factor(&self) -> f32 {
        self.elastic_factor
    }
    pub fn set_elastic_factor(
        &mut self,
        value: f32,
        callbacks: &mut impl ElasticScrollPhysicsBaseCallbacks,
    ) {
        if self.elastic_factor == value {
            return;
        }
        self.elastic_factor = value;
        callbacks.elastic_factor_changed();
        callbacks.notify_property_changed(Self::ELASTIC_FACTOR_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl ElasticScrollPhysicsBaseCallbacks,
    ) -> ElasticScrollPhysics {
        let mut cloned = ElasticScrollPhysics::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ElasticScrollPhysicsBaseCallbacks) {
        self.friction = object.friction;
        self.speed_multiplier = object.speed_multiplier;
        self.elastic_factor = object.elastic_factor;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ElasticScrollPhysicsBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::FRICTION_PROPERTY_KEY => {
                self.friction = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::SPEED_MULTIPLIER_PROPERTY_KEY => {
                self.speed_multiplier = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::ELASTIC_FACTOR_PROPERTY_KEY => {
                self.elastic_factor = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
