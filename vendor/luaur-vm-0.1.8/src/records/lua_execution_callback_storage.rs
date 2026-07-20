#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(C, align(16))]
pub struct LuaExecutionCallbackStorage {
    pub bytes: [u8; 512],
}

impl LuaExecutionCallbackStorage {
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.bytes.as_mut_ptr()
    }
}
