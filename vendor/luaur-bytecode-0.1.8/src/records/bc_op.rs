use crate::enums::bc_op_kind::BcOpKind;
use luaur_common::records::dense_hash_table::DenseDefault;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BcOp {
    pub kind: BcOpKind,
    pub index: u32,
}

impl DenseDefault for BcOp {
    fn dense_default() -> Self {
        Self {
            kind: BcOpKind::None,
            index: 0,
        }
    }
}
