use crate::mechanical_port::source::{
    container_component::ContainerComponent, core::binary_reader::BinaryReader,
    shapes::paint::linear_gradient::LinearGradient,
};

pub trait LinearGradientBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
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
    transform_x: f32,
    transform_y: f32,
    transform_rotation: f32,
    transform_scale_x: f32,
    transform_scale_y: f32,
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
            transform_x: 0.0,
            transform_y: 0.0,
            transform_rotation: 0.0,
            transform_scale_x: 1.0,
            transform_scale_y: 1.0,
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
    pub const TRANSFORM_X_PROPERTY_KEY: u16 = 1103;
    pub const TRANSFORM_Y_PROPERTY_KEY: u16 = 1104;
    pub const TRANSFORM_ROTATION_PROPERTY_KEY: u16 = 1105;
    pub const TRANSFORM_SCALE_X_PROPERTY_KEY: u16 = 1106;
    pub const TRANSFORM_SCALE_Y_PROPERTY_KEY: u16 = 1107;

    pub fn is_type_of(type_key: u16) -> bool {
        Self::TYPE_KEY == type_key
            || crate::mechanical_port::source::generated::core_type_tree::has_ancestor(
                Self::TYPE_KEY,
                type_key,
            )
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn start_x(&self) -> f32 {
        self.start_x
    }
    pub fn set_start_x(&mut self, value: f32, callbacks: &mut impl LinearGradientBaseCallbacks) {
        if !self.set_start_x_value(value) {
            return;
        }
        callbacks.start_x_changed();
        LinearGradientBaseCallbacks::notify_property_changed(callbacks, Self::START_X_PROPERTY_KEY);
    }

    pub(crate) fn set_start_x_value(&mut self, value: f32) -> bool {
        if self.start_x == value {
            return false;
        }
        self.start_x = value;
        true
    }
    pub fn start_y(&self) -> f32 {
        self.start_y
    }
    pub fn set_start_y(&mut self, value: f32, callbacks: &mut impl LinearGradientBaseCallbacks) {
        if !self.set_start_y_value(value) {
            return;
        }
        callbacks.start_y_changed();
        LinearGradientBaseCallbacks::notify_property_changed(callbacks, Self::START_Y_PROPERTY_KEY);
    }

    pub(crate) fn set_start_y_value(&mut self, value: f32) -> bool {
        if self.start_y == value {
            return false;
        }
        self.start_y = value;
        true
    }
    pub fn end_x(&self) -> f32 {
        self.end_x
    }
    pub fn set_end_x(&mut self, value: f32, callbacks: &mut impl LinearGradientBaseCallbacks) {
        if !self.set_end_x_value(value) {
            return;
        }
        callbacks.end_x_changed();
        LinearGradientBaseCallbacks::notify_property_changed(callbacks, Self::END_X_PROPERTY_KEY);
    }

    pub(crate) fn set_end_x_value(&mut self, value: f32) -> bool {
        if self.end_x == value {
            return false;
        }
        self.end_x = value;
        true
    }
    pub fn end_y(&self) -> f32 {
        self.end_y
    }
    pub fn set_end_y(&mut self, value: f32, callbacks: &mut impl LinearGradientBaseCallbacks) {
        if !self.set_end_y_value(value) {
            return;
        }
        callbacks.end_y_changed();
        LinearGradientBaseCallbacks::notify_property_changed(callbacks, Self::END_Y_PROPERTY_KEY);
    }

    pub(crate) fn set_end_y_value(&mut self, value: f32) -> bool {
        if self.end_y == value {
            return false;
        }
        self.end_y = value;
        true
    }
    pub fn opacity(&self) -> f32 {
        self.opacity
    }
    pub fn set_opacity(&mut self, value: f32, callbacks: &mut impl LinearGradientBaseCallbacks) {
        if !self.set_opacity_value(value) {
            return;
        }
        callbacks.opacity_changed();
        LinearGradientBaseCallbacks::notify_property_changed(callbacks, Self::OPACITY_PROPERTY_KEY);
    }

    pub(crate) fn set_opacity_value(&mut self, value: f32) -> bool {
        if self.opacity == value {
            return false;
        }
        self.opacity = value;
        true
    }
    pub fn transform_x(&self) -> f32 {
        self.transform_x
    }
    pub fn set_transform_x(
        &mut self,
        value: f32,
        callbacks: &mut impl LinearGradientBaseCallbacks,
    ) {
        if !self.set_transform_x_value(value) {
            return;
        }
        LinearGradientBaseCallbacks::notify_property_changed(
            callbacks,
            Self::TRANSFORM_X_PROPERTY_KEY,
        );
    }
    pub(crate) fn set_transform_x_value(&mut self, value: f32) -> bool {
        if self.transform_x == value {
            return false;
        }
        self.transform_x = value;
        true
    }
    pub fn transform_y(&self) -> f32 {
        self.transform_y
    }
    pub fn set_transform_y(
        &mut self,
        value: f32,
        callbacks: &mut impl LinearGradientBaseCallbacks,
    ) {
        if !self.set_transform_y_value(value) {
            return;
        }
        LinearGradientBaseCallbacks::notify_property_changed(
            callbacks,
            Self::TRANSFORM_Y_PROPERTY_KEY,
        );
    }
    pub(crate) fn set_transform_y_value(&mut self, value: f32) -> bool {
        if self.transform_y == value {
            return false;
        }
        self.transform_y = value;
        true
    }
    pub fn transform_rotation(&self) -> f32 {
        self.transform_rotation
    }
    pub fn set_transform_rotation(
        &mut self,
        value: f32,
        callbacks: &mut impl LinearGradientBaseCallbacks,
    ) {
        if !self.set_transform_rotation_value(value) {
            return;
        }
        LinearGradientBaseCallbacks::notify_property_changed(
            callbacks,
            Self::TRANSFORM_ROTATION_PROPERTY_KEY,
        );
    }
    pub(crate) fn set_transform_rotation_value(&mut self, value: f32) -> bool {
        if self.transform_rotation == value {
            return false;
        }
        self.transform_rotation = value;
        true
    }
    pub fn transform_scale_x(&self) -> f32 {
        self.transform_scale_x
    }
    pub fn set_transform_scale_x(
        &mut self,
        value: f32,
        callbacks: &mut impl LinearGradientBaseCallbacks,
    ) {
        if !self.set_transform_scale_x_value(value) {
            return;
        }
        LinearGradientBaseCallbacks::notify_property_changed(
            callbacks,
            Self::TRANSFORM_SCALE_X_PROPERTY_KEY,
        );
    }
    pub(crate) fn set_transform_scale_x_value(&mut self, value: f32) -> bool {
        if self.transform_scale_x == value {
            return false;
        }
        self.transform_scale_x = value;
        true
    }
    pub fn transform_scale_y(&self) -> f32 {
        self.transform_scale_y
    }
    pub fn set_transform_scale_y(
        &mut self,
        value: f32,
        callbacks: &mut impl LinearGradientBaseCallbacks,
    ) {
        if !self.set_transform_scale_y_value(value) {
            return;
        }
        LinearGradientBaseCallbacks::notify_property_changed(
            callbacks,
            Self::TRANSFORM_SCALE_Y_PROPERTY_KEY,
        );
    }
    pub(crate) fn set_transform_scale_y_value(&mut self, value: f32) -> bool {
        if self.transform_scale_y == value {
            return false;
        }
        self.transform_scale_y = value;
        true
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
        self.transform_x = object.transform_x;
        self.transform_y = object.transform_y;
        self.transform_rotation = object.transform_rotation;
        self.transform_scale_x = object.transform_scale_x;
        self.transform_scale_y = object.transform_scale_y;
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
            Self::TRANSFORM_X_PROPERTY_KEY => {
                self.transform_x = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::TRANSFORM_Y_PROPERTY_KEY => {
                self.transform_y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::TRANSFORM_ROTATION_PROPERTY_KEY => {
                self.transform_rotation = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::TRANSFORM_SCALE_X_PROPERTY_KEY => {
                self.transform_scale_x = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::TRANSFORM_SCALE_Y_PROPERTY_KEY => {
                self.transform_scale_y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for LinearGradientBase {
    type Target = ContainerComponent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for LinearGradientBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

// Supplemental coverage for the public 16332c8e property delta; upstream added no tests.
#[cfg(test)]
mod angular_transform_property_tests {
    use super::*;
    use crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks;

    #[derive(Default)]
    struct Notifications(Vec<u16>);
    impl ComponentBaseCallbacks for Notifications {
        fn notify_property_changed(&mut self, key: u16) {
            self.0.push(key);
        }
    }
    impl LinearGradientBaseCallbacks for Notifications {
        fn notify_property_changed(&mut self, key: u16) {
            self.0.push(key);
        }
        fn start_x_changed(&mut self) {
            panic!("transform must not change start_x");
        }
        fn start_y_changed(&mut self) {
            panic!("transform must not change start_y");
        }
        fn end_x_changed(&mut self) {
            panic!("transform must not change end_x");
        }
        fn end_y_changed(&mut self) {
            panic!("transform must not change end_y");
        }
        fn opacity_changed(&mut self) {
            panic!("transform must not change opacity");
        }
    }

    fn values(base: &LinearGradientBase) -> [f32; 5] {
        [
            base.transform_x(),
            base.transform_y(),
            base.transform_rotation(),
            base.transform_scale_x(),
            base.transform_scale_y(),
        ]
    }

    #[test]
    fn angular_transform_defaults_setters_and_copy_match_public_base() {
        let mut base = LinearGradientBase::default();
        let mut notifications = Notifications::default();
        assert_eq!(values(&base), [0.0, 0.0, 0.0, 1.0, 1.0]);
        base.set_transform_x(2.0, &mut notifications);
        base.set_transform_x(2.0, &mut notifications);
        base.set_transform_y(3.0, &mut notifications);
        base.set_transform_y(3.0, &mut notifications);
        base.set_transform_rotation(4.0, &mut notifications);
        base.set_transform_rotation(4.0, &mut notifications);
        base.set_transform_scale_x(5.0, &mut notifications);
        base.set_transform_scale_x(5.0, &mut notifications);
        base.set_transform_scale_y(6.0, &mut notifications);
        base.set_transform_scale_y(6.0, &mut notifications);
        assert_eq!(notifications.0, [1103, 1104, 1105, 1106, 1107]);
        let copied = base.clone_into(&mut notifications);
        assert_eq!(values(&copied.base), [2.0, 3.0, 4.0, 5.0, 6.0]);
        assert_eq!(notifications.0, [1103, 1104, 1105, 1106, 1107]);

        base.set_transform_x(0.0, &mut notifications);
        let count = notifications.0.len();
        base.set_transform_x(-0.0, &mut notifications);
        assert_eq!(base.transform_x().to_bits(), 0.0f32.to_bits());
        assert_eq!(notifications.0.len(), count);
        base.set_transform_x(f32::NAN, &mut notifications);
        base.set_transform_x(f32::NAN, &mut notifications);
        assert!(base.transform_x().is_nan());
        assert_eq!(notifications.0.len(), count + 2);
    }

    #[test]
    fn angular_transform_deserialization_is_direct_without_notifications() {
        let mut base = LinearGradientBase::default();
        let mut notifications = Notifications::default();
        for (key, value) in [
            (1103, 7.0f32),
            (1104, 8.0),
            (1105, 9.0),
            (1106, 10.0),
            (1107, 11.0),
        ] {
            let bytes = value.to_le_bytes();
            let mut reader = BinaryReader::new(&bytes);
            assert!(base.deserialize(key, &mut reader, &mut notifications));
            assert!(reader.reached_end());
            assert!(!reader.has_error());
        }
        assert_eq!(values(&base), [7.0, 8.0, 9.0, 10.0, 11.0]);
        assert!(notifications.0.is_empty());
    }
}
