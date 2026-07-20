use crate::functions::lua_g_getline::luaG_getline;
use crate::macros::getstr::getstr;
use crate::records::proto::Proto;
use crate::type_aliases::instruction::Instruction;
use crate::type_aliases::lua_coverage::lua_Coverage;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::macros::luau_insn_e::LUAU_INSN_E;
use luaur_common::macros::luau_insn_op::LUAU_INSN_OP;

#[allow(non_snake_case)]
pub(crate) unsafe fn getcoverage(
    p: *mut Proto,
    depth: core::ffi::c_int,
    buffer: *mut core::ffi::c_int,
    size: usize,
    context: *mut core::ffi::c_void,
    callback: lua_Coverage,
) {
    core::ptr::write_bytes(buffer, 0xFF, size);

    let p_ref = &*p;
    for i in 0..p_ref.sizecode {
        let insn = *p_ref.code.add(i as usize);
        if LUAU_INSN_OP(insn) != LuauOpcode::LOP_COVERAGE as u32 {
            continue;
        }

        let line = luaG_getline(p, i);
        let hits = LUAU_INSN_E(insn);

        LUAU_ASSERT!((line as usize) < size);
        let val = *buffer.add(line as usize);
        if val < hits {
            *buffer.add(line as usize) = hits;
        }
    }

    let debugname = if !p_ref.debugname.is_null() {
        getstr(p_ref.debugname)
    } else {
        core::ptr::null()
    };
    let linedefined = p_ref.linedefined;

    if let Some(cb) = callback {
        cb(context, debugname, linedefined, depth, buffer, size);
    }

    for i in 0..p_ref.sizep {
        getcoverage(
            *p_ref.p.add(i as usize),
            depth + 1,
            buffer,
            size,
            context,
            callback,
        );
    }
}
