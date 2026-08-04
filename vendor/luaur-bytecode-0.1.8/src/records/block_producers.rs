use crate::records::bc_op::BcOp;
use crate::type_aliases::reg::Reg;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockProducers {
    pub(crate) own: HashMap<Reg, BcOp>,
    pub(crate) cached: HashMap<Reg, BcOp>,
    pub(crate) multiReturn: BcOp,
    pub(crate) multiReturnStart: Reg,
    pub(crate) invalidAfter: i32,
    pub(crate) sealed: bool,
    pub(crate) unsealed_preds: u32,
    pub(crate) incomplete_phis: HashMap<Reg, BcOp>,
}

impl Default for BlockProducers {
    fn default() -> Self {
        Self {
            own: HashMap::default(),
            cached: HashMap::default(),
            multiReturn: unsafe { std::mem::zeroed() },
            multiReturnStart: 0,
            invalidAfter: 255,
            sealed: false,
            unsealed_preds: 0,
            incomplete_phis: HashMap::default(),
        }
    }
}
