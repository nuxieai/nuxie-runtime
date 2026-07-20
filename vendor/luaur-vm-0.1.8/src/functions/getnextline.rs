use crate::functions::lua_g_getline::luaG_getline;
use crate::type_aliases::proto::Proto;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_insn_op::LUAU_INSN_OP;

#[allow(non_snake_case)]
pub unsafe fn getnextline(p: *mut Proto, line: core::ffi::c_int) -> core::ffi::c_int {
    let mut closest = -1;

    if !(*p).lineinfo.is_null() {
        for i in 0..(*p).sizecode {
            if LUAU_INSN_OP(*(*p).code.add(i as usize)) == LuauOpcode::LOP_PREPVARARGS as u32 {
                continue;
            }

            let candidate = luaG_getline(p, i as core::ffi::c_int);

            if candidate == line {
                return line;
            }

            if candidate > line && (closest == -1 || candidate < closest) {
                closest = candidate;
            }
        }
    }

    for i in 0..(*p).sizep {
        let candidate = getnextline(*(*p).p.add(i as usize), line);

        if candidate == line {
            return line;
        }

        if candidate > line && (closest == -1 || candidate < closest) {
            closest = candidate;
        }
    }

    closest
}
