use crate::records::l_value::LValue;

#[derive(Debug, Clone)]
pub struct Assignment {
    pub(crate) lvalue: LValue,
    pub(crate) conflict_reg: u8,
    pub(crate) value_reg: u8,
}

impl Assignment {
    pub(crate) const kInvalidReg: u8 = 255;
}

impl Default for Assignment {
    fn default() -> Self {
        Self {
            lvalue: unsafe { core::mem::zeroed() },
            conflict_reg: Self::kInvalidReg,
            value_reg: Self::kInvalidReg,
        }
    }
}
