use crate::mechanical_port::source::{
    animation::state_machine_instance::{EventReport, RuntimeStateMachineInstanceWeakHandle},
    artboard::RuntimeArtboardInstanceWeakHandle,
    core::CoreHandle,
    core_context::CoreContext,
    generated::nested_animation_base::NestedAnimationBase,
    status_code::StatusCode,
};

use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

#[derive(Default)]
pub(crate) struct NestedEventListenerLinks {
    notifiers: RefCell<Vec<Weak<NestedEventNotifierLinks>>>,
}
#[derive(Default)]
struct NestedEventNotifierLinks {
    listeners: RefCell<Vec<RuntimeStateMachineInstanceWeakHandle>>,
}

/// Registration state lives independently of the listener's borrowed machine.
/// Neither endpoint owns the other, and unlinking never enters user callbacks.
#[derive(Default)]
pub struct NestedEventListener {
    links: Rc<NestedEventListenerLinks>,
}
impl NestedEventListener {
    pub(crate) fn downgrade(&self) -> Weak<NestedEventListenerLinks> {
        Rc::downgrade(&self.links)
    }
    pub fn remove_from_nested_event_notifiers(&self) {
        let notifiers = std::mem::take(&mut *self.links.notifiers.borrow_mut());
        for notifier in notifiers {
            if let Some(notifier) = notifier.upgrade() {
                notifier.listeners.borrow_mut().retain(|listener| {
                    !listener
                        .nested_event_listener_links()
                        .is_some_and(|links| Rc::ptr_eq(&links, &self.links))
                });
            }
        }
    }
    pub fn notifier_count(&self) -> usize {
        self.links.notifiers.borrow().len()
    }
}
impl Clone for NestedEventListener {
    fn clone(&self) -> Self {
        Self::default()
    }
    fn clone_from(&mut self, _source: &Self) {}
}
impl Drop for NestedEventListener {
    fn drop(&mut self) {
        self.remove_from_nested_event_notifiers();
    }
}

#[derive(Default)]
pub struct NestedEventNotifier {
    nested_artboard: Option<CoreHandle>,
    links: Rc<NestedEventNotifierLinks>,
}
impl Clone for NestedEventNotifier {
    fn clone(&self) -> Self {
        Self::default()
    }
    fn clone_from(&mut self, _source: &Self) {}
}
impl NestedEventNotifier {
    pub fn add_nested_event_listener(&mut self, listener: RuntimeStateMachineInstanceWeakHandle) {
        let links = listener
            .nested_event_listener_links()
            .expect("live nested event listener registration");
        self.links.listeners.borrow_mut().push(listener);
        links
            .notifiers
            .borrow_mut()
            .push(Rc::downgrade(&self.links));
    }
    pub fn remove_nested_event_listener(
        &mut self,
        listener: RuntimeStateMachineInstanceWeakHandle,
    ) {
        self.links
            .listeners
            .borrow_mut()
            .retain(|candidate| !candidate.ptr_eq(&listener));
        self.unlink_listener(&listener);
    }
    fn unlink_listener(&self, listener: &RuntimeStateMachineInstanceWeakHandle) {
        if let Some(links) = listener.nested_event_listener_links() {
            let this = Rc::downgrade(&self.links);
            links
                .notifiers
                .borrow_mut()
                .retain(|notifier| !notifier.ptr_eq(&this));
        }
    }
    pub fn nested_event_listeners(&self) -> Vec<RuntimeStateMachineInstanceWeakHandle> {
        self.links.listeners.borrow().clone()
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
            .map(|event| EventReport {
                event: Some(event),
                seconds_delay: 0.0,
                host_sequence: 0,
                host_view_model_instance_id: None,
            })
            .collect();
        let nested_artboard = self.nested_artboard.clone();
        for listener in self.nested_event_listeners() {
            listener.with_instance_mut(|listener| {
                listener.notify_nested(&event_reports, nested_artboard.clone())
            });
        }
    }
}
impl Drop for NestedEventNotifier {
    fn drop(&mut self) {
        self.nested_artboard = None;
        let listeners = std::mem::take(&mut *self.links.listeners.borrow_mut());
        for listener in listeners {
            self.unlink_listener(&listener);
        }
    }
}

#[derive(Default)]
pub struct NestedAnimation {
    pub base: NestedAnimationBase,
}

pub trait NestedAnimationBehavior {
    fn advance(&mut self, elapsed_seconds: f32, new_frame: bool) -> bool;
    fn animation_initializer(&self) -> NestedAnimationInitializer;
    fn release_dependencies(&mut self);
}

pub type NestedAnimationInitializer = fn(&CoreHandle, RuntimeArtboardInstanceWeakHandle);

pub fn initialize_animation(owner: &CoreHandle, artboard: RuntimeArtboardInstanceWeakHandle) {
    let initialize = owner
        .with(|owner| owner.nested_animation_initializer())
        .flatten()
        .expect("nested animation exposes its concrete initializer");
    initialize(owner, artboard);
}

impl NestedAnimation {
    pub fn validate(&mut self, context: &mut dyn CoreContext) -> bool {
        if !self.base.base.validate(context) {
            return false;
        }
        context
            .resolve(self.base.base.parent_id())
            .is_some_and(|parent| parent.is_type_of(crate::mechanical_port::source::generated::nested_artboard_base::NestedArtboardBase::TYPE_KEY))
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.base.on_added_dirty(context);
        if code == StatusCode::Ok {
            let animation = self.base.handle();
            let parent = context.resolve(self.base.base.parent_id());
            if let (Some(animation), Some(parent)) = (animation, parent) {
                parent.with_mut(|parent| {
                    parent
                        .as_nested_artboard_mut()
                        .expect("validated NestedArtboard parent")
                        .add_nested_animation_handle(animation);
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

impl crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
    for NestedAnimation
{
    fn notify_property_changed(&mut self, key: u16) {
        crate::mechanical_port::source::core::Core::notify_property_changed(&mut self.base, key);
    }
}
