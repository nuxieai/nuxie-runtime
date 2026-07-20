use crate::records::temp_buffer::TempBuffer;

impl<T> Drop for TempBuffer<T> {
    fn drop(&mut self) {
        if !self.data.is_null() {
            unsafe {
                crate::functions::lua_m_free::luaM_free_(
                    self.L,
                    self.data as *mut core::ffi::c_void,
                    self.count * core::mem::size_of::<T>(),
                    0,
                )
            };
            self.data = core::ptr::null_mut();
            self.count = 0;
        }
    }
}
