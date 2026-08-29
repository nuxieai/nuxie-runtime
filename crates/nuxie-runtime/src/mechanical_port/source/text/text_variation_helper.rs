use crate::mechanical_port::source::{
    component::{Component, ComponentOccurrenceHandle},
    component_dirt::ComponentDirt,
    core::CoreHandle,
};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub struct RuntimeTextVariationHelperHandle(Rc<RefCell<TextVariationHelper>>);
impl RuntimeTextVariationHelperHandle {
    pub fn new(text_style: CoreHandle) -> Self {
        Self(Rc::new_cyclic(|weak| {
            let mut component = Component::default();
            component.bind_runtime_occurrence(ComponentOccurrenceHandle::TextVariationHelper(
                weak.clone(),
            ));
            RefCell::new(TextVariationHelper {
                component,
                text_style,
            })
        }))
    }
    pub fn occurrence(&self) -> ComponentOccurrenceHandle {
        ComponentOccurrenceHandle::TextVariationHelper(Rc::downgrade(&self.0))
    }
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut TextVariationHelper) -> R) -> R {
        f(&mut self.0.borrow_mut())
    }
}
pub struct TextVariationHelper {
    pub component: Component,
    text_style: CoreHandle,
}
impl TextVariationHelper {
    pub fn style(&self) -> CoreHandle {
        self.text_style.clone()
    }
    pub fn build_dependencies_for_text(&mut self, text: CoreHandle) {
        if let Some(dependent) = self.component.occurrence_handle() {
            text.with(|text| text.component_artboard_handle())
                .flatten()
                .and_then(|artboard| {
                    artboard.with_mut(|artboard| {
                        artboard
                            .as_component_mut()
                            .map(|component| component.add_dependent(dependent.clone()))
                    })
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
