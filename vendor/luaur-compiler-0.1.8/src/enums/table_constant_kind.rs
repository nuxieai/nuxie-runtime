#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TableConstantKind {
    ConstantTable,
    ConstantOther,
    NotConstant,
}

impl Default for TableConstantKind {
    fn default() -> Self {
        Self::ConstantTable
    }
}

impl luaur_common::records::dense_hash_table::DenseDefault for TableConstantKind {
    fn dense_default() -> Self {
        Self::default()
    }
}
