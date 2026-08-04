use crate::records::closure::Closure;
use crate::records::proto::Proto;
use luaur_common::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn luaF_promoteproto(cl: *mut Closure) -> *mut Proto {
    LUAU_ASSERT!((*cl).isC == 0);

    let l = core::ptr::addr_of_mut!((*cl).inner.l) as *mut crate::records::closure::LClosure;
    while !(*(*l).p).optimized.is_null() {
        (*l).p = (*(*l).p).optimized;
        (*cl).stacksize = (*(*l).p).maxstacksize;
    }

    (*l).p
}
