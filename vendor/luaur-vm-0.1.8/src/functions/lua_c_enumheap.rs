use crate::functions::enumgco::enumgco;
use crate::functions::lua_m_visitgco::lua_m_visitgco;
use crate::records::enum_context::EnumContext;
use crate::records::gc_object::GCObject;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::{c_char, c_void};

#[allow(non_snake_case)]
pub unsafe fn lua_c_enumheap(
    l: *mut lua_State,
    context: *mut c_void,
    node: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, u8, u8, usize, *const c_char)>,
    edge: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *const c_char)>,
) {
    let g = (*l).global;

    let mut ctx = EnumContext {
        L: l,
        context,
        node,
        edge,
    };

    // In Luau, lua_State is a collectible object. Its first field is hdr (GCheader),
    // which contains the tt, marked, and memcat fields required by GCObject.
    // The obj2gco macro expects a pointer to something that has these fields.
    // We cast the mainthread (lua_State*) to GCObject* to satisfy the macro and the enumgco signature.
    let mainthread_gco = (*g).mainthread as *mut GCObject;

    enumgco(
        &mut ctx as *mut EnumContext as *mut c_void,
        core::ptr::null_mut(),
        mainthread_gco,
    );

    lua_m_visitgco(
        l,
        &mut ctx as *mut EnumContext as *mut c_void,
        enumgco as *mut c_void,
    );
}

#[allow(non_snake_case)]
pub unsafe fn luaC_enumheap(
    L: *mut lua_State,
    context: *mut c_void,
    node: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, u8, u8, usize, *const c_char)>,
    edge: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *const c_char)>,
) {
    lua_c_enumheap(L, context, node, edge);
}
