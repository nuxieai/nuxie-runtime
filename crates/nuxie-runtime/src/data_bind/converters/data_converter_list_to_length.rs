//! Direct owner for C++ `DataConverterListToLength`.

pub(crate) fn convert(item_count: Option<usize>) -> f32 {
    item_count.unwrap_or(0) as f32
}
