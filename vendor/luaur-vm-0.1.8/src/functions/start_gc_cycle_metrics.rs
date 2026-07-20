use crate::functions::lua_clock::lua_clock;
use crate::records::global_state::global_State;

#[cfg(feature = "luai_gcmetrics")]
#[allow(non_snake_case)]
pub fn start_gc_cycle_metrics(g: *mut global_State) {
    unsafe {
        (*g).gcmetrics.currcycle.starttimestamp = lua_clock();
        (*g).gcmetrics.currcycle.pausetime =
            (*g).gcmetrics.currcycle.starttimestamp - (*g).gcmetrics.lastcycle.endtimestamp;
    }
}

#[cfg(not(feature = "luai_gcmetrics"))]
#[allow(non_snake_case)]
pub fn start_gc_cycle_metrics(_g: *mut global_State) {}
