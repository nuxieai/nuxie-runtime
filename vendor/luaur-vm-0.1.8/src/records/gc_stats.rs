#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GCStats {
    pub(crate) triggerterms: [i32; 32],
    pub(crate) triggertermpos: u32,
    pub(crate) triggerintegral: i32,
    pub(crate) atomicstarttotalsizebytes: usize,
    pub(crate) endtotalsizebytes: usize,
    pub(crate) heapgoalsizebytes: usize,
    pub(crate) starttimestamp: f64,
    pub(crate) atomicstarttimestamp: f64,
    pub(crate) endtimestamp: f64,
}

#[allow(non_camel_case_types)]
pub type gc_stats = GCStats;

impl Default for GCStats {
    fn default() -> Self {
        Self {
            triggerterms: [0; 32],
            triggertermpos: 0,
            triggerintegral: 0,
            atomicstarttotalsizebytes: 0,
            endtotalsizebytes: 0,
            heapgoalsizebytes: 0,
            starttimestamp: 0.0,
            atomicstarttimestamp: 0.0,
            endtimestamp: 0.0,
        }
    }
}
