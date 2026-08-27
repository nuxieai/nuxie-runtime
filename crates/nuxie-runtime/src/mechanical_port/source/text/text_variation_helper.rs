use super::text_style::TextStyle;
use crate::mechanical_port::source::{component::Component, component_dirt::ComponentDirt};
use std::ptr::NonNull;
pub struct TextVariationHelper {
    pub component: Component,
    text_style: NonNull<TextStyle>,
}
impl TextVariationHelper {
    pub fn new(text_style: NonNull<TextStyle>) -> Self {
        Self {
            component: Component::default(),
            text_style,
        }
    }
    pub fn style(&self) -> NonNull<TextStyle> {
        self.text_style
    }
    pub fn build_dependencies(&mut self) {
        let text = unsafe { self.text_style.as_ref() }
            .base
            .parent()
            .expect("TextStyle parent");
        text.artboard()
            .expect("Text parent artboard")
            .add_dependent(self);
        self.component.add_dependent(text);
    }
    pub fn update(&mut self, _value: ComponentDirt) {
        unsafe { self.text_style.as_mut() }.update_variable_font();
    }
}
