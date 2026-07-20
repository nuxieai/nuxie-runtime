use crate::enums::lua_type::lua_Type;
use crate::macros::setnilvalue::setnilvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub(crate) unsafe fn clearstack(l: *mut lua_State) {
    let stack_end: StkId = (*l).stack.wrapping_add((*l).stacksize as usize);
    let mut o: StkId = (*l).top;
    while o < stack_end {
        setnilvalue!(o);
        o = o.wrapping_add(1);
    }
}
