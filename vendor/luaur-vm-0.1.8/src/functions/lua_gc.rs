use crate::enums::lua_gc_op::lua_GCOp;
use crate::functions::lua_c_fullgc::lua_c_fullgc;
use crate::functions::lua_c_step::luaC_step;
use crate::functions::lua_c_validate::lua_c_validate;
use crate::macros::cast_int::cast_int;
use crate::macros::condhardmemtests::condhardmemtests;
use crate::records::gc_cycle_metrics::GCCycleMetrics;
use crate::records::global_state::global_State;
use crate::type_aliases::lua_state::lua_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

pub const GCSpause: core::ffi::c_int = 0;
pub const GCSpropagate: core::ffi::c_int = 1;
pub const GCSpropagateagain: core::ffi::c_int = 2;
pub const GCSatomic: core::ffi::c_int = 3;
pub const GCSsweep: core::ffi::c_int = 4;

#[allow(non_snake_case)]
pub fn lua_gc(
    L: *mut lua_State,
    what: core::ffi::c_int,
    data: core::ffi::c_int,
) -> core::ffi::c_int {
    let mut res: core::ffi::c_int = 0;
    unsafe {
        condhardmemtests!(lua_c_validate(L), 1);
        let g: *mut global_State = (*L).global;
        match what as i32 {
            x if x == lua_GCOp::LUA_GCSTOP as i32 => {
                (*g).GCthreshold = usize::MAX;
            }
            x if x == lua_GCOp::LUA_GCRESTART as i32 => {
                (*g).GCthreshold = (*g).totalbytes;
            }
            x if x == lua_GCOp::LUA_GCCOLLECT as i32 => {
                lua_c_fullgc(L);
            }
            x if x == lua_GCOp::LUA_GCCOUNT as i32 => {
                res = cast_int!((*g).totalbytes >> 10);
            }
            x if x == lua_GCOp::LUA_GCCOUNTB as i32 => {
                res = cast_int!((*g).totalbytes & 1023);
            }
            x if x == lua_GCOp::LUA_GCISRUNNING as i32 => {
                res = if (*g).GCthreshold != usize::MAX { 1 } else { 0 };
            }
            x if x == lua_GCOp::LUA_GCSTEP as i32 => {
                let amount: usize = (data as usize) << 10;
                let gcstate_i32: i32 = i32::from((*g).gcstate);

                let oldcredit: ptrdiff_t = if gcstate_i32 == GCSpause {
                    0
                } else {
                    (*g).GCthreshold as ptrdiff_t - (*g).totalbytes as ptrdiff_t
                };

                // temporarily adjust the threshold so that we can perform GC work
                if amount <= (*g).totalbytes {
                    (*g).GCthreshold = (*g).totalbytes - amount;
                } else {
                    (*g).GCthreshold = 0;
                }

                #[cfg(feature = "luai_gcmetrics")]
                let startmarktime = (*g).gcmetrics.currcycle.marktime;
                #[cfg(feature = "luai_gcmetrics")]
                let startsweeptime = (*g).gcmetrics.currcycle.sweeptime;

                // track how much work the loop will actually perform
                let mut actualwork: usize = 0;

                while (*g).GCthreshold <= (*g).totalbytes {
                    let stepsize = luaC_step(L, false);
                    actualwork += stepsize;

                    let gcstate_i32 = i32::from((*g).gcstate);
                    if gcstate_i32 == GCSpause {
                        res = 1; // signal it
                        break;
                    }
                }

                #[cfg(feature = "luai_gcmetrics")]
                {
                    // record explicit step statistics
                    let cyclemetrics: &mut GCCycleMetrics = if i32::from((*g).gcstate) == GCSpause {
                        &mut (*g).gcmetrics.lastcycle
                    } else {
                        &mut (*g).gcmetrics.currcycle
                    };

                    let totalmarktime = cyclemetrics.marktime - startmarktime;
                    let totalsweeptime = cyclemetrics.sweeptime - startsweeptime;

                    if totalmarktime > 0.0 {
                        cyclemetrics.markexplicitsteps += 1;

                        if totalmarktime > cyclemetrics.markmaxexplicittime {
                            cyclemetrics.markmaxexplicittime = totalmarktime;
                        }
                    }

                    if totalsweeptime > 0.0 {
                        cyclemetrics.sweepexplicitsteps += 1;

                        if totalsweeptime > cyclemetrics.sweepmaxexplicittime {
                            cyclemetrics.sweepmaxexplicittime = totalsweeptime;
                        }
                    }
                }

                // if cycle hasn't finished, advance threshold forward for the amount of extra work performed
                let gcstate_i32 = i32::from((*g).gcstate);
                if gcstate_i32 != GCSpause {
                    // if a new cycle was triggered by explicit step, old 'credit' of GC work is 0
                    let newthreshold =
                        (*g).totalbytes as ptrdiff_t + actualwork as ptrdiff_t + oldcredit;
                    (*g).GCthreshold = if newthreshold < 0 {
                        0
                    } else {
                        newthreshold as usize
                    };
                }
            }
            x if x == lua_GCOp::LUA_GCSETGOAL as i32 => {
                res = (*g).gcgoal;
                (*g).gcgoal = data;
            }
            x if x == lua_GCOp::LUA_GCSETSTEPMUL as i32 => {
                res = (*g).gcstepmul;
                (*g).gcstepmul = data;
            }
            x if x == lua_GCOp::LUA_GCSETSTEPSIZE as i32 => {
                // GC values are expressed in Kbytes: #bytes/2^10
                res = (*g).gcstepsize >> 10;
                (*g).gcstepsize = data << 10;
            }
            _ => {
                res = -1; // invalid option
            }
        }
    }
    res
}

type ptrdiff_t = core::ffi::c_long;
