#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Local {
    pub(crate) reg: u8,
    pub(crate) allocated: bool,
    pub(crate) captured: bool,
    pub(crate) debugpc: u32,
    pub(crate) allocpc: u32,
}

impl Default for Local {
    fn default() -> Self {
        Self {
            reg: 0,
            allocated: false,
            captured: false,
            debugpc: 0,
            allocpc: 0,
        }
    }
}

impl luaur_common::records::dense_hash_table::DenseDefault for Local {
    fn dense_default() -> Self {
        Self::default()
    }
}
