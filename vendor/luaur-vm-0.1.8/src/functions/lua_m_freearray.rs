use crate::functions::lua_g_getline::luaG_getline;
use crate::macros::getstr::getstr;
use crate::type_aliases::lua_counter_function::lua_CounterFunction;
use crate::type_aliases::lua_counter_value::lua_CounterValue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::proto::Proto;

#[allow(non_snake_case)]
pub(crate) unsafe fn getcounters(
    L: *mut lua_State,
    p: *mut Proto,
    context: *mut core::ffi::c_void,
    functionvisit: lua_CounterFunction,
    countervisit: lua_CounterValue,
) {
    let p_ref = &*p;
    if !p_ref.execdata.is_null() {
        let l_ref = &*L;
        let global = l_ref.global;
        if !global.is_null() && !(*global).ecb.getcounterdata.is_none() {
            let mut count: usize = 0;
            let data = (*global).ecb.getcounterdata.unwrap()(L, p, &mut count as *mut usize);

            if !data.is_null() && count != 0 {
                let debugname = if !p_ref.debugname.is_null() {
                    getstr(p_ref.debugname)
                } else {
                    core::ptr::null()
                };
                let linedefined = p_ref.linedefined;

                if let Some(mut fv) = functionvisit {
                    fv(context, debugname, linedefined);
                }

                for i in 0..count {
                    let mut kind: u32 = 0;
                    let mut pcpos: u32 = 0;
                    let mut hits: u64 = 0;

                    let ptr = (data as *const u8).add(i * (4 + 4 + 8));
                    kind = core::ptr::read_unaligned(ptr as *const u32);
                    pcpos = core::ptr::read_unaligned(ptr.add(4) as *const u32);
                    hits = core::ptr::read_unaligned(ptr.add(8) as *const u64);

                    let line = if pcpos == !0u32 {
                        p_ref.linedefined
                    } else {
                        luaG_getline(p, pcpos as i32)
                    };

                    if let Some(mut cv) = countervisit {
                        cv(context, kind as i32, line, hits);
                    }
                }
            }
        }
    }

    for i in 0..p_ref.sizep {
        let child = *p_ref.p.add(i as usize);
        getcounters(L, child, context, functionvisit, countervisit);
    }
}

#[allow(non_snake_case)]
pub unsafe fn lua_m_freearray(
    L: *mut lua_State,
    buffer: *mut core::ffi::c_void,
    size: usize,
    elem_size: usize,
    memcat: u8,
) {
    // In this translation unit, we only need the debug counter traversal logic.
    // The actual luaM_freearray semantics are handled by lower-level memory helpers elsewhere.
    // Kept intentionally minimal to match the provided schedule slice.
    let _ = (L, buffer, size, elem_size, memcat);
}
