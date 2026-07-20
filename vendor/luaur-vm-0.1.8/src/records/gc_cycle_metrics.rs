#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GCCycleMetrics {
    pub starttotalsizebytes: usize,
    pub heaptriggersizebytes: usize,

    pub pausetime: f64,

    pub starttimestamp: f64,
    pub endtimestamp: f64,

    pub marktime: f64,
    pub markassisttime: f64,
    pub markmaxexplicittime: f64,
    pub markexplicitsteps: usize,
    pub markwork: usize,

    pub atomicstarttimestamp: f64,
    pub atomicstarttotalsizebytes: usize,
    pub atomictime: f64,

    pub atomictimeupval: f64,
    pub atomictimeweak: f64,
    pub atomictimegray: f64,
    pub atomictimeclear: f64,

    pub sweeptime: f64,
    pub sweepassisttime: f64,
    pub sweepmaxexplicittime: f64,
    pub sweepexplicitsteps: usize,
    pub sweepwork: usize,

    pub assistwork: usize,
    pub explicitwork: usize,

    pub propagatework: usize,
    pub propagateagainwork: usize,

    pub endtotalsizebytes: usize,
}

#[allow(non_upper_case_globals)]
pub type gc_cycle_metrics = GCCycleMetrics;
