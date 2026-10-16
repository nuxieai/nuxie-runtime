use crate::mechanical_port::source::{
    animation::animation::Animation,
    animation::linear_animation::{LinearAnimation, LinearAnimationCells},
    core::binary_reader::BinaryReader,
};

pub trait LinearAnimationBaseCallbacks:
    crate::mechanical_port::source::generated::animation::animation_base::AnimationBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn duration_changed(&mut self) {}
}

pub struct LinearAnimationBase {
    pub base: Animation,
    // The timing properties' storage, shared with the animation's instances.
    cells: std::rc::Rc<LinearAnimationCells>,
}

impl Default for LinearAnimationBase {
    fn default() -> Self {
        Self {
            base: Animation::default(),
            cells: std::rc::Rc::new(LinearAnimationCells::default()),
        }
    }
}

impl LinearAnimationBase {
    pub const TYPE_KEY: u16 = 31;
    pub const FPS_PROPERTY_KEY: u16 = 56;
    pub const DURATION_PROPERTY_KEY: u16 = 57;
    pub const SPEED_PROPERTY_KEY: u16 = 58;
    pub const LOOP_VALUE_PROPERTY_KEY: u16 = 59;
    pub const WORK_START_PROPERTY_KEY: u16 = 60;
    pub const WORK_END_PROPERTY_KEY: u16 = 61;
    pub const ENABLE_WORK_AREA_PROPERTY_KEY: u16 = 62;
    pub const QUANTIZE_PROPERTY_KEY: u16 = 376;

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
    pub(crate) fn cells(&self) -> &std::rc::Rc<LinearAnimationCells> {
        &self.cells
    }
    pub fn fps(&self) -> u32 {
        self.cells.fps()
    }
    pub fn set_fps(&mut self, value: u32, callbacks: &mut impl LinearAnimationBaseCallbacks) {
        if !self.set_fps_value(value) {
            return;
        }
        LinearAnimationBaseCallbacks::notify_property_changed(callbacks, Self::FPS_PROPERTY_KEY);
    }

    pub(crate) fn set_fps_value(&mut self, value: u32) -> bool {
        if self.cells.fps() == value {
            return false;
        }
        self.cells.set_fps(value);
        true
    }
    pub fn duration(&self) -> u32 {
        self.cells.duration()
    }
    pub fn set_duration(&mut self, value: u32, callbacks: &mut impl LinearAnimationBaseCallbacks) {
        if !self.set_duration_value(value) {
            return;
        }
        callbacks.duration_changed();
        LinearAnimationBaseCallbacks::notify_property_changed(
            callbacks,
            Self::DURATION_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_duration_value(&mut self, value: u32) -> bool {
        if self.cells.duration() == value {
            return false;
        }
        self.cells.set_duration(value);
        true
    }
    pub fn speed(&self) -> f32 {
        self.cells.speed()
    }
    pub fn set_speed(&mut self, value: f32, callbacks: &mut impl LinearAnimationBaseCallbacks) {
        if !self.set_speed_value(value) {
            return;
        }
        LinearAnimationBaseCallbacks::notify_property_changed(callbacks, Self::SPEED_PROPERTY_KEY);
    }

    pub(crate) fn set_speed_value(&mut self, value: f32) -> bool {
        if self.cells.speed() == value {
            return false;
        }
        self.cells.set_speed(value);
        true
    }
    pub fn loop_value(&self) -> u32 {
        self.cells.loop_value()
    }
    pub fn set_loop_value(
        &mut self,
        value: u32,
        callbacks: &mut impl LinearAnimationBaseCallbacks,
    ) {
        if !self.set_loop_value_value(value) {
            return;
        }
        LinearAnimationBaseCallbacks::notify_property_changed(
            callbacks,
            Self::LOOP_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_loop_value_value(&mut self, value: u32) -> bool {
        if self.cells.loop_value() == value {
            return false;
        }
        self.cells.set_loop_value(value);
        true
    }
    pub fn work_start(&self) -> u32 {
        self.cells.work_start()
    }
    pub fn set_work_start(
        &mut self,
        value: u32,
        callbacks: &mut impl LinearAnimationBaseCallbacks,
    ) {
        if !self.set_work_start_value(value) {
            return;
        }
        LinearAnimationBaseCallbacks::notify_property_changed(
            callbacks,
            Self::WORK_START_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_work_start_value(&mut self, value: u32) -> bool {
        if self.cells.work_start() == value {
            return false;
        }
        self.cells.set_work_start(value);
        true
    }
    pub fn work_end(&self) -> u32 {
        self.cells.work_end()
    }
    pub fn set_work_end(&mut self, value: u32, callbacks: &mut impl LinearAnimationBaseCallbacks) {
        if !self.set_work_end_value(value) {
            return;
        }
        LinearAnimationBaseCallbacks::notify_property_changed(
            callbacks,
            Self::WORK_END_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_work_end_value(&mut self, value: u32) -> bool {
        if self.cells.work_end() == value {
            return false;
        }
        self.cells.set_work_end(value);
        true
    }
    pub fn enable_work_area(&self) -> bool {
        self.cells.enable_work_area()
    }
    pub fn set_enable_work_area(
        &mut self,
        value: bool,
        callbacks: &mut impl LinearAnimationBaseCallbacks,
    ) {
        if !self.set_enable_work_area_value(value) {
            return;
        }
        LinearAnimationBaseCallbacks::notify_property_changed(
            callbacks,
            Self::ENABLE_WORK_AREA_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_enable_work_area_value(&mut self, value: bool) -> bool {
        if self.cells.enable_work_area() == value {
            return false;
        }
        self.cells.set_enable_work_area(value);
        true
    }
    pub fn quantize(&self) -> bool {
        self.cells.quantize()
    }
    pub fn set_quantize(&mut self, value: bool, callbacks: &mut impl LinearAnimationBaseCallbacks) {
        if !self.set_quantize_value(value) {
            return;
        }
        LinearAnimationBaseCallbacks::notify_property_changed(
            callbacks,
            Self::QUANTIZE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_quantize_value(&mut self, value: bool) -> bool {
        if self.cells.quantize() == value {
            return false;
        }
        self.cells.set_quantize(value);
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl LinearAnimationBaseCallbacks) -> LinearAnimation {
        let mut cloned = LinearAnimation::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl LinearAnimationBaseCallbacks) {
        self.cells.copy_from(&object.cells);
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl LinearAnimationBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::FPS_PROPERTY_KEY => {
                self.cells.set_fps(crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader));
                true
            }
            Self::DURATION_PROPERTY_KEY => {
                self.cells.set_duration(crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader));
                true
            }
            Self::SPEED_PROPERTY_KEY => {
                self.cells.set_speed(crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader));
                true
            }
            Self::LOOP_VALUE_PROPERTY_KEY => {
                self.cells.set_loop_value(crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader));
                true
            }
            Self::WORK_START_PROPERTY_KEY => {
                self.cells.set_work_start(crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader));
                true
            }
            Self::WORK_END_PROPERTY_KEY => {
                self.cells.set_work_end(crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader));
                true
            }
            Self::ENABLE_WORK_AREA_PROPERTY_KEY => {
                self.cells.set_enable_work_area(crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader));
                true
            }
            Self::QUANTIZE_PROPERTY_KEY => {
                self.cells.set_quantize(crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader));
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for LinearAnimationBase {
    type Target = Animation;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for LinearAnimationBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
