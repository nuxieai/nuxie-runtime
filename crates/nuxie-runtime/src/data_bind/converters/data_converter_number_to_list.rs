//! Direct owner for C++ `DataConverterNumberToList`.

pub(crate) fn convert(value: f32, view_model_exists: bool) -> usize {
    const MAXIMUM_ITEMS: usize = 10_000;
    if !view_model_exists || !value.is_finite() {
        return 0;
    }
    value.floor().max(0.0).min(MAXIMUM_ITEMS as f32) as usize
}
