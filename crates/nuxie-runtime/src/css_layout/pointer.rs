//! Pointer events in CSS content (docs/CSS_LAYOUT.md, 4.5): an event
//! bubbles from the topmost hit to the CSS boxes it is in, and a pointer's
//! drag goes to the innermost drag handler under it. Translated content
//! keeps Rive's hit testing.
use crate::source::{
    core::CoreHandle, drawable::RuntimeDrawableOccurrence, layout_component::LayoutComponent,
};

/// Whether `component` is a CSS box (docs/CSS_LAYOUT.md, 2.1).
pub(crate) fn is_css_box(component: &CoreHandle) -> bool {
    component
        .with(|object| {
            object
                .as_layout_component()
                .is_some_and(LayoutComponent::is_css_mode)
        })
        .unwrap_or(false)
}

/// The component a hit stands for: a box's listeners hit through its proxy.
pub(crate) fn hit_owner(hit: &RuntimeDrawableOccurrence) -> CoreHandle {
    match hit {
        RuntimeDrawableOccurrence::Authored(handle) => handle.clone(),
        RuntimeDrawableOccurrence::RuntimeProxy(proxy) => proxy.borrow().owner_handle(),
    }
}

/// The CSS box whose own hit `hit` is, if it is one.
pub(crate) fn css_box_of(hit: &RuntimeDrawableOccurrence) -> Option<CoreHandle> {
    let owner = hit_owner(hit);
    is_css_box(&owner).then_some(owner)
}

/// The CSS boxes an event that reaches `hit` bubbles to, innermost first.
/// Translated layouts between them do not count.
pub(crate) fn css_boxes_around(hit: &RuntimeDrawableOccurrence) -> Vec<CoreHandle> {
    let mut boxes = Vec::new();
    let mut component = hit_owner(hit);
    while let Some(parent) = component
        .with(|object| object.component_parent_handle())
        .flatten()
    {
        if is_css_box(&parent) {
            boxes.push(parent.clone());
        }
        component = parent;
    }
    boxes
}

/// Whether `hit` is CSS content: a CSS box's own hit or inside one.
pub(crate) fn in_css(hit: &RuntimeDrawableOccurrence) -> bool {
    css_box_of(hit).is_some() || !css_boxes_around(hit).is_empty()
}
