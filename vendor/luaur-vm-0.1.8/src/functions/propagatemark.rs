//! Node: `cxx:Function:Luau.VM:VM/src/lgc.cpp:504:propagatemark`
//! Source: `VM/src/lgc.cpp` (lgc.cpp:504-590, hand-ported)

use crate::enums::lua_type::lua_Type;
use crate::functions::clearstack::clearstack;
use crate::functions::shrinkstackprotected::shrinkstackprotected;
use crate::functions::traverseclass::traverseclass;
use crate::functions::traverseclosure::traverseclosure;
use crate::functions::traverseobject::traverseobject;
use crate::functions::traverseproto::traverseproto;
use crate::functions::traversestack::traversestack;
use crate::functions::traversetable::traversetable;
use crate::macros::black_2_gray::black2gray;
use crate::macros::gc_satomic::GCSatomic;
use crate::macros::gc_spropagate::GCSpropagate;
use crate::macros::gco_2_cl::gco2cl;
use crate::macros::gco_2_class::gco2class;
use crate::macros::gco_2_h::gco2h;
use crate::macros::gco_2_object::gco2object;
use crate::macros::gco_2_p::gco2p;
use crate::macros::gco_2_th::gco2th;
use crate::macros::gray_2_black::gray2black;
use crate::macros::isgray::isgray;
use crate::macros::size_cclosure::size_cclosure;
use crate::macros::size_lclosure::size_lclosure;
use crate::macros::sizenode::sizenode;
use crate::records::call_info::CallInfo;
use crate::records::closure::Closure;
use crate::records::global_state::global_State;
use crate::records::loc_var::LocVar;
use crate::records::lua_node::LuaNode;
use crate::records::lua_table::LuaTable;
use crate::records::luau_class::LuauClass;
use crate::records::luau_object::LuauObject;
use crate::records::proto::Proto;
use crate::records::t_string::TString;
use crate::type_aliases::instruction::Instruction;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;
use core::ffi::c_int;
use core::mem::size_of;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

// traverse one gray object, turning it to black.
// Returns `quantity' traversed.
#[allow(non_snake_case)]
pub(crate) unsafe fn propagatemark(g: *mut global_State) -> usize {
    let o = (*g).gray;
    LUAU_ASSERT!(isgray!(o));
    gray2black!(o);
    match (*o).gch.tt as i32 {
        t if t == lua_Type::LUA_TTABLE as i32 => {
            let h = gco2h!(o) as *const _ as *mut LuaTable;
            (*g).gray = (*h).gclist;
            if traversetable(g, h) != 0 {
                // table is weak?
                black2gray!(o); // keep it gray
            }
            size_of::<LuaTable>()
                + size_of::<TValue>() * (*h).sizearray as usize
                + size_of::<LuaNode>() * sizenode!(h) as usize
        }
        t if t == lua_Type::LUA_TFUNCTION as i32 => {
            let cl = gco2cl!(o) as *const _ as *mut Closure;
            (*g).gray = (*cl).gclist;
            traverseclosure(g, cl);
            if (*cl).isC != 0 {
                size_cclosure((*cl).nupvalues as c_int)
            } else {
                size_lclosure((*cl).nupvalues as usize)
            }
        }
        t if t == lua_Type::LUA_TTHREAD as i32 => {
            let th = gco2th!(o) as *const _ as *mut lua_State;
            (*g).gray = (*th).gclist;

            let active = (*th).isactive || th == (*(*th).global).mainthread;

            traversestack(g, th);

            // active threads will need to be rescanned later to mark new stack writes so we mark them gray again
            if active {
                (*th).gclist = (*g).grayagain;
                (*g).grayagain = o;

                black2gray!(o);
            }

            // the stack needs to be cleared after the last modification of the thread state before sweep begins
            // if the thread is inactive, we might not see the thread in this cycle so we must clear it now
            if !active || (*g).gcstate as i32 == GCSatomic {
                clearstack(th);
            }

            // we could shrink stack at any time but we opt to do it during initial mark to do that just once per cycle
            if (*g).gcstate as i32 == GCSpropagate {
                shrinkstackprotected(th);
            }

            size_of::<lua_State>()
                + size_of::<TValue>() * (*th).stacksize as usize
                + size_of::<CallInfo>() * (*th).size_ci as usize
        }
        t if t == lua_Type::LUA_TPROTO as i32 => {
            let p = gco2p!(o) as *const _ as *mut Proto;
            (*g).gray = (*p).gclist;
            traverseproto(g, p);

            size_of::<Proto>()
                + size_of::<Instruction>() * (*p).sizecode as usize
                + size_of::<*mut Proto>() * (*p).sizep as usize
                + size_of::<TValue>() * (*p).sizek as usize
                + (*p).sizelineinfo as usize
                + size_of::<LocVar>() * (*p).sizelocvars as usize
                + size_of::<*mut TString>() * (*p).sizeupvalues as usize
                + (*p).sizetypeinfo as usize
        }
        t if t == lua_Type::LUA_TCLASS as i32 => {
            let classobject = gco2class!(o) as *const _ as *mut LuauClass;
            (*g).gray = (*classobject).gclist;
            traverseclass(g, classobject);
            // We've traversed the "object" itself ...
            size_of::<LuauClass>()
                // ... plus the method closures, each a `TValue` wide ...
                + (((*classobject).numberofallmembers - (*classobject).numberofinstancemembers)
                    as usize
                    * size_of::<TValue>())
                // ... plus a string pointer for each method or property, each a pointer wide.
                + ((*classobject).numberofallmembers as usize * size_of::<*mut TString>())
        }
        t if t == lua_Type::LUA_TOBJECT as i32 => {
            let classinst = gco2object!(o) as *const _ as *mut LuauObject;
            (*g).gray = (*classinst).gclist;
            traverseobject(g, classinst);
            // We've traversed the instance ...
            size_of::<LuauObject>()
                // ... plus all of the instance fields.
                + (*classinst).numberofmembers as usize * size_of::<TValue>()
        }
        _ => {
            LUAU_ASSERT!(false);
            0
        }
    }
}
