//! Node: `cxx:Function:Luau.VM:VM/src/lgc.cpp:1100:getheaptriggererroroffset`
//! Source: `VM/src/lgc.cpp:1100-1130` (hand-ported)

use crate::type_aliases::global_state::global_State;

#[allow(non_snake_case)]
pub(crate) fn getheaptriggererroroffset(g: *mut global_State) -> i64 {
    let gcstats = unsafe { &mut (*g).gcstats };

    let atomicstarttotalsizebytes = gcstats.atomicstarttotalsizebytes;
    let heapgoalsizebytes = gcstats.heapgoalsizebytes;

    let errorKb = (atomicstarttotalsizebytes.wrapping_sub(heapgoalsizebytes) / 1024) as i32;

    const TRIGGERTERMCOUNT: usize = 32;

    let slot = &mut gcstats.triggerterms[gcstats.triggertermpos as usize % TRIGGERTERMCOUNT];
    let prev = *slot;
    *slot = errorKb;
    gcstats.triggerintegral += errorKb - prev;
    gcstats.triggertermpos += 1;

    const KU: f64 = 0.9;
    const TU: f64 = 2.5;

    const KP: f64 = 0.45 * KU;
    const TI: f64 = 0.8 * TU;
    const KI: f64 = 0.54 * KU / TI;

    let proportionalTerm = KP * errorKb as f64;
    let integralTerm = KI * gcstats.triggerintegral as f64;

    let totalTerm = proportionalTerm + integralTerm;

    (totalTerm * 1024.0) as i64
}
