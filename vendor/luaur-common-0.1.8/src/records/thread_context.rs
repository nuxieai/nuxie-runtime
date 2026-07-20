extern crate alloc;

use crate::records::event::Event;
use crate::records::global_context::GlobalContext;
use alloc::vec::Vec;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ThreadContext {
    pub(crate) global_context: Arc<GlobalContext>,
    pub(crate) thread_id: u32,
    pub(crate) events: Vec<Event>,
    pub(crate) data: Vec<core::ffi::c_char>,
}

impl ThreadContext {
    pub(crate) const kEventFlushLimit: usize = 8192;
}
