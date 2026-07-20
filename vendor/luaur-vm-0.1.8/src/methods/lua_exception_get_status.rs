#[cfg(any())]
use crate::records::lua_exception::lua_exception;

#[cfg(any())]
impl lua_exception {
    /// C++ `int getStatus() const`
    pub fn get_status(&self) -> core::ffi::c_int {
        self.status
    }
}
