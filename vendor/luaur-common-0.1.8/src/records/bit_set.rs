use alloc::vec::Vec;

/// Occupancy bitmap used by `DenseHashTable2`.
#[derive(Clone, Debug)]
pub(crate) struct BitSet {
    pub(crate) data: Vec<u64>,
}

impl BitSet {
    pub(crate) const NUM_ELEMENTS: usize = u64::BITS as usize;
    pub(crate) const NUM_ELEMENTS_LOG2: usize = 6;
}
