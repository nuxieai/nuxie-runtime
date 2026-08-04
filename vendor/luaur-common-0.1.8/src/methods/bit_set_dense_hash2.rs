use alloc::vec;

use crate::records::bit_set::BitSet;

impl BitSet {
    pub(crate) fn new(capacity: usize) -> Self {
        debug_assert_eq!(capacity & capacity.wrapping_sub(1), 0);

        let count = if capacity == 0 {
            0
        } else if capacity < Self::NUM_ELEMENTS {
            1
        } else {
            capacity >> Self::NUM_ELEMENTS_LOG2
        };

        Self {
            data: vec![0; count],
        }
    }

    pub(crate) fn contains(&self, bucket: usize) -> bool {
        let word = bucket >> Self::NUM_ELEMENTS_LOG2;
        let offset = bucket & (Self::NUM_ELEMENTS - 1);
        ((self.data[word] >> offset) & 1) != 0
    }

    pub(crate) fn clear(&mut self) {
        self.data.fill(0);
    }

    pub(crate) fn set(&mut self, bucket: usize, value: bool) {
        let word = bucket >> Self::NUM_ELEMENTS_LOG2;
        let offset = bucket & (Self::NUM_ELEMENTS - 1);
        let mask = 1u64 << offset;

        if value {
            self.data[word] |= mask;
        } else {
            self.data[word] &= !mask;
        }
    }

    pub(crate) fn word_at(&self, index: usize) -> u64 {
        self.data[index]
    }

    pub(crate) fn num_words(&self) -> usize {
        self.data.len()
    }
}

impl Default for BitSet {
    fn default() -> Self {
        Self::new(0)
    }
}
