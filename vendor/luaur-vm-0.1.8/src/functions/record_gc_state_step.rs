use crate::records::global_state::global_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

pub const GCSpause: core::ffi::c_int = 0;
pub const GCSpropagate: core::ffi::c_int = 1;
pub const GCSpropagateagain: core::ffi::c_int = 2;
pub const GCSatomic: core::ffi::c_int = 3;
pub const GCSsweep: core::ffi::c_int = 4;

#[allow(non_snake_case)]
pub fn record_gc_state_step(
    g: *mut global_State,
    startgcstate: core::ffi::c_int,
    seconds: f64,
    assist: bool,
    work: usize,
) {
    #[cfg(feature = "luai_gcmetrics")]
    unsafe {
        match startgcstate {
            GCSpause => {
                if (*g).gcstate as core::ffi::c_int == GCSpropagate {
                    (*g).gcmetrics.currcycle.marktime += seconds;
                    if assist {
                        (*g).gcmetrics.currcycle.markassisttime += seconds;
                    }
                }
            }
            GCSpropagate | GCSpropagateagain => {
                (*g).gcmetrics.currcycle.marktime += seconds;
                (*g).gcmetrics.currcycle.markwork += work;
                if assist {
                    (*g).gcmetrics.currcycle.markassisttime += seconds;
                }
            }
            GCSatomic => {
                (*g).gcmetrics.currcycle.atomictime += seconds;
            }
            GCSsweep => {
                (*g).gcmetrics.currcycle.sweeptime += seconds;
                (*g).gcmetrics.currcycle.sweepwork += work;
                if assist {
                    (*g).gcmetrics.currcycle.sweepassisttime += seconds;
                }
            }
            _ => {
                LUAU_ASSERT!(false);
            }
        }

        if assist {
            (*g).gcmetrics.stepassisttimeacc += seconds;
            (*g).gcmetrics.currcycle.assistwork += work;
        } else {
            (*g).gcmetrics.stepexplicittimeacc += seconds;
            (*g).gcmetrics.currcycle.explicitwork += work;
        }
    }
}
