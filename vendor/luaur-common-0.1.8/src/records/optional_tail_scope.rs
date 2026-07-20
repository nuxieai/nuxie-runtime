use crate::records::thread_context::ThreadContext;

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct OptionalTailScope {
    pub(crate) context: *mut ThreadContext,
    pub(crate) token: u16,
    pub(crate) threshold: u32,
    pub(crate) microsec: u32,
    pub(crate) pos: u32,
}

impl Drop for OptionalTailScope {
    fn drop(&mut self) {
        // The implementation of the destructor is a separate item.
        // This file only defines the record structure.
    }
}
