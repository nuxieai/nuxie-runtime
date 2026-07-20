#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct UpValOpen {
    pub prev: *mut UpVal,
    pub next: *mut UpVal,
    pub threadnext: *mut UpVal,
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Copy, Clone)]
pub union UpValInner {
    pub value: crate::type_aliases::t_value::TValue,
    pub open: UpValOpen,
}

impl core::fmt::Debug for UpValInner {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UpValInner").finish_non_exhaustive()
    }
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct UpVal {
    pub hdr: crate::records::g_cheader::GCheader,
    pub markedopen: u8,
    pub _padding: [u8; 4],
    pub v: *mut crate::type_aliases::t_value::TValue,
    pub u: UpValInner,
}

#[allow(non_camel_case_types)]
pub type UpValRecord = UpVal;
