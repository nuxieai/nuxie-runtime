/// Mechanical owner for pinned C++ `BitFieldLoc`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeBitFieldLoc {
    start: u32,
    count: u32,
    mask: u32,
}

impl RuntimeBitFieldLoc {
    /// Pinned constructor order: retain `start`, validate the inclusive end,
    /// then derive and retain `count` and `mask`.
    pub(crate) fn new(start: u32, end: u32) -> Self {
        assert!(end >= start);
        assert!(end < 32);

        let count = end - start + 1;
        // Compute through u64 so the otherwise-valid inclusive range 0..31
        // has the intended all-bits mask instead of inheriting C++'s signed
        // `1 << 32` undefined behavior at Rust's safe boundary.
        let mask = ((((1u64 << count) - 1) << start) & u64::from(u32::MAX)) as u32;
        Self { start, count, mask }
    }

    pub(crate) fn read(self, bits: u32) -> u32 {
        (bits & self.mask) >> self.start
    }

    pub(crate) fn write(self, bits: u32, value: u32) -> u32 {
        (bits & !self.mask) | ((value << self.start) & self.mask)
    }
}

/// Adapter for the generated schema's `(bit, width)` representation and u64
/// field storage. Every generated passthrough range is validated to fit the
/// pinned owner's 32-bit inclusive range.
pub(crate) fn bitmask_field_mask(bit: u8, width: u8) -> u64 {
    let Some(end) = u32::from(bit)
        .checked_add(u32::from(width))
        .and_then(|exclusive| exclusive.checked_sub(1))
    else {
        return 0;
    };
    if end >= 32 {
        return 0;
    }
    u64::from(RuntimeBitFieldLoc::new(u32::from(bit), end).mask)
}
