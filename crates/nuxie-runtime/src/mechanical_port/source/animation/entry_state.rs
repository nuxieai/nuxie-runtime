use crate::mechanical_port::source::generated::animation::entry_state_base::EntryStateBase;

#[derive(Default)]
pub struct EntryState {
    pub base: EntryStateBase,
}

impl std::ops::Deref for EntryState {
    type Target = EntryStateBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for EntryState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
