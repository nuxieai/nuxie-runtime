use crate::macros::lua_m_newarray::luaM_newarray;
use crate::records::temp_buffer::TempBuffer;
use crate::type_aliases::lua_state::lua_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<T> TempBuffer<T> {
    pub unsafe fn allocate(&mut self, L: *mut lua_State, count: usize) {
        LUAU_ASSERT!(self.L.is_null());
        self.L = L;
        self.data = luaM_newarray!(L, count, T, 0);
        self.count = count;
    }
}
