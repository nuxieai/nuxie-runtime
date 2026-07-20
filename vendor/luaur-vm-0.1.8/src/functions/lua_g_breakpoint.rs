use crate::functions::lua_g_getline::luaG_getline;
use crate::macros::lua_m_newarray::luaM_newarray;
use crate::records::global_state::global_State;
use crate::records::proto::Proto;
use crate::type_aliases::lua_state::lua_State;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::macros::luau_insn_op::LUAU_INSN_OP;

/// C++ `void luaG_breakpoint(lua_State* L, Proto* p, int line, bool enable)`.
#[allow(non_snake_case)]
pub unsafe fn lua_g_breakpoint(
    L: *mut lua_State,
    p: *mut Proto,
    line: core::ffi::c_int,
    enable: bool,
) {
    let ondisable = (*(*L).global).ecb.disable;

    if !(*p).lineinfo.is_null() && (ondisable.is_some() || (*p).execdata.is_null()) {
        for i in 0..(*p).sizecode {
            if LUAU_INSN_OP(*(*p).code.add(i as usize)) == LuauOpcode::LOP_PREPVARARGS as u32 {
                continue;
            }

            if luaG_getline(p, i as core::ffi::c_int) != line {
                continue;
            }

            if (*p).debuginsn.is_null() {
                (*p).debuginsn =
                    luaM_newarray!(L, (*p).sizecode, core::ffi::c_uchar, (*p).hdr.memcat);
                for j in 0..(*p).sizecode {
                    *((*p).debuginsn.add(j as usize)) =
                        LUAU_INSN_OP(*(*p).code.add(j as usize)) as core::ffi::c_uchar;
                }
            }

            let op = if enable {
                LuauOpcode::LOP_BREAK as u32
            } else {
                *((*p).debuginsn.add(i as usize)) as u32
            };

            (*(*p).code.add(i as usize)) &=
                !(0xff as crate::type_aliases::instruction::Instruction);
            (*(*p).code.add(i as usize)) |= op as crate::type_aliases::instruction::Instruction;
            LUAU_ASSERT!(LUAU_INSN_OP(*(*p).code.add(i as usize)) == op);

            if enable && !(*p).execdata.is_null() && ondisable.is_some() {
                ondisable.unwrap()(L, p);
            }

            break;
        }
    }

    for i in 0..(*p).sizep {
        lua_g_breakpoint(L, *((*p).p.add(i as usize)), line, enable);
    }
}
