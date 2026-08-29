use crate::mechanical_port::source::generated::animation::focus_action_base::FocusActionBase;

#[derive(Default)]
pub struct FocusAction {
    pub base: FocusActionBase,
}
impl std::ops::Deref for FocusAction {
    type Target = FocusActionBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for FocusAction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl crate::mechanical_port::source::generated::animation::listener_action_base::ListenerActionBaseCallbacks for FocusAction {
    fn notify_property_changed(&mut self, key: u16) {
        crate::mechanical_port::source::core::Core::notify_property_changed(self, key);
    }
}
