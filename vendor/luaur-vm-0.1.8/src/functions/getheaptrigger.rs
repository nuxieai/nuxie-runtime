use crate::functions::getheaptriggererroroffset::getheaptriggererroroffset;
use crate::records::gc_stats::GCStats;
use crate::type_aliases::global_state::global_State;

pub(crate) fn getheaptrigger(g: *mut global_State, heapgoal: usize) -> usize {
    const DURATION_THRESHOLD: f64 = 1e-3;

    let gcstats = unsafe { &(*g).gcstats };

    let allocationduration = gcstats.atomicstarttimestamp - gcstats.endtimestamp;

    if allocationduration < DURATION_THRESHOLD {
        return heapgoal;
    }

    let allocationrate = (gcstats.atomicstarttotalsizebytes as f64
        - gcstats.endtotalsizebytes as f64)
        / allocationduration;
    let markduration = gcstats.atomicstarttimestamp - gcstats.starttimestamp;

    let expectedgrowth = (markduration * allocationrate) as i64;
    let offset = getheaptriggererroroffset(g);
    let heaptrigger = heapgoal as i64 - (expectedgrowth + offset);

    let totalbytes = unsafe { (*g).totalbytes as i64 };

    if heaptrigger < totalbytes {
        unsafe { (*g).totalbytes }
    } else if heaptrigger > heapgoal as i64 {
        heapgoal
    } else {
        heaptrigger as usize
    }
}
