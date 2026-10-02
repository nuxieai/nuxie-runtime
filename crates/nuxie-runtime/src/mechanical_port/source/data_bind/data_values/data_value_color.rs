use super::{data_type::DataType, data_value::DataValue};
use crate::mechanical_port::source::shapes::paint::color::color_lerp;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolation_delegates_to_the_shared_color_owner_in_both_modes() {
        let from = DataValueColor::new(0xff65_677a_u32 as i32);
        let to = DataValueColor::new(0xff68_fa2f_u32 as i32);
        for mix in [0.0, f32::from_bits(0x3e99_9998), 1.0, f32::NAN] {
            let mut destination = DataValueColor::default();
            from.interpolate(Some(&to), Some(&mut destination), mix);
            assert_eq!(
                destination.value() as u32,
                color_lerp(from.value() as u32, to.value() as u32, mix)
            );
            if mix.is_nan() {
                assert_eq!(destination.value(), -1);
            }
        }
    }
}
