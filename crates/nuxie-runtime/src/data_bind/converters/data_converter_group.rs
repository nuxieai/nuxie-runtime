//! Authored-order ownership matching C++ `DataConverterGroup`.

pub(crate) fn forward_indices(length: usize) -> impl Iterator<Item = usize> {
    0..length
}

pub(crate) fn reverse_indices(length: usize) -> impl Iterator<Item = usize> {
    (0..length).rev()
}
