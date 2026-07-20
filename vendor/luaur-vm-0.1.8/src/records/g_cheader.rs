#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GCheader {
    pub tt: u8,
    pub marked: u8,
    pub memcat: u8,
}

impl Default for GCheader {
    fn default() -> Self {
        Self {
            tt: 0,
            marked: 0,
            memcat: 0,
        }
    }
}
