use crate::mechanical_port::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceWeakHandle, core::CoreHandle,
    event_report::EventReport, generated::nested_animation_base::NestedAnimationBase,
    status_code::StatusCode,
};

#[derive(Clone)]
pub struct NestedEventNotifier {
    nested_artboard: Option<CoreHandle>,
    nested_event_listeners: Vec<RuntimeStateMachineInstanceWeakHandle>,
}

impl Default for NestedEventNotifier {
    fn default() -> Self {
        Self {
            nested_artboard: None,
            nested_event_listeners: Vec::new(),
        }
    }
}

impl NestedEventNotifier {
    pub fn add_nested_event_listener(&mut self, listener: RuntimeStateMachineInstanceWeakHandle) {
        if self
            .nested_event_listeners
            .iter()
            .any(|candidate| candidate.ptr_eq(&listener))
        {
            return;
        }
        self.nested_event_listeners.push(listener);
    }

    pub fn remove_nested_event_listener(
        &mut self,
        listener: RuntimeStateMachineInstanceWeakHandle,
    ) {
        self.nested_event_listeners
            .retain(|candidate| !candidate.ptr_eq(&listener));
    }

    pub fn nested_event_listeners(&self) -> Vec<RuntimeStateMachineInstanceWeakHandle> {
        self.nested_event_listeners.clone()
    }

    pub fn set_nested_artboard(&mut self, artboard: CoreHandle) {
        self.nested_artboard = Some(artboard);
    }

    pub fn nested_artboard(&self) -> Option<CoreHandle> {
        self.nested_artboard.clone()
    }

    pub fn notify_listeners(&mut self, events: &[CoreHandle]) {
        let Some(nested_artboard) = self.nested_artboard.clone() else {
            return;
        };
        let event_reports: Vec<_> = events
            .iter()
            .cloned()
            .map(|event| EventReport::new(event, 0.0))
            .collect();
        self.nested_event_listeners
            .retain(|listener| listener.upgrade().is_some());
        for listener in &self.nested_event_listeners {
            listener.with_instance_mut(|listener| {
                listener.notify(&event_reports, nested_artboard.clone())
            });
        }
    }
}

impl Drop for NestedEventNotifier {
    fn drop(&mut self) {
        self.nested_artboard = None;
        self.nested_event_listeners.clear();
    }
}

#[derive(Default)]
pub struct NestedAnimation {
    pub base: NestedAnimationBase,
}

pub trait NestedAnimationBehavior {
    fn advance(&mut self, elapsed_seconds: f32, new_frame: bool) -> bool;
    fn initialize_animation(&mut self, artboard_instance: RuntimeArtboardInstanceWeakHandle);
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
impl std::ops::Deref for NestedAnimation {
    type Target = NestedAnimationBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for NestedAnimation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl crate::mechanical_port::source::generated::nested_animation_base::NestedAnimationBaseCallbacks
    for NestedAnimation
{
    fn notify_property_changed(&mut self, key: u16) {
        self.base.notify_property_changed(key);
    }
}
