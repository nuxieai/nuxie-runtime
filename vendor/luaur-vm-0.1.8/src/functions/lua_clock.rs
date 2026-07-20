use crate::functions::clock_period::clock_period;
use crate::functions::clock_timestamp::clock_timestamp;

pub fn lua_clock() -> f64 {
    use core::sync::atomic::{AtomicU64, Ordering};

    static PERIOD_BITS: AtomicU64 = AtomicU64::new(0);

    let mut bits = PERIOD_BITS.load(Ordering::Relaxed);

    if bits == 0 {
        let p = clock_period();
        bits = p.to_bits();
        PERIOD_BITS.store(bits, Ordering::Relaxed);
    }

    let period = f64::from_bits(bits);
    clock_timestamp() * period
}
