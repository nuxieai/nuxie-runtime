use crate::mechanical_port::source::generated::animation::any_state_base::AnyStateBase;

#[derive(Default)]
pub struct AnyState {
    pub base: AnyStateBase,
}

impl std::ops::Deref for AnyState {
    type Target = AnyStateBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for AnyState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
