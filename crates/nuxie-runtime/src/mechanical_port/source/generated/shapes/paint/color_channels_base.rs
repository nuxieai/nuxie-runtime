use crate::mechanical_port::source::{
    generated::core_registry::CoreRegistryObject,
    shapes::paint::{gradient_stop::GradientStop, solid_color::SolidColor},
};

// This mixin has no Core type or stored state. Its channels are views of the
// concrete owner's colorValue, and writes use that owner's normal setter.
pub struct ColorChannelsBase;

impl ColorChannelsBase {
    pub const COLOR_RED_PROPERTY_KEY: u16 = 118;
    pub const COLOR_RED_BIT_OFFSET: u32 = 16;
    pub const COLOR_RED_FIELD_MASK: u32 = 16_711_680;
    pub const COLOR_GREEN_PROPERTY_KEY: u16 = 136;
    pub const COLOR_GREEN_BIT_OFFSET: u32 = 8;
    pub const COLOR_GREEN_FIELD_MASK: u32 = 65_280;
    pub const COLOR_BLUE_PROPERTY_KEY: u16 = 210;
    pub const COLOR_BLUE_BIT_OFFSET: u32 = 0;
    pub const COLOR_BLUE_FIELD_MASK: u32 = 255;
    pub const COLOR_ALPHA_PROPERTY_KEY: u16 = 218;
    pub const COLOR_ALPHA_BIT_OFFSET: u32 = 24;
    pub const COLOR_ALPHA_FIELD_MASK: u32 = 4_278_190_080;

    pub fn from<O: CoreRegistryObject + ?Sized>(object: &O) -> Option<&dyn ColorChannels> {
        let object = object.as_registry_any();
        if let Some(color) = object.downcast_ref::<SolidColor>() {
            return Some(color);
        }
        object
            .downcast_ref::<GradientStop>()
            .map(|stop| stop as &dyn ColorChannels)
    }

    pub fn from_mut<O: CoreRegistryObject + ?Sized>(
        object: &mut O,
    ) -> Option<&mut dyn ColorChannels> {
        let object = object.as_registry_any_mut();
        if object.is::<SolidColor>() {
            return object
                .downcast_mut::<SolidColor>()
                .map(|color| color as &mut dyn ColorChannels);
        }
        object
            .downcast_mut::<GradientStop>()
            .map(|stop| stop as &mut dyn ColorChannels)
    }
}

pub trait ColorChannels {
    fn color_value(&self) -> i32;
    fn set_color_value(&mut self, value: i32);

    fn color_red(&self) -> u32 {
        ((self.color_value() as u32) >> ColorChannelsBase::COLOR_RED_BIT_OFFSET) & 255
    }

    fn set_color_red(&mut self, value: u32) {
        let value = value.min(255);
        let current = self.color_value();
        let mask = ColorChannelsBase::COLOR_RED_FIELD_MASK as i32;
        let next = (current & !mask)
            | (((value << ColorChannelsBase::COLOR_RED_BIT_OFFSET) as i32) & mask);
        if current != next {
            self.set_color_value(next);
        }
    }

    fn color_green(&self) -> u32 {
        ((self.color_value() as u32) >> ColorChannelsBase::COLOR_GREEN_BIT_OFFSET) & 255
    }

    fn set_color_green(&mut self, value: u32) {
        let value = value.min(255);
        let current = self.color_value();
        let mask = ColorChannelsBase::COLOR_GREEN_FIELD_MASK as i32;
        let next = (current & !mask)
            | (((value << ColorChannelsBase::COLOR_GREEN_BIT_OFFSET) as i32) & mask);
        if current != next {
            self.set_color_value(next);
        }
    }

    fn color_blue(&self) -> u32 {
        ((self.color_value() as u32) >> ColorChannelsBase::COLOR_BLUE_BIT_OFFSET) & 255
    }

    fn set_color_blue(&mut self, value: u32) {
        let value = value.min(255);
        let current = self.color_value();
        let mask = ColorChannelsBase::COLOR_BLUE_FIELD_MASK as i32;
        let next = (current & !mask)
            | (((value << ColorChannelsBase::COLOR_BLUE_BIT_OFFSET) as i32) & mask);
        if current != next {
            self.set_color_value(next);
        }
    }

    fn color_alpha(&self) -> u32 {
        ((self.color_value() as u32) >> ColorChannelsBase::COLOR_ALPHA_BIT_OFFSET) & 255
    }

    fn set_color_alpha(&mut self, value: u32) {
        let value = value.min(255);
        let current = self.color_value();
        let mask = ColorChannelsBase::COLOR_ALPHA_FIELD_MASK as i32;
        let next = (current & !mask)
            | (((value << ColorChannelsBase::COLOR_ALPHA_BIT_OFFSET) as i32) & mask);
        if current != next {
            self.set_color_value(next);
        }
    }
}
