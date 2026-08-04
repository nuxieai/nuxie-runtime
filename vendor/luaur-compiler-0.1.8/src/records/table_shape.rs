#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TableShape {
    pub(crate) array_size: core::ffi::c_uint,
    pub(crate) hash_size: core::ffi::c_uint,
}

impl luaur_common::records::dense_hash_table::DenseDefault for TableShape {
    fn dense_default() -> Self {
        Self::default()
    }
}
