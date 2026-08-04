use crate::records::page::Page;

#[repr(C)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct Allocator {
    pub(crate) root: *mut Page,
    pub(crate) offset: usize,
}

// Safety: The Allocator owns its pages and does not use thread-local storage.
// It is safe to send to another thread if the memory it manages is not being accessed.
unsafe impl Send for Allocator {}
unsafe impl Sync for Allocator {}
