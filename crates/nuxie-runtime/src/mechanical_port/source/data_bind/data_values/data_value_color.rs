use super::{data_type::DataType, data_value::DataValue};
use core::any::Any;
#[derive(Clone, Debug, Default)]
pub struct DataValueColor {
    value: i32,
}
impl DataValueColor {
    pub const TYPE_KEY: DataType = DataType::Color;
    pub const DEFAULT_VALUE: i32 = 0;
    pub fn new(value: i32) -> Self {
        Self { value }
    }
    pub fn value(&self) -> i32 {
        self.value
    }
    pub fn set_value(&mut self, value: i32) {
        self.value = value
    }
    pub fn alpha(&self) -> i32 {
        (self.value >> 24) & 0xff
    }
    pub fn red(&self) -> i32 {
        (self.value >> 16) & 0xff
    }
    pub fn green(&self) -> i32 {
        (self.value >> 8) & 0xff
    }
    pub fn blue(&self) -> i32 {
        self.value & 0xff
    }
    pub fn set_alpha(&mut self, value: i32) {
        self.value = ((self.value as u32 & 0x00ff_ffff) | ((value as u32) << 24)) as i32
    }
    pub fn set_red(&mut self, value: i32) {
        self.value = ((self.value as u32 & 0xff00_ffff) | ((value as u32) << 16)) as i32
    }
    pub fn set_green(&mut self, value: i32) {
        self.value = ((self.value as u32 & 0xffff_00ff) | ((value as u32) << 8)) as i32
    }
    pub fn set_blue(&mut self, value: i32) {
        self.value = ((self.value as u32 & 0xffff_ff00) | value as u32) as i32
    }
}
fn lerp_channel(a: u32, b: u32, mix: f32) -> u32 {
    (a as f32 * (1.0 - mix) + b as f32 * mix)
        .clamp(0.0, 255.0)
        .round() as u32
}
fn color_lerp(from: u32, to: u32, mix: f32) -> u32 {
    (lerp_channel(from >> 24, to >> 24, mix) << 24)
        | (lerp_channel((from >> 16) & 255, (to >> 16) & 255, mix) << 16)
        | (lerp_channel((from >> 8) & 255, (to >> 8) & 255, mix) << 8)
        | lerp_channel(from & 255, to & 255, mix)
}
impl DataValue for DataValueColor {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, data_type: DataType) -> bool {
        data_type == DataType::Color
    }
    fn compare(&self, comparand: Option<&dyn DataValue>) -> bool {
        comparand
            .and_then(|v| v.as_any().downcast_ref::<Self>())
            .is_some_and(|v| v.value == self.value)
    }
    fn interpolate(
        &self,
        to: Option<&dyn DataValue>,
        destination: Option<&mut dyn DataValue>,
        mix: f32,
    ) {
        if let (Some(to), Some(destination)) = (
            to.and_then(|v| v.as_any().downcast_ref::<Self>()),
            destination.and_then(|v| v.as_any_mut().downcast_mut::<Self>()),
        ) {
            destination.value = color_lerp(self.value as u32, to.value as u32, mix) as i32;
        }
    }
    fn copy_value(&self, destination: Option<&mut dyn DataValue>) {
        if let Some(destination) = destination.and_then(|v| v.as_any_mut().downcast_mut::<Self>()) {
            destination.value = self.value;
        }
    }
}
