use crate::functions::lua_clock::lua_clock;

pub fn record_gc_delta_time(timer: &mut f64) -> f64 {
    let now = lua_clock();
    let delta = now - *timer;
    *timer = now;
    delta
}
