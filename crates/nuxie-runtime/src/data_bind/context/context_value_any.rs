//! Unsigned core-property coercion owned by C++ `ContextValueAny`.

pub(crate) fn number_to_core_uint(value: f32) -> Option<u64> {
    if value < 0.0 {
        return Some(0);
    }
    if !value.is_finite() {
        return None;
    }
    let rounded = value.round();
    (rounded < 2_147_483_648.0).then_some(rounded as u64)
}
