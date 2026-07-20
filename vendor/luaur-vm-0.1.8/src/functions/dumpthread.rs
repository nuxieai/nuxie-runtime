use crate::functions::dumpref::dumpref;
use crate::functions::dumprefs::dumprefs;
use crate::functions::dumpstringdata::dumpstringdata;
use crate::functions::lua_f_findlocal::luaF_findlocal;
use crate::macros::ci_func::ci_func;
use crate::macros::clvalue::clvalue;
use crate::macros::getstr::getstr;
use crate::macros::is_lua::isLua;
use crate::macros::iscollectable::iscollectable;
use crate::macros::obj_2_gco::obj2gco;
use crate::macros::pc_rel::pcRel;
use crate::macros::ttisfunction::ttisfunction;
use crate::records::call_info::CallInfo;
use crate::records::closure::Closure;
use crate::records::loc_var::LocVar;
use crate::records::lua_state::lua_State;
use crate::records::proto::Proto;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;
use core::ffi::{c_char, c_int, c_void};

pub(crate) unsafe fn dumpthread(f: *mut c_void, th: *mut lua_State) {
    extern "C" {
        fn fprintf(stream: *mut c_void, format: *const c_char, ...) -> c_int;
        fn fputc(c: c_int, stream: *mut c_void) -> c_int;
    }

    let size = core::mem::size_of::<lua_State>()
        + core::mem::size_of::<TValue>() * (*th).stacksize as usize
        + core::mem::size_of::<CallInfo>() * (*th).size_ci as usize;

    fprintf(
        f,
        c"{\"type\":\"thread\",\"cat\":%d,\"size\":%d".as_ptr(),
        (*th).hdr.memcat as c_int,
        size as c_int,
    );

    fprintf(f, c",\"env\":".as_ptr());
    dumpref(f, obj2gco!((*th).gt));

    let mut tcl: *mut Closure = core::ptr::null_mut();
    let mut ci = (*th).base_ci;
    while ci <= (*th).ci {
        if ttisfunction!((*ci).func) {
            tcl = clvalue!((*ci).func);
            break;
        }
        ci = ci.add(1);
    }

    if !tcl.is_null() && (*tcl).isC == 0 {
        let tcl_l = core::ptr::addr_of!((*tcl).inner.l).cast::<crate::records::closure::LClosure>();
        let tcl_p: *mut Proto = (*tcl_l).p;
        if !(*tcl_p).source.is_null() {
            let p: *mut Proto = tcl_p;
            fprintf(f, c",\"source\":\"".as_ptr());
            dumpstringdata(f, getstr((*p).source), (*(*p).source).len as usize);
            fprintf(f, c"\",\"line\":%d".as_ptr(), (*p).linedefined);
        }
    }

    if (*th).top > (*th).stack {
        fprintf(f, c",\"stack\":[".as_ptr());
        dumprefs(f, (*th).stack, (*th).top.offset_from((*th).stack) as usize);
        fprintf(f, c"]".as_ptr());

        let mut ci = (*th).base_ci;
        let mut first = true;
        fprintf(f, c",\"stacknames\":[".as_ptr());

        let mut v: StkId = (*th).stack;
        while v < (*th).top {
            if iscollectable!(v) {
                while ci < (*th).ci && v >= (*ci.add(1)).func {
                    ci = ci.add(1);
                }

                if !first {
                    fputc(',' as c_int, f);
                }
                first = false;

                if v == (*ci).func {
                    let cl = ci_func!(ci);
                    if (*cl).isC != 0 {
                        let c = core::ptr::addr_of!((*cl).inner.c)
                            .cast::<crate::records::closure::CClosure>();
                        fprintf(
                            f,
                            c"\"frame:%s\"".as_ptr(),
                            if !(*c).debugname.is_null() {
                                (*c).debugname
                            } else {
                                c"[C]".as_ptr()
                            },
                        );
                    } else {
                        let lcl = core::ptr::addr_of!((*cl).inner.l)
                            .cast::<crate::records::closure::LClosure>();
                        let p = (*lcl).p;
                        fprintf(f, c"\"frame:".as_ptr());
                        if !(*p).source.is_null() {
                            dumpstringdata(f, getstr((*p).source), (*(*p).source).len as usize);
                        }
                        fprintf(
                            f,
                            c":%d:%s\"".as_ptr(),
                            (*p).linedefined,
                            if !(*p).debugname.is_null() {
                                getstr((*p).debugname)
                            } else {
                                c"".as_ptr()
                            },
                        );
                    }
                } else if isLua!(ci) {
                    let cl = ci_func!(ci);
                    let lcl = core::ptr::addr_of!((*cl).inner.l)
                        .cast::<crate::records::closure::LClosure>();
                    let p = (*lcl).p;
                    let pc = pcRel!((*ci).savedpc, p);
                    let var: *const LocVar =
                        luaF_findlocal(p, v.offset_from((*ci).base) as c_int, pc);

                    if !var.is_null() && !(*var).varname.is_null() {
                        fprintf(f, c"\"%s\"".as_ptr(), getstr((*var).varname));
                    } else {
                        fprintf(f, c"null".as_ptr());
                    }
                } else {
                    fprintf(f, c"null".as_ptr());
                }
            }

            v = v.add(1);
        }
        fprintf(f, c"]".as_ptr());
    }

    fprintf(f, c"}".as_ptr());
}
