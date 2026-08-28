use crate::mechanical_port::source::{
    component::Component, component_dirt::ComponentDirt, core::CoreHandle,
};
pub struct TextVariationHelper {
    pub component: Component,
    text_style: CoreHandle,
}
impl TextVariationHelper {
    pub fn new(text_style: CoreHandle) -> Self {
        Self {
            component: Component::default(),
            text_style,
        }
    }
    pub fn style(&self) -> CoreHandle {
        self.text_style.clone()
    }
    pub fn build_dependencies(&mut self) {
        let text = self
            .text_style
            .with(|style| style.component_parent_handle())
            .flatten()
            .expect("TextStyle parent");
        if let Some(dependent) = self.component.handle() {
            text.with(|text| text.component_artboard_handle())
                .flatten()
                .and_then(|artboard| {
                    artboard
                        .with_mut(|artboard| artboard.component_add_dependent(dependent.clone()))
                });
            self.component.add_dependent(text);
        }
    }
    pub fn update(&mut self, _value: ComponentDirt) {
        self.text_style.with_mut(|style| {
            if let Some(style) = style.as_text_style_mut() {
                style.update_variable_font();
            }
        });
    }
}
