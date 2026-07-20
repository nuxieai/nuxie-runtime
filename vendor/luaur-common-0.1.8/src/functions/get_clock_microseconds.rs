use crate::functions::get_clock_period::get_clock_period;
use crate::functions::get_clock_timestamp::get_clock_timestamp;
use std::sync::OnceLock;

pub fn get_clock_microseconds() -> u32 {
    struct ClockState {
        period: f64,
        start: f64,
    }

    static STATE: OnceLock<ClockState> = OnceLock::new();

    let state = STATE.get_or_init(|| ClockState {
        period: get_clock_period() * 1e6,
        start: get_clock_timestamp(),
    });

    ((get_clock_timestamp() - state.start) * state.period) as u32
}
