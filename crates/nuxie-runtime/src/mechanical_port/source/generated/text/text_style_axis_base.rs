use crate::mechanical_port::source::{
    component::Component, core::binary_reader::BinaryReader, text::text_style_axis::TextStyleAxis,
};

pub trait TextStyleAxisBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn tag_changed(&mut self) {}
    fn axis_value_changed(&mut self) {}
}

pub struct TextStyleAxisBase {
    pub base: Component,
    tag: u32,
    axis_value: f32,
    // Runtime freshness only, bound for CSS input styles.
    css_input_options_generation: Option<std::rc::Rc<std::cell::Cell<u64>>>,
}

impl Default for TextStyleAxisBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            tag: 0,
            axis_value: 0.0,
            css_input_options_generation: None,
        }
    }
}

impl TextStyleAxisBase {
    pub(crate) fn bind_css_input_options_generation(
        &mut self,
        generation: std::rc::Rc<std::cell::Cell<u64>>,
    ) {
        if self
            .css_input_options_generation
            .as_ref()
            .is_some_and(|current| std::rc::Rc::ptr_eq(current, &generation))
        {
            return;
        }
        self.bump_css_input_options_generation();
        generation.set(generation.get().wrapping_add(1));
        self.css_input_options_generation = Some(generation);
    }
    pub(crate) fn detach_css_input_options_generation(&mut self) {
        self.bump_css_input_options_generation();
        self.css_input_options_generation = None;
    }
    fn bump_css_input_options_generation(&self) {
        if let Some(generation) = &self.css_input_options_generation {
            generation.set(generation.get().wrapping_add(1));
        }
    }

    pub const TYPE_KEY: u16 = 144;
    pub const TAG_PROPERTY_KEY: u16 = 289;
    pub const AXIS_VALUE_PROPERTY_KEY: u16 = 288;

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
    pub fn tag(&self) -> u32 {
        self.tag
    }
    pub fn set_tag(&mut self, value: u32, callbacks: &mut impl TextStyleAxisBaseCallbacks) {
        if !self.set_tag_value(value) {
            return;
        }
        callbacks.tag_changed();
        TextStyleAxisBaseCallbacks::notify_property_changed(callbacks, Self::TAG_PROPERTY_KEY);
    }

    pub(crate) fn set_tag_value(&mut self, value: u32) -> bool {
        if self.tag == value {
            return false;
        }
        self.tag = value;
        self.bump_css_input_options_generation();
        true
    }
    pub fn axis_value(&self) -> f32 {
        self.axis_value
    }
    pub fn set_axis_value(&mut self, value: f32, callbacks: &mut impl TextStyleAxisBaseCallbacks) {
        if !self.set_axis_value_value(value) {
            return;
        }
        callbacks.axis_value_changed();
        TextStyleAxisBaseCallbacks::notify_property_changed(
            callbacks,
            Self::AXIS_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_axis_value_value(&mut self, value: f32) -> bool {
        if self.axis_value == value {
            return false;
        }
        self.axis_value = value;
        self.bump_css_input_options_generation();
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl TextStyleAxisBaseCallbacks) -> TextStyleAxis {
        let mut cloned = TextStyleAxis::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl TextStyleAxisBaseCallbacks) {
        self.tag = object.tag;
        self.axis_value = object.axis_value;
        self.bump_css_input_options_generation();
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TextStyleAxisBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::TAG_PROPERTY_KEY => {
                self.set_tag_value(crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader));
                true
            }
            Self::AXIS_VALUE_PROPERTY_KEY => {
                self.set_axis_value_value(crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader));
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for TextStyleAxisBase {
    type Target = Component;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextStyleAxisBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Drop for TextStyleAxisBase {
    fn drop(&mut self) {
        self.bump_css_input_options_generation();
    }
}
