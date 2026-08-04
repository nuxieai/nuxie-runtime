use crate::type_aliases::bc_ops::BcOps;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BcPhi {
    pub ops: BcOps,
}
