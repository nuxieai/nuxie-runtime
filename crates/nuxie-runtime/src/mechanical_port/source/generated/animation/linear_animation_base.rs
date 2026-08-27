use crate::mechanical_port::source::{
    animation::animation::Animation, animation::linear_animation::LinearAnimation,
    core::binary_reader::BinaryReader,
};

pub trait LinearAnimationBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn fps_changed(&mut self) {}
    fn duration_changed(&mut self) {}
    fn speed_changed(&mut self) {}
    fn loop_value_changed(&mut self) {}
    fn work_start_changed(&mut self) {}
    fn work_end_changed(&mut self) {}
    fn enable_work_area_changed(&mut self) {}
    fn quantize_changed(&mut self) {}
}

pub struct LinearAnimationBase {
    pub base: Animation,
    fps: u32,
    duration: u32,
    speed: f32,
    loop_value: u32,
    work_start: u32,
    work_end: u32,
    enable_work_area: bool,
    quantize: bool,
}

impl Default for LinearAnimationBase {
    fn default() -> Self {
        Self {
            base: Animation::default(),
            fps: 60,
            duration: 60,
            speed: 1.0,
            loop_value: 0,
            work_start: u32::MAX,
            work_end: u32::MAX,
            enable_work_area: false,
            quantize: false,
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
        matches!(type_key, Self::TYPE_KEY | 27)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn fps(&self) -> u32 {
        self.fps
    }
    pub fn set_fps(&mut self, value: u32, callbacks: &mut impl LinearAnimationBaseCallbacks) {
        if self.fps == value {
            return;
        }
        self.fps = value;
        callbacks.fps_changed();
        callbacks.notify_property_changed(Self::FPS_PROPERTY_KEY);
    }
    pub fn duration(&self) -> u32 {
        self.duration
    }
    pub fn set_duration(&mut self, value: u32, callbacks: &mut impl LinearAnimationBaseCallbacks) {
        if self.duration == value {
            return;
        }
        self.duration = value;
        callbacks.duration_changed();
        callbacks.notify_property_changed(Self::DURATION_PROPERTY_KEY);
    }
    pub fn speed(&self) -> f32 {
        self.speed
    }
    pub fn set_speed(&mut self, value: f32, callbacks: &mut impl LinearAnimationBaseCallbacks) {
        if self.speed == value {
            return;
        }
        self.speed = value;
        callbacks.speed_changed();
        callbacks.notify_property_changed(Self::SPEED_PROPERTY_KEY);
    }
    pub fn loop_value(&self) -> u32 {
        self.loop_value
    }
    pub fn set_loop_value(
        &mut self,
        value: u32,
        callbacks: &mut impl LinearAnimationBaseCallbacks,
    ) {
        if self.loop_value == value {
            return;
        }
        self.loop_value = value;
        callbacks.loop_value_changed();
        callbacks.notify_property_changed(Self::LOOP_VALUE_PROPERTY_KEY);
    }
    pub fn work_start(&self) -> u32 {
        self.work_start
    }
    pub fn set_work_start(
        &mut self,
        value: u32,
        callbacks: &mut impl LinearAnimationBaseCallbacks,
    ) {
        if self.work_start == value {
            return;
        }
        self.work_start = value;
        callbacks.work_start_changed();
        callbacks.notify_property_changed(Self::WORK_START_PROPERTY_KEY);
    }
    pub fn work_end(&self) -> u32 {
        self.work_end
    }
    pub fn set_work_end(&mut self, value: u32, callbacks: &mut impl LinearAnimationBaseCallbacks) {
        if self.work_end == value {
            return;
        }
        self.work_end = value;
        callbacks.work_end_changed();
        callbacks.notify_property_changed(Self::WORK_END_PROPERTY_KEY);
    }
    pub fn enable_work_area(&self) -> bool {
        self.enable_work_area
    }
    pub fn set_enable_work_area(
        &mut self,
        value: bool,
        callbacks: &mut impl LinearAnimationBaseCallbacks,
    ) {
        if self.enable_work_area == value {
            return;
        }
        self.enable_work_area = value;
        callbacks.enable_work_area_changed();
        callbacks.notify_property_changed(Self::ENABLE_WORK_AREA_PROPERTY_KEY);
    }
    pub fn quantize(&self) -> bool {
        self.quantize
    }
    pub fn set_quantize(&mut self, value: bool, callbacks: &mut impl LinearAnimationBaseCallbacks) {
        if self.quantize == value {
            return;
        }
        self.quantize = value;
        callbacks.quantize_changed();
        callbacks.notify_property_changed(Self::QUANTIZE_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl LinearAnimationBaseCallbacks) -> LinearAnimation {
        let mut cloned = LinearAnimation::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl LinearAnimationBaseCallbacks) {
        self.fps = object.fps;
        self.duration = object.duration;
        self.speed = object.speed;
        self.loop_value = object.loop_value;
        self.work_start = object.work_start;
        self.work_end = object.work_end;
        self.enable_work_area = object.enable_work_area;
        self.quantize = object.quantize;
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
                self.fps = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::DURATION_PROPERTY_KEY => {
                self.duration = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::SPEED_PROPERTY_KEY => {
                self.speed = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::LOOP_VALUE_PROPERTY_KEY => {
                self.loop_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::WORK_START_PROPERTY_KEY => {
                self.work_start = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::WORK_END_PROPERTY_KEY => {
                self.work_end = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::ENABLE_WORK_AREA_PROPERTY_KEY => {
                self.enable_work_area = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::QUANTIZE_PROPERTY_KEY => {
                self.quantize = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
