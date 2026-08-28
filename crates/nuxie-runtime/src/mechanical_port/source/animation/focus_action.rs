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
