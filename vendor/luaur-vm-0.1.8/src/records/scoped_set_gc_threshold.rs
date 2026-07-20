#[allow(non_camel_case_types)]
#[derive(Debug)]
#[repr(C)]
pub struct ScopedSetGcThreshold {
    pub(crate) global: *mut crate::records::global_state::global_State,
    pub(crate) original_threshold: usize,
}
