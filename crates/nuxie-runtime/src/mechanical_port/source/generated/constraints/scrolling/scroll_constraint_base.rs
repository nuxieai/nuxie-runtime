use crate::mechanical_port::source::{
    constraints::draggable_constraint::DraggableConstraint,
    constraints::scrolling::scroll_constraint::ScrollConstraint, core::binary_reader::BinaryReader,
};

pub trait ScrollConstraintBaseCallbacks: crate::mechanical_port::source::generated::constraints::draggable_constraint_base::DraggableConstraintBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn scroll_offset_x_changed(&mut self) {}
    fn scroll_offset_y_changed(&mut self) {}
    fn scroll_percent_x_changed(&mut self) {}
    fn scroll_percent_y_changed(&mut self) {}
    fn scroll_index_changed(&mut self) {}
    fn snap_changed(&mut self) {}
    fn physics_type_value_changed(&mut self) {}
    fn physics_id_changed(&mut self) {}
    fn virtualize_changed(&mut self) {}
    fn infinite_changed(&mut self) {}
    fn interactive_changed(&mut self) {}
    fn threshold_changed(&mut self) {}
    fn velocity_x_changed(&mut self) {}
    fn velocity_y_changed(&mut self) {}
    fn scroll_active_changed(&mut self) {}
    fn drag_multiplier_changed(&mut self) {}
    fn computed_content_width_changed(&mut self) {}
    fn computed_content_height_changed(&mut self) {}
    fn set_scroll_percent_x(&mut self, value: f32);
    fn scroll_percent_x(&mut self) -> f32;
    fn set_scroll_percent_y(&mut self, value: f32);
    fn scroll_percent_y(&mut self) -> f32;
    fn set_scroll_index(&mut self, value: f32);
    fn scroll_index(&mut self) -> f32;
    fn set_velocity_x(&mut self, value: f32);
    fn velocity_x(&mut self) -> f32;
    fn set_velocity_y(&mut self, value: f32);
    fn velocity_y(&mut self) -> f32;
    fn set_scroll_active(&mut self, value: bool);
    fn scroll_active(&mut self) -> bool;
    fn set_computed_content_width(&mut self, value: f32);
    fn computed_content_width(&mut self) -> f32;
    fn set_computed_content_height(&mut self, value: f32);
    fn computed_content_height(&mut self) -> f32;
}

pub struct ScrollConstraintBase {
    pub base: DraggableConstraint,
    scroll_offset_x: f32,
    scroll_offset_y: f32,
    snap: bool,
    physics_type_value: u32,
    physics_id: u32,
    virtualize: bool,
    infinite: bool,
    interactive: bool,
    threshold: f32,
    drag_multiplier: f32,
}

impl Default for ScrollConstraintBase {
    fn default() -> Self {
        Self {
            base: DraggableConstraint::default(),
            scroll_offset_x: 0.0,
            scroll_offset_y: 0.0,
            snap: false,
            physics_type_value: 0,
            physics_id: u32::MAX,
            virtualize: false,
            infinite: false,
            interactive: true,
            threshold: 0.0,
            drag_multiplier: 1.0,
        }
    }
}

impl ScrollConstraintBase {
    pub const TYPE_KEY: u16 = 521;
    pub const SCROLL_OFFSET_X_PROPERTY_KEY: u16 = 759;
    pub const SCROLL_OFFSET_Y_PROPERTY_KEY: u16 = 760;
    pub const SCROLL_PERCENT_X_PROPERTY_KEY: u16 = 761;
    pub const SCROLL_PERCENT_Y_PROPERTY_KEY: u16 = 762;
    pub const SCROLL_INDEX_PROPERTY_KEY: u16 = 763;
    pub const SNAP_PROPERTY_KEY: u16 = 724;
    pub const PHYSICS_TYPE_VALUE_PROPERTY_KEY: u16 = 727;
    pub const PHYSICS_ID_PROPERTY_KEY: u16 = 726;
    pub const VIRTUALIZE_PROPERTY_KEY: u16 = 850;
    pub const INFINITE_PROPERTY_KEY: u16 = 851;
    pub const INTERACTIVE_PROPERTY_KEY: u16 = 891;
    pub const THRESHOLD_PROPERTY_KEY: u16 = 894;
    pub const VELOCITY_X_PROPERTY_KEY: u16 = 1023;
    pub const VELOCITY_Y_PROPERTY_KEY: u16 = 1024;
    pub const SCROLL_ACTIVE_PROPERTY_KEY: u16 = 1025;
    pub const DRAG_MULTIPLIER_PROPERTY_KEY: u16 = 1029;
    pub const COMPUTED_CONTENT_WIDTH_PROPERTY_KEY: u16 = 1069;
    pub const COMPUTED_CONTENT_HEIGHT_PROPERTY_KEY: u16 = 1070;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 520 | 79 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn scroll_offset_x(&self) -> f32 {
        self.scroll_offset_x
    }
    pub fn set_scroll_offset_x(
        &mut self,
        value: f32,
        callbacks: &mut impl ScrollConstraintBaseCallbacks,
    ) {
        if !self.set_scroll_offset_x_value(value) {
            return;
        }
        callbacks.scroll_offset_x_changed();
        ScrollConstraintBaseCallbacks::notify_property_changed(
            callbacks,
            Self::SCROLL_OFFSET_X_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_scroll_offset_x_value(&mut self, value: f32) -> bool {
        if self.scroll_offset_x == value {
            return false;
        }
        self.scroll_offset_x = value;
        true
    }
    pub fn scroll_offset_y(&self) -> f32 {
        self.scroll_offset_y
    }
    pub fn set_scroll_offset_y(
        &mut self,
        value: f32,
        callbacks: &mut impl ScrollConstraintBaseCallbacks,
    ) {
        if !self.set_scroll_offset_y_value(value) {
            return;
        }
        callbacks.scroll_offset_y_changed();
        ScrollConstraintBaseCallbacks::notify_property_changed(
            callbacks,
            Self::SCROLL_OFFSET_Y_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_scroll_offset_y_value(&mut self, value: f32) -> bool {
        if self.scroll_offset_y == value {
            return false;
        }
        self.scroll_offset_y = value;
        true
    }
    pub fn snap(&self) -> bool {
        self.snap
    }
    pub fn set_snap(&mut self, value: bool, callbacks: &mut impl ScrollConstraintBaseCallbacks) {
        if !self.set_snap_value(value) {
            return;
        }
        callbacks.snap_changed();
        ScrollConstraintBaseCallbacks::notify_property_changed(callbacks, Self::SNAP_PROPERTY_KEY);
    }

    pub(crate) fn set_snap_value(&mut self, value: bool) -> bool {
        if self.snap == value {
            return false;
        }
        self.snap = value;
        true
    }
    pub fn physics_type_value(&self) -> u32 {
        self.physics_type_value
    }
    pub fn set_physics_type_value(
        &mut self,
        value: u32,
        callbacks: &mut impl ScrollConstraintBaseCallbacks,
    ) {
        if !self.set_physics_type_value_value(value) {
            return;
        }
        callbacks.physics_type_value_changed();
        ScrollConstraintBaseCallbacks::notify_property_changed(
            callbacks,
            Self::PHYSICS_TYPE_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_physics_type_value_value(&mut self, value: u32) -> bool {
        if self.physics_type_value == value {
            return false;
        }
        self.physics_type_value = value;
        true
    }
    pub fn physics_id(&self) -> u32 {
        self.physics_id
    }
    pub fn set_physics_id(
        &mut self,
        value: u32,
        callbacks: &mut impl ScrollConstraintBaseCallbacks,
    ) {
        if !self.set_physics_id_value(value) {
            return;
        }
        callbacks.physics_id_changed();
        ScrollConstraintBaseCallbacks::notify_property_changed(
            callbacks,
            Self::PHYSICS_ID_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_physics_id_value(&mut self, value: u32) -> bool {
        if self.physics_id == value {
            return false;
        }
        self.physics_id = value;
        true
    }
    pub fn virtualize(&self) -> bool {
        self.virtualize
    }
    pub fn set_virtualize(
        &mut self,
        value: bool,
        callbacks: &mut impl ScrollConstraintBaseCallbacks,
    ) {
        if !self.set_virtualize_value(value) {
            return;
        }
        callbacks.virtualize_changed();
        ScrollConstraintBaseCallbacks::notify_property_changed(
            callbacks,
            Self::VIRTUALIZE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_virtualize_value(&mut self, value: bool) -> bool {
        if self.virtualize == value {
            return false;
        }
        self.virtualize = value;
        true
    }
    pub fn infinite(&self) -> bool {
        self.infinite
    }
    pub fn set_infinite(
        &mut self,
        value: bool,
        callbacks: &mut impl ScrollConstraintBaseCallbacks,
    ) {
        if !self.set_infinite_value(value) {
            return;
        }
        callbacks.infinite_changed();
        ScrollConstraintBaseCallbacks::notify_property_changed(
            callbacks,
            Self::INFINITE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_infinite_value(&mut self, value: bool) -> bool {
        if self.infinite == value {
            return false;
        }
        self.infinite = value;
        true
    }
    pub fn interactive(&self) -> bool {
        self.interactive
    }
    pub fn set_interactive(
        &mut self,
        value: bool,
        callbacks: &mut impl ScrollConstraintBaseCallbacks,
    ) {
        if !self.set_interactive_value(value) {
            return;
        }
        callbacks.interactive_changed();
        ScrollConstraintBaseCallbacks::notify_property_changed(
            callbacks,
            Self::INTERACTIVE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_interactive_value(&mut self, value: bool) -> bool {
        if self.interactive == value {
            return false;
        }
        self.interactive = value;
        true
    }
    pub fn threshold(&self) -> f32 {
        self.threshold
    }
    pub fn set_threshold(
        &mut self,
        value: f32,
        callbacks: &mut impl ScrollConstraintBaseCallbacks,
    ) {
        if !self.set_threshold_value(value) {
            return;
        }
        callbacks.threshold_changed();
        ScrollConstraintBaseCallbacks::notify_property_changed(
            callbacks,
            Self::THRESHOLD_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_threshold_value(&mut self, value: f32) -> bool {
        if self.threshold == value {
            return false;
        }
        self.threshold = value;
        true
    }
    pub fn drag_multiplier(&self) -> f32 {
        self.drag_multiplier
    }
    pub fn set_drag_multiplier(
        &mut self,
        value: f32,
        callbacks: &mut impl ScrollConstraintBaseCallbacks,
    ) {
        if !self.set_drag_multiplier_value(value) {
            return;
        }
        callbacks.drag_multiplier_changed();
        ScrollConstraintBaseCallbacks::notify_property_changed(
            callbacks,
            Self::DRAG_MULTIPLIER_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_drag_multiplier_value(&mut self, value: f32) -> bool {
        if self.drag_multiplier == value {
            return false;
        }
        self.drag_multiplier = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl ScrollConstraintBaseCallbacks,
    ) -> ScrollConstraint {
        let mut cloned = ScrollConstraint::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ScrollConstraintBaseCallbacks) {
        self.scroll_offset_x = object.scroll_offset_x;
        self.scroll_offset_y = object.scroll_offset_y;
        self.snap = object.snap;
        self.physics_type_value = object.physics_type_value;
        self.physics_id = object.physics_id;
        self.virtualize = object.virtualize;
        self.infinite = object.infinite;
        self.interactive = object.interactive;
        self.threshold = object.threshold;
        self.drag_multiplier = object.drag_multiplier;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ScrollConstraintBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::SCROLL_OFFSET_X_PROPERTY_KEY => {
                self.scroll_offset_x = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::SCROLL_OFFSET_Y_PROPERTY_KEY => {
                self.scroll_offset_y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::SNAP_PROPERTY_KEY => {
                self.snap = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::PHYSICS_TYPE_VALUE_PROPERTY_KEY => {
                self.physics_type_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::PHYSICS_ID_PROPERTY_KEY => {
                self.physics_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::VIRTUALIZE_PROPERTY_KEY => {
                self.virtualize = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::INFINITE_PROPERTY_KEY => {
                self.infinite = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::INTERACTIVE_PROPERTY_KEY => {
                self.interactive = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::THRESHOLD_PROPERTY_KEY => {
                self.threshold = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::DRAG_MULTIPLIER_PROPERTY_KEY => {
                self.drag_multiplier = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for ScrollConstraintBase {
    type Target = DraggableConstraint;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ScrollConstraintBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
