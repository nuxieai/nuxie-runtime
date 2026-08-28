use crate::mechanical_port::source::core::{CoreArena, CoreHandle};

pub use crate::mechanical_port::source::status_code::StatusCode;

pub trait CoreContext {
    fn core_arena(&self) -> &CoreArena;
    fn resolve_handle(&self, id: u32) -> Option<CoreHandle>;

    fn resolve(&self, id: u32) -> Option<CoreHandle> {
        self.resolve_handle(id)
            .filter(|handle| self.core_arena().contains(handle))
    }
}
