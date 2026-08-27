use std::ptr::NonNull;

use crate::mechanical_port::source::{
    event::Event, event_report::EventReport, generated::nested_animation_base::NestedAnimationBase,
    status_code::StatusCode,
};

pub trait NestedEventListener<A> {
    fn notify(&mut self, events: &[EventReport], context: *mut A);
}

pub struct NestedEventNotifier<A> {
    nested_artboard: *mut A,
    nested_event_listeners: Vec<NonNull<dyn NestedEventListener<A>>>,
}

impl<A> Default for NestedEventNotifier<A> {
    fn default() -> Self {
        Self {
            nested_artboard: std::ptr::null_mut(),
            nested_event_listeners: Vec::new(),
        }
    }
}

impl<A> NestedEventNotifier<A> {
    pub fn add_nested_event_listener(&mut self, listener: NonNull<dyn NestedEventListener<A>>) {
        self.nested_event_listeners.push(listener);
    }

    pub fn remove_nested_event_listener(&mut self, listener: NonNull<dyn NestedEventListener<A>>) {
        self.nested_event_listeners
            .retain(|item| !std::ptr::addr_eq(item.as_ptr(), listener.as_ptr()));
    }

    pub fn nested_event_listeners(&self) -> Vec<NonNull<dyn NestedEventListener<A>>> {
        self.nested_event_listeners.clone()
    }

    pub fn set_nested_artboard(&mut self, artboard: *mut A) {
        self.nested_artboard = artboard;
    }

    pub fn nested_artboard(&self) -> *mut A {
        self.nested_artboard
    }

    pub fn notify_listeners(&mut self, events: &[NonNull<Event>]) {
        let event_reports: Vec<_> = events
            .iter()
            .map(|event| EventReport::new(event.as_ptr(), 0.0))
            .collect();
        for mut listener in self.nested_event_listeners.iter().copied() {
            unsafe {
                listener
                    .as_mut()
                    .notify(&event_reports, self.nested_artboard)
            };
        }
    }
}

impl<A> Drop for NestedEventNotifier<A> {
    fn drop(&mut self) {
        self.nested_artboard = std::ptr::null_mut();
        self.nested_event_listeners.clear();
    }
}

#[derive(Default)]
pub struct NestedAnimation {
    pub base: NestedAnimationBase,
}

pub trait NestedAnimationBehavior {
    fn advance(&mut self, elapsed_seconds: f32, new_frame: bool) -> bool;
    fn initialize_animation(&mut self, artboard_instance: *mut ());
    fn release_dependencies(&mut self);
}

pub trait NestedAnimationContext {
    fn super_validate(&self, animation: &NestedAnimation) -> bool;
    fn parent_is_nested_artboard(&self, parent_id: u32) -> bool;
    fn super_on_added_dirty(&mut self, animation: &mut NestedAnimation) -> StatusCode;
    fn add_nested_animation(&mut self, animation: &mut NestedAnimation);
}

impl NestedAnimation {
    pub fn validate(&self, context: &dyn NestedAnimationContext) -> bool {
        if !context.super_validate(self) {
            return false;
        }
        context.parent_is_nested_artboard(self.base.base.parent_id())
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn NestedAnimationContext) -> StatusCode {
        let code = context.super_on_added_dirty(self);
        if code == StatusCode::Ok {
            context.add_nested_animation(self);
        }
        code
    }
}
