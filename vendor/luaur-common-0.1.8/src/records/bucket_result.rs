#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BucketResult {
    pub(crate) bucket: usize,
    pub(crate) found: bool,
}
