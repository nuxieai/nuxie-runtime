use crate::records::local::Local;

#[allow(non_snake_case)]
impl Local {
    pub fn new() -> Self {
        Self {
            local: core::ptr::null_mut(),
            offset: 0,
        }
    }
}

#[allow(non_snake_case)]
pub fn parser_local_local() -> Local {
    Local::new()
}
