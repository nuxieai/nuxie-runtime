use crate::records::gc_cycle_metrics::GCCycleMetrics;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct GCMetrics {
    pub(crate) stepexplicittimeacc: f64,
    pub(crate) stepassisttimeacc: f64,
    pub(crate) completedcycles: u64,
    pub(crate) lastcycle: GCCycleMetrics,
    pub(crate) currcycle: GCCycleMetrics,
}
