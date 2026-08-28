use crate::mechanical_port::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceWeakHandle,
    artboard::RuntimeArtboardInstanceWeakHandle, core::CoreHandle, core_context::CoreContext,
    event_report::EventReport, generated::nested_animation_base::NestedAnimationBase,
    nested_artboard::NestedArtboard, status_code::StatusCode,
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
        let event_reports: Vec<_> = events
            .iter()
            .cloned()
            .map(|event| EventReport::new(event, 0.0))
            .collect();
        let nested_artboard = self.nested_artboard.clone();
        self.nested_event_listeners
            .retain(|listener| listener.upgrade().is_some());
        for listener in &self.nested_event_listeners {
            listener.with_instance_mut(|listener| {
                listener.notify_nested(&event_reports, nested_artboard.clone())
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

impl NestedAnimation {
    pub fn validate(&mut self, context: &mut dyn CoreContext) -> bool {
        if !self.base.base.validate(context) {
            return false;
        }
        context
            .resolve(self.base.base.parent_id())
            .is_some_and(|parent| parent.with_downcast::<NestedArtboard, _>(|_| ()).is_some())
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.base.on_added_dirty(context);
        if code == StatusCode::Ok {
            let animation = self.base.handle();
            let parent = context.resolve(self.base.base.parent_id());
            if let (Some(animation), Some(parent)) = (animation, parent) {
                parent.with_downcast_mut::<NestedArtboard, _>(|parent| {
                    parent.add_nested_animation_handle(animation);
                });
            }
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
