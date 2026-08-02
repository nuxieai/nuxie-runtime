pub(crate) fn bitmask_field_mask(bit: u8, width: u8) -> u64 {
    if bit >= 64 {
        return 0;
    }
    let width = width.min(64 - bit);
    let width_mask = if width >= 64 {
        u64::MAX
    } else {
        (1u64 << width) - 1
    };
    width_mask << bit
}
